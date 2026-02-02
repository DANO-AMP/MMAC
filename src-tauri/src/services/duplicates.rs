use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateScanResult {
    pub groups: Vec<DuplicateGroup>,
    pub total_wasted: u64,
    pub files_scanned: u32,
}

pub struct DuplicateService;

impl DuplicateService {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_duplicates(&self, path: &str, min_size: u64) -> Result<DuplicateScanResult, String> {
        let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        let mut files_scanned = 0u32;

        // First pass: group files by size
        self.collect_files_by_size(path, &mut size_map, &mut files_scanned, min_size);

        // Second pass: hash files with same size
        let mut hash_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut file_sizes: HashMap<String, u64> = HashMap::new();

        for (size, paths) in size_map.iter() {
            if paths.len() > 1 {
                for file_path in paths {
                    if let Ok(hash) = self.hash_file(file_path) {
                        let path_str = file_path.to_string_lossy().to_string();
                        hash_map.entry(hash.clone()).or_default().push(path_str);
                        file_sizes.insert(hash, *size);
                    }
                }
            }
        }

        // Build result
        let mut groups: Vec<DuplicateGroup> = Vec::new();
        let mut total_wasted = 0u64;

        for (hash, files) in hash_map {
            if files.len() > 1 {
                let size = file_sizes.get(&hash).copied().unwrap_or(0);
                // Wasted space is (count - 1) * size (keep one copy)
                total_wasted += size * (files.len() as u64 - 1);
                groups.push(DuplicateGroup { hash, size, files });
            }
        }

        // Sort by wasted space (largest first)
        groups.sort_by(|a, b| {
            let wasted_a = a.size * (a.files.len() as u64 - 1);
            let wasted_b = b.size * (b.files.len() as u64 - 1);
            wasted_b.cmp(&wasted_a)
        });

        Ok(DuplicateScanResult {
            groups,
            total_wasted,
            files_scanned,
        })
    }

    fn collect_files_by_size(
        &self,
        path: &str,
        size_map: &mut HashMap<u64, Vec<PathBuf>>,
        count: &mut u32,
        min_size: u64,
    ) {
        let walker = jwalk::WalkDir::new(path)
            .skip_hidden(false)
            .follow_links(false);

        for entry in walker.into_iter().flatten() {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    let size = metadata.len();
                    if size >= min_size {
                        size_map.entry(size).or_default().push(entry.path());
                        *count += 1;
                    }
                }
            }
        }
    }

    fn hash_file(&self, path: &PathBuf) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();

        // Read in chunks for large files
        let mut buffer = [0u8; 65536];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // ---- Struct serialization tests ----

    #[test]
    fn test_duplicate_group_serialization_roundtrip() {
        let group = DuplicateGroup {
            hash: "abc123def456".to_string(),
            size: 1024,
            files: vec!["/tmp/a.txt".to_string(), "/tmp/b.txt".to_string()],
        };

        let json = serde_json::to_string(&group).expect("serialize");
        let deserialized: DuplicateGroup = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.hash, "abc123def456");
        assert_eq!(deserialized.size, 1024);
        assert_eq!(deserialized.files.len(), 2);
    }

    #[test]
    fn test_duplicate_scan_result_serialization_roundtrip() {
        let result = DuplicateScanResult {
            groups: vec![DuplicateGroup {
                hash: "abc".to_string(),
                size: 500,
                files: vec!["f1".to_string(), "f2".to_string()],
            }],
            total_wasted: 500,
            files_scanned: 10,
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: DuplicateScanResult = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.groups.len(), 1);
        assert_eq!(deserialized.total_wasted, 500);
        assert_eq!(deserialized.files_scanned, 10);
    }

    #[test]
    fn test_duplicate_scan_result_empty() {
        let result = DuplicateScanResult {
            groups: vec![],
            total_wasted: 0,
            files_scanned: 0,
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: DuplicateScanResult = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.groups.is_empty());
        assert_eq!(deserialized.total_wasted, 0);
    }

    // ---- File hashing tests ----

    #[test]
    fn test_hash_file_produces_consistent_hash() {
        let dir = TempDir::new().expect("create tempdir");
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").expect("write");

        let service = DuplicateService::new();
        let hash1 = service.hash_file(&file_path).expect("hash1");
        let hash2 = service.hash_file(&file_path).expect("hash2");

        assert_eq!(hash1, hash2, "Same file should produce the same hash");
    }

    #[test]
    fn test_hash_file_different_content_different_hash() {
        let dir = TempDir::new().expect("create tempdir");

        let file_a = dir.path().join("a.txt");
        std::fs::write(&file_a, b"content A").expect("write a");

        let file_b = dir.path().join("b.txt");
        std::fs::write(&file_b, b"content B").expect("write b");

        let service = DuplicateService::new();
        let hash_a = service.hash_file(&file_a).expect("hash a");
        let hash_b = service.hash_file(&file_b).expect("hash b");

        assert_ne!(hash_a, hash_b, "Different content should produce different hashes");
    }

    #[test]
    fn test_hash_file_same_content_same_hash() {
        let dir = TempDir::new().expect("create tempdir");

        let file_a = dir.path().join("a.txt");
        std::fs::write(&file_a, b"identical content").expect("write a");

        let file_b = dir.path().join("b.txt");
        std::fs::write(&file_b, b"identical content").expect("write b");

        let service = DuplicateService::new();
        let hash_a = service.hash_file(&file_a).expect("hash a");
        let hash_b = service.hash_file(&file_b).expect("hash b");

        assert_eq!(hash_a, hash_b, "Identical content should produce identical hashes");
    }

    #[test]
    fn test_hash_file_empty_file() {
        let dir = TempDir::new().expect("create tempdir");
        let file_path = dir.path().join("empty.txt");
        std::fs::write(&file_path, b"").expect("write");

        let service = DuplicateService::new();
        let hash = service.hash_file(&file_path).expect("hash");

        // SHA256 of empty data is a well-known constant
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_hash_file_nonexistent_returns_error() {
        let service = DuplicateService::new();
        let result = service.hash_file(&PathBuf::from("/nonexistent/path/file.txt"));

        assert!(result.is_err());
    }

    #[test]
    fn test_hash_file_large_content() {
        let dir = TempDir::new().expect("create tempdir");
        let file_path = dir.path().join("large.bin");
        // Create a file larger than the 65536 byte buffer
        let mut f = File::create(&file_path).expect("create");
        let chunk = vec![0xABu8; 100_000];
        f.write_all(&chunk).expect("write");
        drop(f);

        let service = DuplicateService::new();
        let hash = service.hash_file(&file_path).expect("hash");

        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64, "SHA256 hex string should be 64 chars");
    }

    // ---- Scan duplicates integration tests with real temp files ----

    #[test]
    fn test_scan_duplicates_finds_duplicate_files() {
        let dir = TempDir::new().expect("create tempdir");

        // Create two files with identical content
        let content = b"duplicate content here, enough bytes to pass any threshold";
        std::fs::write(dir.path().join("file1.txt"), content).expect("write1");
        std::fs::write(dir.path().join("file2.txt"), content).expect("write2");

        let service = DuplicateService::new();
        let result = service.scan_duplicates(
            dir.path().to_str().unwrap(),
            0, // no min size
        ).expect("scan");

        assert!(result.files_scanned >= 2);
        assert_eq!(result.groups.len(), 1, "Should find one group of duplicates");
        assert_eq!(result.groups[0].files.len(), 2);
        assert_eq!(result.groups[0].size, content.len() as u64);
        // Wasted space = size * (2 - 1) = one copy wasted
        assert_eq!(result.total_wasted, content.len() as u64);
    }

    #[test]
    fn test_scan_duplicates_no_duplicates() {
        let dir = TempDir::new().expect("create tempdir");

        std::fs::write(dir.path().join("a.txt"), b"unique content A").expect("write a");
        std::fs::write(dir.path().join("b.txt"), b"unique content BB").expect("write b");

        let service = DuplicateService::new();
        let result = service.scan_duplicates(
            dir.path().to_str().unwrap(),
            0,
        ).expect("scan");

        assert!(result.groups.is_empty(), "No duplicate groups expected");
        assert_eq!(result.total_wasted, 0);
    }

    #[test]
    fn test_scan_duplicates_respects_min_size() {
        let dir = TempDir::new().expect("create tempdir");

        // Create two files with identical content but smaller than min_size
        let small_content = b"small";
        std::fs::write(dir.path().join("s1.txt"), small_content).expect("write1");
        std::fs::write(dir.path().join("s2.txt"), small_content).expect("write2");

        let service = DuplicateService::new();
        let result = service.scan_duplicates(
            dir.path().to_str().unwrap(),
            1000, // min size larger than file content
        ).expect("scan");

        assert!(result.groups.is_empty(), "Small files should be filtered out");
        assert_eq!(result.files_scanned, 0);
    }

    #[test]
    fn test_scan_duplicates_empty_directory() {
        let dir = TempDir::new().expect("create tempdir");

        let service = DuplicateService::new();
        let result = service.scan_duplicates(
            dir.path().to_str().unwrap(),
            0,
        ).expect("scan");

        assert!(result.groups.is_empty());
        assert_eq!(result.total_wasted, 0);
        assert_eq!(result.files_scanned, 0);
    }

    #[test]
    fn test_scan_duplicates_three_copies() {
        let dir = TempDir::new().expect("create tempdir");

        let content = b"triplicate content right here with sufficient bytes";
        std::fs::write(dir.path().join("f1.txt"), content).expect("write1");
        std::fs::write(dir.path().join("f2.txt"), content).expect("write2");
        std::fs::write(dir.path().join("f3.txt"), content).expect("write3");

        let service = DuplicateService::new();
        let result = service.scan_duplicates(
            dir.path().to_str().unwrap(),
            0,
        ).expect("scan");

        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].files.len(), 3);
        // Wasted = (3 - 1) * size = 2 * size
        assert_eq!(result.total_wasted, content.len() as u64 * 2);
    }

    #[test]
    fn test_scan_duplicates_sorted_by_wasted_space() {
        let dir = TempDir::new().expect("create tempdir");

        // Group A: small duplicates
        let small = b"small duplicate content!!";
        std::fs::write(dir.path().join("sa1.txt"), small).expect("write");
        std::fs::write(dir.path().join("sa2.txt"), small).expect("write");

        // Group B: large duplicates (more wasted space)
        let large = vec![0x42u8; 10_000];
        std::fs::write(dir.path().join("la1.bin"), &large).expect("write");
        std::fs::write(dir.path().join("la2.bin"), &large).expect("write");

        let service = DuplicateService::new();
        let result = service.scan_duplicates(
            dir.path().to_str().unwrap(),
            0,
        ).expect("scan");

        assert_eq!(result.groups.len(), 2);
        // First group should have more wasted space
        let wasted_0 = result.groups[0].size * (result.groups[0].files.len() as u64 - 1);
        let wasted_1 = result.groups[1].size * (result.groups[1].files.len() as u64 - 1);
        assert!(wasted_0 >= wasted_1, "Groups should be sorted by wasted space descending");
    }

    // ---- collect_files_by_size tests ----

    #[test]
    fn test_collect_files_by_size_groups_correctly() {
        let dir = TempDir::new().expect("create tempdir");

        // Two files of same size
        std::fs::write(dir.path().join("a.txt"), b"AAAA").expect("write");
        std::fs::write(dir.path().join("b.txt"), b"BBBB").expect("write");
        // One file of different size
        std::fs::write(dir.path().join("c.txt"), b"CCCCCC").expect("write");

        let service = DuplicateService::new();
        let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        let mut count = 0u32;

        service.collect_files_by_size(
            dir.path().to_str().unwrap(),
            &mut size_map,
            &mut count,
            0,
        );

        assert_eq!(count, 3);
        // size 4 group should have 2 files
        assert_eq!(size_map.get(&4).map(|v| v.len()), Some(2));
        // size 6 group should have 1 file
        assert_eq!(size_map.get(&6).map(|v| v.len()), Some(1));
    }
}
