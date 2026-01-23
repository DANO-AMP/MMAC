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

    pub async fn analyze(&self, path: &str) -> Result<Vec<DiskItem>, Box<dyn std::error::Error + Send + Sync>> {
        let expanded_path = shellexpand::tilde(path).to_string();
        let target_path = PathBuf::from(&expanded_path);

        if !target_path.exists() {
            return Err("Path does not exist".into());
        }

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
                let display_path = if expanded_path.starts_with(&dirs::home_dir().unwrap().to_string_lossy().to_string()) {
                    entry_path
                        .to_string_lossy()
                        .replace(&dirs::home_dir().unwrap().to_string_lossy().to_string(), "~")
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
