use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrphanedFile {
    pub path: String,
    pub size: u64,
    pub likely_app: String,
    pub last_accessed: i64,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanedScanResult {
    pub files: Vec<OrphanedFile>,
    pub total_size: u64,
    pub total_count: u32,
}

pub struct OrphanedService;

impl OrphanedService {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_orphaned_files(&self) -> Result<OrphanedScanResult, String> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;

        // Get list of installed apps (bundle IDs and names)
        let installed_apps = self.get_installed_apps(&home);

        let mut orphaned_files = Vec::new();
        let mut total_size = 0u64;

        // Scan various Library locations
        let scan_paths = vec![
            (home.join("Library/Application Support"), "data"),
            (home.join("Library/Preferences"), "pref"),
            (home.join("Library/Caches"), "cache"),
            (home.join("Library/Containers"), "container"),
            (home.join("Library/Group Containers"), "group_container"),
            (home.join("Library/Saved Application State"), "state"),
        ];

        for (path, file_type) in scan_paths {
            if path.exists() {
                self.scan_directory(&path, &installed_apps, file_type, &mut orphaned_files)?;
            }
        }

        // Calculate total size
        total_size = orphaned_files.iter().map(|f| f.size).sum();
        let total_count = orphaned_files.len() as u32;

        // Sort by size descending
        orphaned_files.sort_by(|a, b| b.size.cmp(&a.size));

        Ok(OrphanedScanResult {
            files: orphaned_files,
            total_size,
            total_count,
        })
    }

    fn get_installed_apps(&self, home: &PathBuf) -> HashSet<String> {
        let mut apps = HashSet::new();

        // Scan /Applications
        if let Ok(entries) = fs::read_dir("/Applications") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".app") {
                        let app_name = name.trim_end_matches(".app").to_lowercase();
                        apps.insert(app_name.clone());

                        // Try to get bundle ID
                        if let Some(bundle_id) = self.get_bundle_id(&entry.path()) {
                            apps.insert(bundle_id.to_lowercase());
                            // Also add parts of the bundle ID
                            for part in bundle_id.split('.') {
                                if part.len() > 3 {
                                    apps.insert(part.to_lowercase());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Scan ~/Applications
        let user_apps = home.join("Applications");
        if let Ok(entries) = fs::read_dir(&user_apps) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".app") {
                        let app_name = name.trim_end_matches(".app").to_lowercase();
                        apps.insert(app_name.clone());

                        if let Some(bundle_id) = self.get_bundle_id(&entry.path()) {
                            apps.insert(bundle_id.to_lowercase());
                            for part in bundle_id.split('.') {
                                if part.len() > 3 {
                                    apps.insert(part.to_lowercase());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add common system apps/services that should not be flagged
        let system_apps = [
            "apple", "com.apple", "macos", "finder", "safari", "mail",
            "spotlight", "siri", "dock", "systemuiserver", "loginwindow",
            "launchservices", "xcode", "simulator", "developer", "coreservices",
            "cloudkit", "icloud", "itunes", "music", "podcasts", "news",
            "stocks", "home", "photos", "preview", "quicktime", "terminal",
            "textedit", "notes", "reminders", "calendar", "contacts", "messages",
            "facetime", "maps", "books", "appstore", "systempreferences",
            "activitymonitor", "diskutility", "keychain", "fontbook", "grapher",
            "screensaver", "bluetooth", "wifi", "audio", "video", "print",
            "plist", "default", "global", "shared", "common", "cache", "temp",
        ];
        for app in system_apps {
            apps.insert(app.to_string());
        }

        apps
    }

    fn get_bundle_id(&self, app_path: &PathBuf) -> Option<String> {
        let plist_path = app_path.join("Contents/Info.plist");
        if !plist_path.exists() {
            return None;
        }

        // Read plist and extract CFBundleIdentifier
        if let Ok(content) = fs::read_to_string(&plist_path) {
            // Simple parsing - look for CFBundleIdentifier
            if let Some(start) = content.find("<key>CFBundleIdentifier</key>") {
                let after_key = &content[start..];
                if let Some(string_start) = after_key.find("<string>") {
                    let value_start = string_start + 8;
                    if let Some(string_end) = after_key[value_start..].find("</string>") {
                        let bundle_id = &after_key[value_start..value_start + string_end];
                        return Some(bundle_id.to_string());
                    }
                }
            }
        }

        None
    }

    fn scan_directory(
        &self,
        path: &PathBuf,
        installed_apps: &HashSet<String>,
        file_type: &str,
        orphaned: &mut Vec<OrphanedFile>,
    ) -> Result<(), String> {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files and some system files
                if name.starts_with('.') || name == "CloudKit" || name == "Apple" {
                    continue;
                }

                // Check if this entry belongs to an installed app
                let name_lower = name.to_lowercase();
                let is_orphaned = !self.is_app_related(&name_lower, installed_apps);

                if is_orphaned {
                    let size = self.calculate_size(&entry_path);

                    // Only include if size is significant (> 1MB)
                    if size > 1_000_000 {
                        let likely_app = self.extract_app_name(&name);
                        let last_accessed = self.get_last_accessed(&entry_path);

                        orphaned.push(OrphanedFile {
                            path: entry_path.to_string_lossy().to_string(),
                            size,
                            likely_app,
                            last_accessed,
                            file_type: file_type.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn is_app_related(&self, name: &str, installed_apps: &HashSet<String>) -> bool {
        // Check direct match
        if installed_apps.contains(name) {
            return true;
        }

        // Check if any installed app name is contained in this name
        for app in installed_apps {
            if name.contains(app) || app.contains(name) {
                return true;
            }
        }

        // Check bundle ID style names (com.company.app)
        for part in name.split('.') {
            if part.len() > 3 && installed_apps.contains(part) {
                return true;
            }
        }

        false
    }

    fn calculate_size(&self, path: &PathBuf) -> u64 {
        if path.is_file() {
            fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        } else if path.is_dir() {
            WalkDir::new(path)
                .max_depth(5)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum()
        } else {
            0
        }
    }

    fn extract_app_name(&self, name: &str) -> String {
        // Try to extract a readable app name from the file/folder name
        // e.g., "com.somecompany.someapp" -> "Someapp"
        // e.g., "SomeApp Helper" -> "SomeApp"

        let cleaned = name
            .trim_end_matches(".plist")
            .trim_end_matches(".savedState");

        // Handle bundle ID format
        if cleaned.contains('.') {
            let parts: Vec<&str> = cleaned.split('.').collect();
            if let Some(last) = parts.last() {
                return self.capitalize_first(last);
            }
        }

        // Handle space-separated format
        if cleaned.contains(' ') {
            let parts: Vec<&str> = cleaned.split(' ').collect();
            if let Some(first) = parts.first() {
                return first.to_string();
            }
        }

        cleaned.to_string()
    }

    fn capitalize_first(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    fn get_last_accessed(&self, path: &PathBuf) -> i64 {
        fs::metadata(path)
            .and_then(|m| m.accessed())
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    pub fn delete_orphaned(&self, path: &str) -> Result<u64, String> {
        let path_buf = PathBuf::from(path);

        if !path_buf.exists() {
            return Err("Path does not exist".to_string());
        }

        // Calculate size before deletion
        let size = self.calculate_size(&path_buf);

        // Move to trash instead of permanent deletion
        trash::delete(&path_buf).map_err(|e| format!("Failed to delete: {}", e))?;

        Ok(size)
    }
}
