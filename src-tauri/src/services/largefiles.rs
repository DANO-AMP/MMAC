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
