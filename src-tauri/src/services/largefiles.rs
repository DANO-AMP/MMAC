use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LargeFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: u64,
}

pub struct LargeFilesService;

impl LargeFilesService {
    pub fn new() -> Self {
        Self
    }

    pub fn find_large_files(&self, path: &str, min_size: u64, limit: usize) -> Vec<LargeFile> {
        let mut files: Vec<LargeFile> = Vec::new();

        let walker = jwalk::WalkDir::new(path)
            .skip_hidden(false)
            .follow_links(false);

        for entry in walker.into_iter().flatten() {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    let size = metadata.len();
                    if size >= min_size {
                        let modified = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0);

                        files.push(LargeFile {
                            path: entry.path().to_string_lossy().to_string(),
                            name: entry.file_name().to_string_lossy().to_string(),
                            size,
                            modified,
                        });
                    }
                }
            }
        }

        // Sort by size descending
        files.sort_by(|a, b| b.size.cmp(&a.size));

        // Limit results
        files.truncate(limit);

        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ---- Struct serialization tests ----

    #[test]
    fn test_large_file_serialization_roundtrip() {
        let file = LargeFile {
            path: "/Users/me/bigfile.zip".to_string(),
            name: "bigfile.zip".to_string(),
            size: 1_073_741_824, // 1 GB
            modified: 1700000000,
        };

        let json = serde_json::to_string(&file).expect("serialize");
        let deserialized: LargeFile = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.path, "/Users/me/bigfile.zip");
        assert_eq!(deserialized.name, "bigfile.zip");
        assert_eq!(deserialized.size, 1_073_741_824);
        assert_eq!(deserialized.modified, 1700000000);
    }

    #[test]
    fn test_large_file_serialization_zero_values() {
        let file = LargeFile {
            path: "".to_string(),
            name: "".to_string(),
            size: 0,
            modified: 0,
        };

        let json = serde_json::to_string(&file).expect("serialize");
        let deserialized: LargeFile = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.size, 0);
        assert_eq!(deserialized.modified, 0);
    }

    // ---- find_large_files tests with tempdir ----

    #[test]
    fn test_find_large_files_returns_files_above_threshold() {
        let dir = TempDir::new().expect("create tempdir");

        // Create a file above threshold (100 bytes)
        let big_content = vec![0u8; 200];
        std::fs::write(dir.path().join("big.bin"), &big_content).expect("write big");

        // Create a file below threshold
        std::fs::write(dir.path().join("small.txt"), b"tiny").expect("write small");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 100, 10);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "big.bin");
        assert_eq!(result[0].size, 200);
    }

    #[test]
    fn test_find_large_files_respects_limit() {
        let dir = TempDir::new().expect("create tempdir");

        // Create 5 files all above the threshold
        for i in 0..5 {
            let content = vec![0u8; 500 + i * 100];
            std::fs::write(dir.path().join(format!("file{}.bin", i)), &content).expect("write");
        }

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 100, 3);

        assert_eq!(result.len(), 3, "Should be limited to 3 results");
    }

    #[test]
    fn test_find_large_files_sorted_by_size_descending() {
        let dir = TempDir::new().expect("create tempdir");

        std::fs::write(dir.path().join("small.bin"), &vec![0u8; 100]).expect("write");
        std::fs::write(dir.path().join("medium.bin"), &vec![0u8; 500]).expect("write");
        std::fs::write(dir.path().join("large.bin"), &vec![0u8; 1000]).expect("write");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 50, 100);

        assert_eq!(result.len(), 3);
        assert!(result[0].size >= result[1].size);
        assert!(result[1].size >= result[2].size);
        assert_eq!(result[0].name, "large.bin");
    }

    #[test]
    fn test_find_large_files_empty_directory() {
        let dir = TempDir::new().expect("create tempdir");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 0, 100);

        assert!(result.is_empty());
    }

    #[test]
    fn test_find_large_files_min_size_zero_returns_all() {
        let dir = TempDir::new().expect("create tempdir");

        std::fs::write(dir.path().join("a.txt"), b"a").expect("write");
        std::fs::write(dir.path().join("b.txt"), b"bb").expect("write");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 0, 100);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_find_large_files_subdirectories() {
        let dir = TempDir::new().expect("create tempdir");
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).expect("create subdir");

        std::fs::write(subdir.join("nested.bin"), &vec![0u8; 300]).expect("write nested");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 100, 100);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "nested.bin");
        assert!(result[0].path.contains("subdir"));
    }

    #[test]
    fn test_find_large_files_limit_zero_returns_empty() {
        let dir = TempDir::new().expect("create tempdir");

        std::fs::write(dir.path().join("file.bin"), &vec![0u8; 1000]).expect("write");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 0, 0);

        assert!(result.is_empty(), "Limit 0 should return no files");
    }

    #[test]
    fn test_find_large_files_has_valid_modified_timestamp() {
        let dir = TempDir::new().expect("create tempdir");
        std::fs::write(dir.path().join("file.txt"), b"content here").expect("write");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 0, 10);

        assert_eq!(result.len(), 1);
        // modified should be a reasonable unix timestamp (after 2020-01-01 = 1577836800)
        assert!(result[0].modified > 1577836800, "Modified timestamp should be recent");
    }

    #[test]
    fn test_find_large_files_path_contains_full_path() {
        let dir = TempDir::new().expect("create tempdir");
        std::fs::write(dir.path().join("check_path.dat"), &vec![0u8; 100]).expect("write");

        let service = LargeFilesService::new();
        let result = service.find_large_files(dir.path().to_str().unwrap(), 0, 10);

        assert_eq!(result.len(), 1);
        assert!(result[0].path.ends_with("check_path.dat"));
        assert!(result[0].path.starts_with(dir.path().to_str().unwrap()));
    }

    #[test]
    fn test_find_large_files_threshold_exactly_at_size() {
        let dir = TempDir::new().expect("create tempdir");
        std::fs::write(dir.path().join("exact.bin"), &vec![0u8; 100]).expect("write");

        let service = LargeFilesService::new();
        // min_size equals the file size -- should be included (>= check)
        let result = service.find_large_files(dir.path().to_str().unwrap(), 100, 10);

        assert_eq!(result.len(), 1, "File exactly at threshold should be included");
    }
}
