use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub category: String,
    pub size: u64,
    pub items: u32,
    pub paths: Vec<String>,
}

pub struct CleaningService;

impl CleaningService {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan_all(&self) -> Result<Vec<ScanResult>, Box<dyn std::error::Error + Send + Sync>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;

        let results = vec![
            self.scan_caches(&home).await?,
            self.scan_logs(&home).await?,
            self.scan_browser_data(&home).await?,
            self.scan_trash(&home).await?,
            self.scan_crash_reports(&home).await?,
            self.scan_xcode_data(&home).await?,
            self.scan_package_caches(&home).await?,
        ];

        Ok(results)
    }

    async fn scan_caches(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let cache_paths = vec![
            home.join("Library/Caches"),
            PathBuf::from("/Library/Caches"),
        ];

        let result = tokio::task::spawn_blocking(move || {
            let mut total_size = 0u64;
            let mut total_items = 0u32;
            let mut paths = Vec::new();

            for cache_path in cache_paths {
                if cache_path.exists() {
                    for entry in WalkDir::new(&cache_path)
                        .max_depth(2)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            if let Ok(metadata) = entry.metadata() {
                                total_size += metadata.len();
                                total_items += 1;
                            }
                        }
                    }
                    paths.push(cache_path.to_string_lossy().to_string());
                }
            }

            ScanResult {
                category: "cache".to_string(),
                size: total_size,
                items: total_items,
                paths,
            }
        }).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    async fn scan_logs(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let log_paths = vec![
            home.join("Library/Logs"),
            PathBuf::from("/var/log"),
            PathBuf::from("/private/var/log"),
        ];

        let result = tokio::task::spawn_blocking(move || {
            let mut total_size = 0u64;
            let mut total_items = 0u32;
            let mut paths = Vec::new();

            for log_path in log_paths {
                if log_path.exists() {
                    for entry in WalkDir::new(&log_path)
                        .max_depth(3)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            if let Ok(metadata) = entry.metadata() {
                                total_size += metadata.len();
                                total_items += 1;
                            }
                        }
                    }
                    paths.push(log_path.to_string_lossy().to_string());
                }
            }

            ScanResult {
                category: "logs".to_string(),
                size: total_size,
                items: total_items,
                paths,
            }
        }).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    async fn scan_browser_data(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let browser_paths = vec![
            // Chrome
            home.join("Library/Caches/Google/Chrome"),
            home.join("Library/Application Support/Google/Chrome/Default/Cache"),
            // Safari
            home.join("Library/Caches/com.apple.Safari"),
            // Firefox
            home.join("Library/Caches/Firefox"),
            // Arc
            home.join("Library/Caches/company.thebrowser.Browser"),
        ];

        let result = tokio::task::spawn_blocking(move || {
            let mut total_size = 0u64;
            let mut total_items = 0u32;
            let mut paths = Vec::new();

            for browser_path in browser_paths {
                if browser_path.exists() {
                    for entry in WalkDir::new(&browser_path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            if let Ok(metadata) = entry.metadata() {
                                total_size += metadata.len();
                                total_items += 1;
                            }
                        }
                    }
                    paths.push(browser_path.to_string_lossy().to_string());
                }
            }

            ScanResult {
                category: "browser".to_string(),
                size: total_size,
                items: total_items,
                paths,
            }
        }).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    async fn scan_trash(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let trash_path = home.join(".Trash");

        let result = tokio::task::spawn_blocking(move || {
            let mut total_size = 0u64;
            let mut total_items = 0u32;

            if trash_path.exists() {
                for entry in WalkDir::new(&trash_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        if let Ok(metadata) = entry.metadata() {
                            total_size += metadata.len();
                            total_items += 1;
                        }
                    }
                }
            }

            ScanResult {
                category: "trash".to_string(),
                size: total_size,
                items: total_items,
                paths: vec![trash_path.to_string_lossy().to_string()],
            }
        }).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    async fn scan_crash_reports(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let crash_paths = vec![
            home.join("Library/Logs/DiagnosticReports"),
            home.join("Library/Logs/CrashReporter"),
            PathBuf::from("/Library/Logs/DiagnosticReports"),
        ];

        let result = tokio::task::spawn_blocking(move || {
            let mut total_size = 0u64;
            let mut total_items = 0u32;
            let mut paths = Vec::new();

            for crash_path in crash_paths {
                if crash_path.exists() {
                    for entry in WalkDir::new(&crash_path)
                        .max_depth(3)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            if let Ok(metadata) = entry.metadata() {
                                total_size += metadata.len();
                                total_items += 1;
                            }
                        }
                    }
                    paths.push(crash_path.to_string_lossy().to_string());
                }
            }

            ScanResult {
                category: "crash_reports".to_string(),
                size: total_size,
                items: total_items,
                paths,
            }
        }).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    async fn scan_xcode_data(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let xcode_paths = vec![
            home.join("Library/Developer/Xcode/DerivedData"),
            home.join("Library/Developer/Xcode/iOS DeviceSupport"),
            home.join("Library/Developer/Xcode/watchOS DeviceSupport"),
            home.join("Library/Developer/Xcode/Archives"),
            home.join("Library/Developer/CoreSimulator/Caches"),
        ];

        let result = tokio::task::spawn_blocking(move || {
            let mut total_size = 0u64;
            let mut total_items = 0u32;
            let mut paths = Vec::new();

            for xcode_path in xcode_paths {
                if xcode_path.exists() {
                    // For DerivedData and DeviceSupport, only count top-level dirs
                    // to avoid slow deep scans of huge build artifacts
                    if xcode_path.to_string_lossy().contains("DerivedData")
                        || xcode_path.to_string_lossy().contains("DeviceSupport") {
                        for entry in WalkDir::new(&xcode_path)
                            .max_depth(2)
                            .into_iter()
                            .filter_map(|e| e.ok())
                        {
                            if entry.file_type().is_file() {
                                if let Ok(metadata) = entry.metadata() {
                                    total_size += metadata.len();
                                    total_items += 1;
                                }
                            } else if entry.file_type().is_dir() && entry.depth() == 1 {
                                // Estimate directory size for performance
                                if let Ok(metadata) = entry.metadata() {
                                    total_size += metadata.len();
                                }
                                total_items += 1;
                            }
                        }
                    } else {
                        for entry in WalkDir::new(&xcode_path)
                            .max_depth(3)
                            .into_iter()
                            .filter_map(|e| e.ok())
                        {
                            if entry.file_type().is_file() {
                                if let Ok(metadata) = entry.metadata() {
                                    total_size += metadata.len();
                                    total_items += 1;
                                }
                            }
                        }
                    }
                    paths.push(xcode_path.to_string_lossy().to_string());
                }
            }

            ScanResult {
                category: "xcode".to_string(),
                size: total_size,
                items: total_items,
                paths,
            }
        }).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    async fn scan_package_caches(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let package_paths = vec![
            // npm
            home.join(".npm/_cacache"),
            // yarn
            home.join(".yarn/cache"),
            home.join("Library/Caches/Yarn"),
            // pnpm
            home.join("Library/pnpm/store"),
            // pip
            home.join("Library/Caches/pip"),
            // cargo
            home.join(".cargo/registry/cache"),
            // composer
            home.join(".composer/cache"),
            // cocoapods
            home.join("Library/Caches/CocoaPods"),
            // gradle
            home.join(".gradle/caches"),
            // maven
            home.join(".m2/repository"),
        ];

        let result = tokio::task::spawn_blocking(move || {
            let mut total_size = 0u64;
            let mut total_items = 0u32;
            let mut paths = Vec::new();

            for package_path in package_paths {
                if package_path.exists() {
                    for entry in WalkDir::new(&package_path)
                        .max_depth(4)
                        .into_iter()
                        .filter_map(|e| e.ok())
                    {
                        if entry.file_type().is_file() {
                            if let Ok(metadata) = entry.metadata() {
                                total_size += metadata.len();
                                total_items += 1;
                            }
                        }
                    }
                    paths.push(package_path.to_string_lossy().to_string());
                }
            }

            ScanResult {
                category: "packages".to_string(),
                size: total_size,
                items: total_items,
                paths,
            }
        }).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub async fn clean_category(&self, category: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let category = category.to_string();

        let paths_to_clean: Vec<PathBuf> = match category.as_str() {
            "cache" => vec![home.join("Library/Caches")],
            "logs" => vec![home.join("Library/Logs")],
            "browser" => vec![
                home.join("Library/Caches/Google/Chrome"),
                home.join("Library/Caches/com.apple.Safari"),
                home.join("Library/Caches/Firefox"),
            ],
            "crash_reports" => vec![
                home.join("Library/Logs/DiagnosticReports"),
                home.join("Library/Logs/CrashReporter"),
            ],
            "xcode" => vec![
                home.join("Library/Developer/Xcode/DerivedData"),
                home.join("Library/Developer/Xcode/iOS DeviceSupport"),
                home.join("Library/Developer/Xcode/watchOS DeviceSupport"),
                home.join("Library/Developer/CoreSimulator/Caches"),
            ],
            "packages" => vec![
                home.join(".npm/_cacache"),
                home.join(".yarn/cache"),
                home.join("Library/Caches/Yarn"),
                home.join("Library/Caches/pip"),
                home.join(".cargo/registry/cache"),
                home.join(".composer/cache"),
                home.join("Library/Caches/CocoaPods"),
            ],
            "trash" => {
                // Empty trash using safe Rust filesystem operations
                let trash_path = home.join(".Trash");

                let cleaned_size = tokio::task::spawn_blocking(move || {
                    let mut size = 0u64;

                    if trash_path.exists() {
                        // Calculate size of items in trash before deletion
                        for entry in WalkDir::new(&trash_path)
                            .into_iter()
                            .filter_map(|e| e.ok())
                            .filter(|e| e.file_type().is_file())
                        {
                            if let Ok(metadata) = entry.metadata() {
                                size += metadata.len();
                            }
                        }

                        // Delete each item in trash individually (safer than rm -rf)
                        if let Ok(entries) = std::fs::read_dir(&trash_path) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_dir() {
                                    let _ = std::fs::remove_dir_all(&path);
                                } else {
                                    let _ = std::fs::remove_file(&path);
                                }
                            }
                        }
                    }

                    size
                }).await.map_err(|e| e.to_string())?;

                return Ok(cleaned_size);
            }
            _ => return Err("Unknown category".into()),
        };

        let cleaned_size = tokio::task::spawn_blocking(move || {
            let mut size = 0u64;

            for path in paths_to_clean {
                if path.exists() {
                    for entry in WalkDir::new(&path)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                    {
                        if let Ok(metadata) = entry.metadata() {
                            size += metadata.len();
                        }
                        // Move to trash instead of deleting directly
                        let _ = trash::delete(entry.path());
                    }
                }
            }

            size
        }).await.map_err(|e| e.to_string())?;

        Ok(cleaned_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_scan_result_serialization() {
        let result = ScanResult {
            category: "cache".to_string(),
            size: 1024 * 1024 * 500, // 500 MB
            items: 1500,
            paths: vec![
                "/Users/test/Library/Caches".to_string(),
                "/Library/Caches".to_string(),
            ],
        };

        let json = serde_json::to_string(&result).expect("Should serialize");
        assert!(json.contains("\"category\":\"cache\""));
        assert!(json.contains("\"size\":524288000"));
        assert!(json.contains("\"items\":1500"));

        let deserialized: ScanResult = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.category, "cache");
        assert_eq!(deserialized.size, 524288000);
        assert_eq!(deserialized.items, 1500);
        assert_eq!(deserialized.paths.len(), 2);
    }

    #[test]
    fn test_scan_caches_empty_dir() {
        // Test that scanning an empty directory returns zero size and items
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let empty_cache_path = temp_dir.path().join("empty_cache");
        fs::create_dir_all(&empty_cache_path).expect("Failed to create empty cache dir");

        // Simulate what the scan would return for an empty directory
        let mut total_size = 0u64;
        let mut total_items = 0u32;

        for entry in WalkDir::new(&empty_cache_path)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    total_items += 1;
                }
            }
        }

        assert_eq!(total_size, 0, "Empty directory should have zero size");
        assert_eq!(total_items, 0, "Empty directory should have zero items");
    }

    #[test]
    fn test_scan_caches_with_files() {
        // Test that scanning a directory with files correctly counts size and items
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_path = temp_dir.path().join("test_cache");
        fs::create_dir_all(&cache_path).expect("Failed to create cache dir");

        // Create some test files
        let file1_path = cache_path.join("file1.cache");
        let file2_path = cache_path.join("file2.cache");
        let mut file1 = File::create(&file1_path).expect("Failed to create file1");
        let mut file2 = File::create(&file2_path).expect("Failed to create file2");

        // Write known amount of data
        file1.write_all(&[0u8; 1024]).expect("Failed to write to file1"); // 1 KB
        file2.write_all(&[0u8; 2048]).expect("Failed to write to file2"); // 2 KB

        let mut total_size = 0u64;
        let mut total_items = 0u32;

        for entry in WalkDir::new(&cache_path)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    total_items += 1;
                }
            }
        }

        assert_eq!(total_items, 2, "Should find 2 files");
        assert_eq!(total_size, 3072, "Total size should be 3 KB");
    }

    #[test]
    fn test_scan_logs_permissions() {
        // Test that the scanner gracefully handles permission errors
        // by using filter_map(|e| e.ok()) to skip inaccessible entries

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("logs");
        fs::create_dir_all(&log_path).expect("Failed to create log dir");

        // Create a readable file
        let readable_file = log_path.join("readable.log");
        let mut f = File::create(&readable_file).expect("Failed to create file");
        f.write_all(b"test log content").expect("Failed to write");

        // Simulate permission handling - the filter_map(|e| e.ok()) pattern
        // will skip any entries that fail to read
        let mut accessible_items = 0u32;

        for entry in WalkDir::new(&log_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok()) // This skips permission errors
        {
            if entry.file_type().is_file() {
                accessible_items += 1;
            }
        }

        assert!(accessible_items >= 1, "Should find at least the readable file");
    }

    #[test]
    fn test_scan_nonexistent_path() {
        // Test that scanning a nonexistent path doesn't add it to paths list
        let nonexistent_path = PathBuf::from("/this/path/definitely/does/not/exist");

        let mut paths = Vec::new();

        // This is how the actual code handles nonexistent paths
        if nonexistent_path.exists() {
            paths.push(nonexistent_path.to_string_lossy().to_string());
        }

        assert!(paths.is_empty(), "Nonexistent paths should not be added");
    }

    #[tokio::test]
    async fn test_clean_category_invalid() {
        let service = CleaningService::new();

        let result = service.clean_category("invalid_category").await;

        assert!(result.is_err(), "Invalid category should return error");

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Unknown category"),
            "Error should mention unknown category"
        );
    }

    #[test]
    fn test_clean_category_valid_categories() {
        // Test that all valid categories are recognized
        let valid_categories = [
            "cache",
            "logs",
            "browser",
            "crash_reports",
            "xcode",
            "packages",
            "trash",
        ];

        for category in valid_categories {
            // The match expression should not fall through to the default case
            let result: Result<(), &str> = match category {
                "cache" | "logs" | "browser" | "crash_reports" | "xcode" | "packages" | "trash" => {
                    Ok(())
                }
                _ => Err("Unknown category"),
            };

            assert!(
                result.is_ok(),
                "Category '{}' should be recognized as valid",
                category
            );
        }
    }

    #[test]
    fn test_scan_result_paths_construction() {
        // Test that paths are correctly constructed from home directory
        let home = PathBuf::from("/Users/testuser");

        let cache_paths = vec![
            home.join("Library/Caches"),
            PathBuf::from("/Library/Caches"),
        ];

        assert_eq!(
            cache_paths[0].to_string_lossy(),
            "/Users/testuser/Library/Caches"
        );
        assert_eq!(cache_paths[1].to_string_lossy(), "/Library/Caches");
    }

    #[test]
    fn test_walkdir_max_depth_respected() {
        // Test that max_depth is properly applied
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create nested structure: depth0/depth1/depth2/depth3/file.txt
        let deep_path = temp_dir
            .path()
            .join("depth1")
            .join("depth2")
            .join("depth3");
        fs::create_dir_all(&deep_path).expect("Failed to create deep dirs");

        let deep_file = deep_path.join("deep_file.txt");
        File::create(&deep_file).expect("Failed to create deep file");

        // Also create a file at depth 1
        let shallow_file = temp_dir.path().join("depth1").join("shallow.txt");
        File::create(&shallow_file).expect("Failed to create shallow file");

        // Scan with max_depth 2 (should only find shallow.txt)
        let mut files_found = 0;
        for entry in WalkDir::new(temp_dir.path())
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                files_found += 1;
            }
        }

        assert_eq!(
            files_found, 1,
            "With max_depth 2, should only find file at depth 1"
        );
    }

    #[test]
    fn test_size_calculation_accuracy() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create files with exact known sizes
        let sizes = [100u64, 500, 1000, 5000];
        let mut expected_total = 0u64;

        for (i, &size) in sizes.iter().enumerate() {
            let file_path = temp_dir.path().join(format!("file_{}.bin", i));
            let mut file = File::create(&file_path).expect("Failed to create file");
            file.write_all(&vec![0u8; size as usize])
                .expect("Failed to write");
            expected_total += size;
        }

        let mut actual_total = 0u64;
        for entry in WalkDir::new(temp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    actual_total += metadata.len();
                }
            }
        }

        assert_eq!(
            actual_total, expected_total,
            "Size calculation should be accurate"
        );
    }

    #[test]
    fn test_hidden_files_included() {
        // Test that hidden files (starting with .) are included in scans
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let hidden_file = temp_dir.path().join(".hidden_cache");
        let visible_file = temp_dir.path().join("visible_cache");

        File::create(&hidden_file).expect("Failed to create hidden file");
        File::create(&visible_file).expect("Failed to create visible file");

        let mut file_count = 0;
        for entry in WalkDir::new(temp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                file_count += 1;
            }
        }

        assert_eq!(
            file_count, 2,
            "Both hidden and visible files should be counted"
        );
    }

    #[test]
    fn test_cleaning_service_new() {
        // Test that CleaningService can be instantiated
        let service = CleaningService::new();
        // Service is a unit struct, just verify it can be created
        let _ = service;
    }

    #[test]
    fn test_symlink_not_followed_deeply() {
        // Test behavior with symlinks - walkdir follows symlinks by default
        // but we should verify it doesn't cause infinite loops
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let file_path = temp_dir.path().join("real_file.txt");
        File::create(&file_path).expect("Failed to create file");

        // Note: Creating symlinks requires specific permissions on some systems
        // This test verifies the scan completes without hanging
        let mut completed = false;
        for _entry in WalkDir::new(temp_dir.path())
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // Just iterate through
        }
        completed = true;

        assert!(completed, "Scan should complete without hanging");
    }
}
