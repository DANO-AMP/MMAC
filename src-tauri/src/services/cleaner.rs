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
        ];

        Ok(results)
    }

    async fn scan_caches(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let cache_paths = vec![
            home.join("Library/Caches"),
            PathBuf::from("/Library/Caches"),
        ];

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

        Ok(ScanResult {
            category: "cache".to_string(),
            size: total_size,
            items: total_items,
            paths,
        })
    }

    async fn scan_logs(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let log_paths = vec![
            home.join("Library/Logs"),
            PathBuf::from("/var/log"),
            PathBuf::from("/private/var/log"),
        ];

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

        Ok(ScanResult {
            category: "logs".to_string(),
            size: total_size,
            items: total_items,
            paths,
        })
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

        Ok(ScanResult {
            category: "browser".to_string(),
            size: total_size,
            items: total_items,
            paths,
        })
    }

    async fn scan_trash(&self, home: &PathBuf) -> Result<ScanResult, Box<dyn std::error::Error + Send + Sync>> {
        let trash_path = home.join(".Trash");

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

        Ok(ScanResult {
            category: "trash".to_string(),
            size: total_size,
            items: total_items,
            paths: vec![trash_path.to_string_lossy().to_string()],
        })
    }

    pub async fn clean_category(&self, category: &str) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let mut cleaned_size = 0u64;

        let paths_to_clean: Vec<PathBuf> = match category {
            "cache" => vec![home.join("Library/Caches")],
            "logs" => vec![home.join("Library/Logs")],
            "browser" => vec![
                home.join("Library/Caches/Google/Chrome"),
                home.join("Library/Caches/com.apple.Safari"),
                home.join("Library/Caches/Firefox"),
            ],
            "trash" => {
                // Empty trash using system command
                std::process::Command::new("rm")
                    .args(["-rf", &home.join(".Trash/*").to_string_lossy()])
                    .output()?;
                return Ok(0);
            }
            _ => return Err("Unknown category".into()),
        };

        for path in paths_to_clean {
            if path.exists() {
                for entry in WalkDir::new(&path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    if let Ok(metadata) = entry.metadata() {
                        cleaned_size += metadata.len();
                    }
                    // Move to trash instead of deleting directly
                    let _ = trash::delete(entry.path());
                }
            }
        }

        Ok(cleaned_size)
    }
}
