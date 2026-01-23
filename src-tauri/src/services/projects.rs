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
