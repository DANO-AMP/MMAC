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
