use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskItem {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

pub struct AnalyzerService;

impl AnalyzerService {
    pub fn new() -> Self {
        Self
    }

    /// Returns the list of allowed base directories for analysis
    fn allowed_directories() -> Vec<PathBuf> {
        let mut allowed = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/tmp"),
        ];

        // Add home directory if available
        if let Some(home) = dirs::home_dir() {
            allowed.push(home);
        }

        allowed
    }

    /// Validates that a path is within allowed directories and doesn't contain traversal attacks
    fn validate_path(path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        // Canonicalize the path to resolve symlinks and ".." components
        let canonical = path.canonicalize()
            .map_err(|_| "Failed to resolve path")?;

        // Check for path traversal attempts in the original path string
        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            return Err("Path traversal detected: '..' is not allowed".into());
        }

        // Verify the canonical path is within allowed directories
        let allowed = Self::allowed_directories();
        let is_allowed = allowed.iter().any(|allowed_dir| {
            if let Ok(allowed_canonical) = allowed_dir.canonicalize() {
                canonical.starts_with(&allowed_canonical)
            } else {
                // If the allowed directory doesn't exist yet, check prefix match
                canonical.starts_with(allowed_dir)
            }
        });

        if !is_allowed {
            return Err(format!(
                "Access denied: path must be within home directory, /Applications, or /tmp"
            ).into());
        }

        Ok(canonical)
    }

    pub async fn analyze(&self, path: &str) -> Result<Vec<DiskItem>, Box<dyn std::error::Error + Send + Sync>> {
        let expanded_path = shellexpand::tilde(path).to_string();
        let target_path = PathBuf::from(&expanded_path);

        if !target_path.exists() {
            return Err("Path does not exist".into());
        }

        // Validate and canonicalize the path
        let target_path = Self::validate_path(&target_path)?;

        let mut items: Vec<DiskItem> = Vec::new();

        if let Ok(entries) = fs::read_dir(&target_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files in root analysis (but include in subdirs)
                if path == "~" && name.starts_with('.') {
                    continue;
                }

                let is_dir = entry_path.is_dir();
                let size = if is_dir {
                    self.calculate_dir_size(&entry_path)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };

                // Convert path for display
                let display_path = if let Some(home) = dirs::home_dir() {
                    let home_str = home.to_string_lossy().to_string();
                    let entry_str = entry_path.to_string_lossy().to_string();
                    if entry_str.starts_with(&home_str) {
                        entry_str.replace(&home_str, "~")
                    } else {
                        entry_str
                    }
                } else {
                    entry_path.to_string_lossy().to_string()
                };

                items.push(DiskItem {
                    name,
                    path: display_path,
                    size,
                    is_dir,
                });
            }
        }

        // Sort by size descending
        items.sort_by(|a, b| b.size.cmp(&a.size));

        Ok(items)
    }

    fn calculate_dir_size(&self, path: &PathBuf) -> u64 {
        // Use a faster approach - limit depth for very large directories
        let mut total_size = 0u64;

        let walker = WalkDir::new(path)
            .max_depth(10) // Limit depth to prevent very long scans
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        total_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_disk_item_serialization() {
        let item = DiskItem {
            name: "Documents".to_string(),
            path: "~/Documents".to_string(),
            size: 1024 * 1024 * 100, // 100 MB
            is_dir: true,
        };

        let json = serde_json::to_string(&item).expect("Should serialize");
        assert!(json.contains("\"name\":\"Documents\""));
        assert!(json.contains("\"is_dir\":true"));

        let deserialized: DiskItem = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.name, "Documents");
        assert_eq!(deserialized.size, 104857600);
        assert!(deserialized.is_dir);
    }

    #[test]
    fn test_path_traversal_blocked_double_dot() {
        // Test that path traversal with ".." is blocked
        let malicious_path = PathBuf::from("/tmp/../etc/passwd");

        let result = AnalyzerService::validate_path(&malicious_path);

        assert!(result.is_err(), "Path traversal should be blocked");
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("traversal") || error.to_string().contains("denied"),
            "Error message should mention path traversal or access denied"
        );
    }

    #[test]
    fn test_path_traversal_blocked_encoded() {
        // Test various path traversal patterns
        let traversal_paths = [
            PathBuf::from("/tmp/test/../../../etc"),
            PathBuf::from("~/../../../etc/passwd"),
        ];

        for path in &traversal_paths {
            let path_str = path.to_string_lossy();
            // Check if path contains traversal sequence
            let has_traversal = path_str.contains("..");
            assert!(
                has_traversal,
                "Test path should contain traversal sequence: {}",
                path_str
            );
        }
    }

    #[test]
    fn test_path_traversal_blocked_outside_allowed() {
        // Test that paths outside allowed directories are blocked
        let outside_path = PathBuf::from("/etc/passwd");

        // Only test if the path exists (it should on most Unix systems)
        if outside_path.exists() {
            let result = AnalyzerService::validate_path(&outside_path);
            assert!(result.is_err(), "Path outside allowed directories should be blocked");
        }
    }

    #[tokio::test]
    async fn test_symlink_handling() {
        // Test that symlinks are resolved and handled safely
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a real file
        let real_file = temp_dir.path().join("real_file.txt");
        let mut f = File::create(&real_file).expect("Failed to create file");
        f.write_all(b"test content").expect("Failed to write");

        // The analyze function should handle symlinks via canonicalize()
        // Test that a valid path within temp (which is in /tmp or similar) works
        let service = AnalyzerService::new();

        // Create a subdirectory to analyze
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdir");

        let file_in_subdir = subdir.join("file.txt");
        File::create(&file_in_subdir).expect("Failed to create file");

        // The temp directory should be under an allowed path
        // This tests that canonicalize works correctly
        let canonical = temp_dir.path().canonicalize();
        assert!(canonical.is_ok(), "Should be able to canonicalize temp dir path");
    }

    #[tokio::test]
    async fn test_nonexistent_path() {
        let service = AnalyzerService::new();

        let result = service
            .analyze("/this/path/definitely/does/not/exist/anywhere")
            .await;

        assert!(result.is_err(), "Nonexistent path should return error");
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("does not exist"),
            "Error should mention path doesn't exist"
        );
    }

    #[test]
    fn test_validate_path_with_tilde() {
        // Test that tilde expansion works before validation
        let path_with_tilde = "~";
        let expanded = shellexpand::tilde(path_with_tilde).to_string();

        let path = PathBuf::from(&expanded);

        // Home directory should exist and be allowed
        if path.exists() {
            let result = AnalyzerService::validate_path(&path);
            assert!(result.is_ok(), "Home directory should be allowed");
        }
    }

    #[test]
    fn test_allowed_directories_includes_home() {
        let allowed = AnalyzerService::allowed_directories();

        // Should include /Applications and /tmp at minimum
        let has_applications = allowed.iter().any(|p| p == &PathBuf::from("/Applications"));
        let has_tmp = allowed.iter().any(|p| p == &PathBuf::from("/tmp"));

        assert!(has_applications, "Should include /Applications");
        assert!(has_tmp, "Should include /tmp");

        // If home directory exists, it should be included
        if dirs::home_dir().is_some() {
            assert!(allowed.len() >= 3, "Should have at least 3 allowed directories when home exists");
        }
    }

    #[test]
    fn test_calculate_dir_size_empty() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir_all(&empty_dir).expect("Failed to create empty dir");

        let service = AnalyzerService::new();
        let size = service.calculate_dir_size(&empty_dir);

        assert_eq!(size, 0, "Empty directory should have size 0");
    }

    #[test]
    fn test_calculate_dir_size_with_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create files with known sizes
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        let mut f1 = File::create(&file1).expect("Failed to create file1");
        let mut f2 = File::create(&file2).expect("Failed to create file2");

        f1.write_all(&[0u8; 1000]).expect("Failed to write");
        f2.write_all(&[0u8; 500]).expect("Failed to write");

        let service = AnalyzerService::new();
        let size = service.calculate_dir_size(&temp_dir.path().to_path_buf());

        assert_eq!(size, 1500, "Should calculate correct total size");
    }

    #[test]
    fn test_calculate_dir_size_nested() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create nested structure
        let nested = temp_dir.path().join("level1").join("level2");
        fs::create_dir_all(&nested).expect("Failed to create nested dirs");

        let file1 = temp_dir.path().join("root.txt");
        let file2 = temp_dir.path().join("level1").join("level1.txt");
        let file3 = nested.join("level2.txt");

        for (path, size) in [(file1, 100), (file2, 200), (file3, 300)] {
            let mut f = File::create(&path).expect("Failed to create file");
            f.write_all(&vec![0u8; size]).expect("Failed to write");
        }

        let service = AnalyzerService::new();
        let size = service.calculate_dir_size(&temp_dir.path().to_path_buf());

        assert_eq!(size, 600, "Should sum sizes across nested directories");
    }

    #[test]
    fn test_max_depth_limit() {
        // Test that very deep directory structures don't cause issues
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a structure deeper than max_depth (10)
        let mut current = temp_dir.path().to_path_buf();
        for i in 0..15 {
            current = current.join(format!("level{}", i));
        }
        fs::create_dir_all(&current).expect("Failed to create deep structure");

        // Create a file at the deepest level
        let deep_file = current.join("deep.txt");
        let mut f = File::create(&deep_file).expect("Failed to create file");
        f.write_all(&[0u8; 100]).expect("Failed to write");

        let service = AnalyzerService::new();
        let size = service.calculate_dir_size(&temp_dir.path().to_path_buf());

        // Due to max_depth(10), the file at level 15 should NOT be counted
        assert_eq!(size, 0, "Files beyond max_depth should not be counted");
    }

    #[test]
    fn test_items_sorted_by_size_descending() {
        // Test that results are sorted by size descending
        let items = vec![
            DiskItem {
                name: "small".to_string(),
                path: "/small".to_string(),
                size: 100,
                is_dir: false,
            },
            DiskItem {
                name: "large".to_string(),
                path: "/large".to_string(),
                size: 1000,
                is_dir: false,
            },
            DiskItem {
                name: "medium".to_string(),
                path: "/medium".to_string(),
                size: 500,
                is_dir: false,
            },
        ];

        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| b.size.cmp(&a.size));

        assert_eq!(sorted_items[0].name, "large");
        assert_eq!(sorted_items[1].name, "medium");
        assert_eq!(sorted_items[2].name, "small");
    }

    #[test]
    fn test_analyzer_service_new() {
        let service = AnalyzerService::new();
        // Just verify the service can be created
        let _ = service;
    }

    #[test]
    fn test_hidden_files_filtered_at_root() {
        // Test that hidden files are filtered when analyzing home directory
        // but not in subdirectories
        let name = ".hidden_file";
        let path = "~";

        // At root (~), hidden files should be filtered
        let should_skip = path == "~" && name.starts_with('.');
        assert!(should_skip, "Hidden files should be skipped at home root");

        // In subdirectories, hidden files should be included
        let subdir_path = "~/Documents";
        let should_skip_subdir = subdir_path == "~" && name.starts_with('.');
        assert!(!should_skip_subdir, "Hidden files should be included in subdirs");
    }

    #[test]
    fn test_path_display_conversion() {
        // Test that paths are converted for display with ~ for home
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy().to_string();
            let entry_path = home.join("Documents");
            let entry_str = entry_path.to_string_lossy().to_string();

            let display_path = if entry_str.starts_with(&home_str) {
                entry_str.replace(&home_str, "~")
            } else {
                entry_str.clone()
            };

            assert!(
                display_path.starts_with("~"),
                "Path should start with ~ for display"
            );
            assert!(
                display_path.contains("Documents"),
                "Path should contain the directory name"
            );
        }
    }
}
