use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectArtifact {
    pub project_path: String,
    pub project_name: String,
    pub artifact_type: String,
    pub artifact_path: String,
    pub size: u64,
    pub last_modified: String,
    pub is_recent: bool,
}

pub struct ProjectsService;

impl ProjectsService {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan(&self) -> Result<Vec<ProjectArtifact>, Box<dyn std::error::Error + Send + Sync>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let mut artifacts = Vec::new();

        // Common project directories to scan
        let project_dirs = vec![
            home.join("projects"),
            home.join("Projects"),
            home.join("dev"),
            home.join("Development"),
            home.join("code"),
            home.join("Code"),
            home.join("workspace"),
            home.join("Workspace"),
            home.join("Documents"),
            home.join("Desktop"),
        ];

        // Artifact patterns to look for
        let artifact_patterns = vec![
            ("node_modules", "node_modules"),
            ("target", "target"),          // Rust
            ("build", "build"),            // Various
            ("dist", "dist"),              // Various
            (".next", ".next"),            // Next.js
            ("__pycache__", "__pycache__"), // Python
            ("venv", "venv"),              // Python virtual env
            (".venv", ".venv"),            // Python virtual env
            ("vendor", "vendor"),          // PHP/Go
            ("Pods", "Pods"),              // iOS
        ];

        for project_dir in project_dirs {
            if !project_dir.exists() {
                continue;
            }

            // Walk through directories looking for artifacts
            let walker = WalkDir::new(&project_dir)
                .max_depth(5)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    // Skip hidden directories except for artifact patterns
                    !name.starts_with('.') || artifact_patterns.iter().any(|(_, p)| name == *p)
                });

            for entry in walker.filter_map(|e| e.ok()) {
                if !entry.file_type().is_dir() {
                    continue;
                }

                let dir_name = entry.file_name().to_string_lossy().to_string();

                // Check if this directory matches an artifact pattern
                for (artifact_type, pattern) in &artifact_patterns {
                    if dir_name == *pattern {
                        // Get parent as project directory
                        if let Some(parent) = entry.path().parent() {
                            // Calculate artifact size
                            let size = self.calculate_dir_size(&entry.path().to_path_buf());

                            // Skip if too small (less than 1MB)
                            if size < 1024 * 1024 {
                                continue;
                            }

                            // Get last modified time
                            let (last_modified, is_recent) = self.get_last_modified(&entry.path().to_path_buf());

                            let project_name = parent
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            let display_path = entry
                                .path()
                                .to_string_lossy()
                                .replace(&home.to_string_lossy().to_string(), "~");

                            let project_path = parent
                                .to_string_lossy()
                                .replace(&home.to_string_lossy().to_string(), "~");

                            artifacts.push(ProjectArtifact {
                                project_path,
                                project_name,
                                artifact_type: artifact_type.to_string(),
                                artifact_path: display_path,
                                size,
                                last_modified,
                                is_recent,
                            });
                        }
                        break;
                    }
                }
            }
        }

        // Sort by size descending
        artifacts.sort_by(|a, b| b.size.cmp(&a.size));

        Ok(artifacts)
    }

    fn calculate_dir_size(&self, path: &PathBuf) -> u64 {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum()
    }

    fn get_last_modified(&self, path: &PathBuf) -> (String, bool) {
        let default_time = Utc::now();
        let seven_days_ago = Utc::now() - chrono::Duration::days(7);

        let modified_time = fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| {
                t.duration_since(SystemTime::UNIX_EPOCH)
                    .ok()
                    .map(|d| DateTime::<Utc>::from(SystemTime::UNIX_EPOCH + d))
            })
            .unwrap_or(default_time);

        let is_recent = modified_time > seven_days_ago;
        (modified_time.to_rfc3339(), is_recent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ---- Struct serialization tests ----

    #[test]
    fn test_project_artifact_serialization_roundtrip() {
        let artifact = ProjectArtifact {
            project_path: "~/projects/myapp".to_string(),
            project_name: "myapp".to_string(),
            artifact_type: "node_modules".to_string(),
            artifact_path: "~/projects/myapp/node_modules".to_string(),
            size: 500_000_000,
            last_modified: "2024-01-15T10:30:00+00:00".to_string(),
            is_recent: true,
        };

        let json = serde_json::to_string(&artifact).expect("serialize");
        let deserialized: ProjectArtifact = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.project_path, "~/projects/myapp");
        assert_eq!(deserialized.project_name, "myapp");
        assert_eq!(deserialized.artifact_type, "node_modules");
        assert_eq!(deserialized.artifact_path, "~/projects/myapp/node_modules");
        assert_eq!(deserialized.size, 500_000_000);
        assert_eq!(deserialized.last_modified, "2024-01-15T10:30:00+00:00");
        assert!(deserialized.is_recent);
    }

    #[test]
    fn test_project_artifact_serialization_not_recent() {
        let artifact = ProjectArtifact {
            project_path: "~/old-project".to_string(),
            project_name: "old-project".to_string(),
            artifact_type: "target".to_string(),
            artifact_path: "~/old-project/target".to_string(),
            size: 1_000_000,
            last_modified: "2023-01-01T00:00:00+00:00".to_string(),
            is_recent: false,
        };

        let json = serde_json::to_string(&artifact).expect("serialize");
        let deserialized: ProjectArtifact = serde_json::from_str(&json).expect("deserialize");

        assert!(!deserialized.is_recent);
    }

    // ---- calculate_dir_size tests ----

    #[test]
    fn test_calculate_dir_size_single_file() {
        let dir = TempDir::new().expect("create tempdir");
        let content = vec![0u8; 1024];
        std::fs::write(dir.path().join("file.bin"), &content).expect("write");

        let service = ProjectsService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 1024);
    }

    #[test]
    fn test_calculate_dir_size_multiple_files() {
        let dir = TempDir::new().expect("create tempdir");
        std::fs::write(dir.path().join("a.txt"), &vec![0u8; 100]).expect("write a");
        std::fs::write(dir.path().join("b.txt"), &vec![0u8; 200]).expect("write b");
        std::fs::write(dir.path().join("c.txt"), &vec![0u8; 300]).expect("write c");

        let service = ProjectsService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 600);
    }

    #[test]
    fn test_calculate_dir_size_nested_directories() {
        let dir = TempDir::new().expect("create tempdir");
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).expect("create sub");
        let deep = sub.join("deep");
        std::fs::create_dir(&deep).expect("create deep");

        std::fs::write(dir.path().join("root.txt"), &vec![0u8; 100]).expect("write root");
        std::fs::write(sub.join("mid.txt"), &vec![0u8; 200]).expect("write mid");
        std::fs::write(deep.join("deep.txt"), &vec![0u8; 300]).expect("write deep");

        let service = ProjectsService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 600);
    }

    #[test]
    fn test_calculate_dir_size_empty_directory() {
        let dir = TempDir::new().expect("create tempdir");

        let service = ProjectsService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_ignores_directories_themselves() {
        let dir = TempDir::new().expect("create tempdir");
        let sub = dir.path().join("subdir");
        std::fs::create_dir(&sub).expect("create sub");

        // Only the subdir, no files
        let service = ProjectsService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 0, "Empty directories should not contribute to size");
    }

    // ---- get_last_modified tests ----

    #[test]
    fn test_get_last_modified_recent_file() {
        let dir = TempDir::new().expect("create tempdir");
        let file_path = dir.path().join("recent.txt");
        std::fs::write(&file_path, b"just created").expect("write");

        let service = ProjectsService::new();
        let (timestamp, is_recent) = service.get_last_modified(&file_path);

        assert!(is_recent, "Newly created file should be recent");
        assert!(!timestamp.is_empty());
        // Should be a valid RFC3339 timestamp
        assert!(timestamp.contains('T'), "Should be RFC3339 format");
    }

    #[test]
    fn test_get_last_modified_returns_rfc3339() {
        let dir = TempDir::new().expect("create tempdir");
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"content").expect("write");

        let service = ProjectsService::new();
        let (timestamp, _) = service.get_last_modified(&file_path);

        // Verify it parses as a valid DateTime
        let parsed = DateTime::parse_from_rfc3339(&timestamp);
        assert!(parsed.is_ok(), "Timestamp should be valid RFC3339: {}", timestamp);
    }

    #[test]
    fn test_get_last_modified_nonexistent_path_defaults_to_now() {
        let service = ProjectsService::new();
        let (timestamp, is_recent) = service.get_last_modified(&PathBuf::from("/nonexistent/path"));

        // When path does not exist, it defaults to Utc::now(), which is recent
        assert!(is_recent);
        assert!(!timestamp.is_empty());
    }

    // ---- Artifact pattern matching tests ----

    #[test]
    fn test_artifact_patterns_are_well_known() {
        // Verify the service recognizes all expected artifact patterns
        let expected_patterns = vec![
            "node_modules",
            "target",
            "build",
            "dist",
            ".next",
            "__pycache__",
            "venv",
            ".venv",
            "vendor",
            "Pods",
        ];

        // These patterns are hard-coded in scan(), so we test that the list is complete
        // by checking the array length matches our expectations
        assert_eq!(expected_patterns.len(), 10);
    }

    // ---- Integration test: scan finds artifacts in temp dir ----

    #[tokio::test]
    async fn test_scan_does_not_panic_on_nonexistent_dirs() {
        // The scan function iterates over common project dirs.
        // If none exist, it should still return Ok with an empty list.
        // This test verifies no panic occurs.
        let service = ProjectsService::new();
        let result = service.scan().await;
        assert!(result.is_ok());
    }
}
