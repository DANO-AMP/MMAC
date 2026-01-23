use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemnantFile {
    pub path: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub remnant_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub bundle_id: String,
    pub path: String,
    pub size: u64,
    pub version: Option<String>,
    pub remnants: Vec<RemnantFile>,
    pub remnants_size: u64,
}

pub struct UninstallerService;

impl UninstallerService {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_apps(&self) -> Result<Vec<AppInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let apps_path = PathBuf::from("/Applications");
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let user_apps_path = home.join("Applications");

        let mut apps = Vec::new();

        // Scan /Applications
        if let Ok(entries) = fs::read_dir(&apps_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(app_info) = self.parse_app(&entry.path(), &home).await {
                    apps.push(app_info);
                }
            }
        }

        // Scan ~/Applications
        if let Ok(entries) = fs::read_dir(&user_apps_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(app_info) = self.parse_app(&entry.path(), &home).await {
                    apps.push(app_info);
                }
            }
        }

        // Sort by name
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(apps)
    }

    async fn parse_app(&self, path: &PathBuf, home: &PathBuf) -> Option<AppInfo> {
        if !path.extension().map(|e| e == "app").unwrap_or(false) {
            return None;
        }

        let info_plist_path = path.join("Contents/Info.plist");
        if !info_plist_path.exists() {
            return None;
        }

        // Parse Info.plist
        let plist: plist::Value = plist::from_file(&info_plist_path).ok()?;
        let dict = plist.as_dictionary()?;

        let bundle_id = dict
            .get("CFBundleIdentifier")?
            .as_string()?
            .to_string();

        let name = dict
            .get("CFBundleName")
            .or_else(|| dict.get("CFBundleDisplayName"))
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| path.file_stem().unwrap().to_str().unwrap())
            .to_string();

        let version = dict
            .get("CFBundleShortVersionString")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        // Calculate app size
        let size = self.calculate_dir_size(path);

        // Find remnants
        let remnants = self.find_remnants(&bundle_id, &name, home);
        let remnants_size: u64 = remnants.iter().map(|r| r.size).sum();

        Some(AppInfo {
            name,
            bundle_id,
            path: path.to_string_lossy().to_string(),
            size,
            version,
            remnants,
            remnants_size,
        })
    }

    fn find_remnants(&self, bundle_id: &str, app_name: &str, home: &PathBuf) -> Vec<RemnantFile> {
        let mut remnants = Vec::new();

        // Locations to check
        let search_locations = vec![
            (home.join("Library/Application Support"), "data"),
            (home.join("Library/Caches"), "cache"),
            (home.join("Library/Preferences"), "pref"),
            (home.join("Library/Containers"), "container"),
            (home.join("Library/Saved Application State"), "state"),
            (home.join("Library/Cookies"), "cookies"),
            (home.join("Library/LaunchAgents"), "agent"),
            (home.join("Library/HTTPStorages"), "storage"),
            (home.join("Library/WebKit"), "webkit"),
            (home.join("Library/Logs"), "logs"),
            (PathBuf::from("/Library/LaunchAgents"), "agent"),
            (PathBuf::from("/Library/LaunchDaemons"), "daemon"),
        ];

        // Patterns to match
        let patterns = vec![
            bundle_id.to_string(),
            app_name.to_lowercase().replace(' ', ""),
            app_name.to_lowercase().replace(' ', "."),
        ];

        for (location, remnant_type) in search_locations {
            if !location.exists() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&location) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let file_name = entry.file_name().to_string_lossy().to_lowercase();

                    for pattern in &patterns {
                        if file_name.contains(&pattern.to_lowercase()) {
                            let size = if entry.path().is_dir() {
                                self.calculate_dir_size(&entry.path())
                            } else {
                                entry.metadata().map(|m| m.len()).unwrap_or(0)
                            };

                            remnants.push(RemnantFile {
                                path: entry.path().to_string_lossy().to_string(),
                                size,
                                remnant_type: remnant_type.to_string(),
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Check for plist files specifically
        let prefs_path = home.join("Library/Preferences");
        if prefs_path.exists() {
            if let Ok(entries) = fs::read_dir(&prefs_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(bundle_id) && file_name.ends_with(".plist") {
                        remnants.push(RemnantFile {
                            path: entry.path().to_string_lossy().to_string(),
                            size: entry.metadata().map(|m| m.len()).unwrap_or(0),
                            remnant_type: "pref".to_string(),
                        });
                    }
                }
            }
        }

        remnants
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

    pub async fn uninstall(
        &self,
        bundle_id: &str,
        include_remnants: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let apps = self.list_apps().await?;
        let app = apps
            .iter()
            .find(|a| a.bundle_id == bundle_id)
            .ok_or("App not found")?;

        // Move app to trash
        trash::delete(&app.path)?;

        // Remove remnants if requested
        if include_remnants {
            for remnant in &app.remnants {
                let _ = trash::delete(&remnant.path);
            }
        }

        Ok(())
    }
}
