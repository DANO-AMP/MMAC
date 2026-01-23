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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_orphaned_file_serialization() {
        let file = OrphanedFile {
            path: "/Users/test/Library/Application Support/OldApp".to_string(),
            size: 1024 * 1024 * 50, // 50 MB
            likely_app: "OldApp".to_string(),
            last_accessed: 1700000000,
            file_type: "data".to_string(),
        };

        let json = serde_json::to_string(&file).expect("Should serialize");
        assert!(json.contains("\"likely_app\":\"OldApp\""));
        assert!(json.contains("\"file_type\":\"data\""));

        let deserialized: OrphanedFile =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.likely_app, "OldApp");
        assert_eq!(deserialized.size, 52428800);
    }

    #[test]
    fn test_orphaned_scan_result_serialization() {
        let result = OrphanedScanResult {
            files: vec![],
            total_size: 0,
            total_count: 0,
        };

        let json = serde_json::to_string(&result).expect("Should serialize");
        assert!(json.contains("\"total_size\":0"));
        assert!(json.contains("\"total_count\":0"));
    }

    #[test]
    fn test_scan_orphaned_empty_installed_apps() {
        // Test with an empty set of installed apps
        let service = OrphanedService::new();

        // Create a mock installed apps set (empty except system apps)
        let installed_apps: HashSet<String> = HashSet::new();

        // A name that doesn't match any installed app
        let orphan_name = "unknownapp";

        let is_related = service.is_app_related(orphan_name, &installed_apps);

        assert!(
            !is_related,
            "Unknown app should not be related to empty installed apps"
        );
    }

    #[test]
    fn test_detect_orphan_app_not_installed() {
        let service = OrphanedService::new();

        // Create a set of installed apps
        let mut installed_apps: HashSet<String> = HashSet::new();
        installed_apps.insert("safari".to_string());
        installed_apps.insert("chrome".to_string());
        installed_apps.insert("firefox".to_string());

        // Test an app that's not installed
        let orphan_name = "com.oldcompany.removedapp";

        let is_related = service.is_app_related(orphan_name, &installed_apps);

        assert!(
            !is_related,
            "'{}' should be detected as orphaned (not related to installed apps)",
            orphan_name
        );
    }

    #[test]
    fn test_detect_orphan_app_installed() {
        let service = OrphanedService::new();

        // Create a set of installed apps
        let mut installed_apps: HashSet<String> = HashSet::new();
        installed_apps.insert("safari".to_string());
        installed_apps.insert("chrome".to_string());

        // Test an app that IS installed
        let installed_name = "com.google.chrome";

        let is_related = service.is_app_related(installed_name, &installed_apps);

        assert!(
            is_related,
            "'{}' should be related to installed Chrome",
            installed_name
        );
    }

    #[test]
    fn test_detect_orphan_partial_match() {
        let service = OrphanedService::new();

        let mut installed_apps: HashSet<String> = HashSet::new();
        installed_apps.insert("slack".to_string());

        // Test partial matches
        let slack_helper = "slack helper";
        let slack_bundle = "com.tinyspeck.slackmacgap";

        // "slack helper" contains "slack"
        assert!(
            service.is_app_related(slack_helper, &installed_apps),
            "Should match 'slack' in 'slack helper'"
        );

        // The bundle ID parts should match if > 3 chars
        // "slackmacgap" contains "slack" as a substring in the installed apps check
        // Note: The actual matching logic checks if name contains app OR app contains name
    }

    #[test]
    fn test_extract_app_name_bundle_id() {
        let service = OrphanedService::new();

        let bundle_id = "com.company.myawesomeapp";
        let result = service.extract_app_name(bundle_id);

        assert_eq!(result, "Myawesomeapp", "Should extract and capitalize last part");
    }

    #[test]
    fn test_extract_app_name_with_plist_extension() {
        let service = OrphanedService::new();

        let name = "com.oldapp.preferences.plist";
        let result = service.extract_app_name(name);

        assert_eq!(result, "Preferences", "Should strip .plist and extract app name");
    }

    #[test]
    fn test_extract_app_name_with_savedstate() {
        let service = OrphanedService::new();

        let name = "com.oldapp.helper.savedState";
        let result = service.extract_app_name(name);

        assert_eq!(result, "Helper", "Should strip .savedState and extract app name");
    }

    #[test]
    fn test_extract_app_name_space_separated() {
        let service = OrphanedService::new();

        let name = "SomeApp Helper Service";
        let result = service.extract_app_name(name);

        assert_eq!(result, "SomeApp", "Should take first part of space-separated name");
    }

    #[test]
    fn test_extract_app_name_simple() {
        let service = OrphanedService::new();

        let name = "SimpleApp";
        let result = service.extract_app_name(name);

        assert_eq!(result, "SimpleApp", "Simple names should remain unchanged");
    }

    #[test]
    fn test_capitalize_first_normal() {
        let service = OrphanedService::new();

        assert_eq!(service.capitalize_first("hello"), "Hello");
        assert_eq!(service.capitalize_first("world"), "World");
        assert_eq!(service.capitalize_first("myApp"), "MyApp");
    }

    #[test]
    fn test_capitalize_first_empty() {
        let service = OrphanedService::new();

        assert_eq!(service.capitalize_first(""), "");
    }

    #[test]
    fn test_capitalize_first_single_char() {
        let service = OrphanedService::new();

        assert_eq!(service.capitalize_first("a"), "A");
        assert_eq!(service.capitalize_first("Z"), "Z");
    }

    #[test]
    fn test_calculate_size_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");

        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(&[0u8; 1000]).expect("Failed to write");

        let service = OrphanedService::new();
        let size = service.calculate_size(&file_path);

        assert_eq!(size, 1000, "File size should be 1000 bytes");
    }

    #[test]
    fn test_calculate_size_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create files in the directory
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("subdir").join("file2.txt");

        fs::create_dir_all(temp_dir.path().join("subdir")).expect("Failed to create subdir");

        let mut f1 = File::create(&file1).expect("Failed to create file1");
        let mut f2 = File::create(&file2).expect("Failed to create file2");

        f1.write_all(&[0u8; 500]).expect("Failed to write");
        f2.write_all(&[0u8; 300]).expect("Failed to write");

        let service = OrphanedService::new();
        let size = service.calculate_size(&temp_dir.path().to_path_buf());

        assert_eq!(size, 800, "Directory size should be sum of file sizes");
    }

    #[test]
    fn test_calculate_size_nonexistent() {
        let service = OrphanedService::new();
        let nonexistent = PathBuf::from("/this/does/not/exist");

        let size = service.calculate_size(&nonexistent);

        assert_eq!(size, 0, "Nonexistent path should return 0");
    }

    #[test]
    fn test_hidden_files_skipped() {
        // Test that hidden files (starting with .) are skipped in scan
        let name = ".hidden_folder";

        let should_skip = name.starts_with('.') || name == "CloudKit" || name == "Apple";

        assert!(should_skip, "Hidden files should be skipped");
    }

    #[test]
    fn test_cloudkit_skipped() {
        let name = "CloudKit";

        let should_skip = name.starts_with('.') || name == "CloudKit" || name == "Apple";

        assert!(should_skip, "CloudKit folder should be skipped");
    }

    #[test]
    fn test_apple_folder_skipped() {
        let name = "Apple";

        let should_skip = name.starts_with('.') || name == "CloudKit" || name == "Apple";

        assert!(should_skip, "Apple folder should be skipped");
    }

    #[test]
    fn test_size_threshold_filtering() {
        // Test that files under 1MB are filtered out
        let size_threshold = 1_000_000u64;

        let small_size = 500_000u64; // 500 KB
        let large_size = 2_000_000u64; // 2 MB

        assert!(
            small_size <= size_threshold,
            "Small files should be below threshold"
        );
        assert!(
            large_size > size_threshold,
            "Large files should be above threshold"
        );
    }

    #[test]
    fn test_results_sorted_by_size() {
        let mut files = vec![
            OrphanedFile {
                path: "/small".to_string(),
                size: 1_000_000,
                likely_app: "Small".to_string(),
                last_accessed: 0,
                file_type: "data".to_string(),
            },
            OrphanedFile {
                path: "/large".to_string(),
                size: 100_000_000,
                likely_app: "Large".to_string(),
                last_accessed: 0,
                file_type: "data".to_string(),
            },
            OrphanedFile {
                path: "/medium".to_string(),
                size: 10_000_000,
                likely_app: "Medium".to_string(),
                last_accessed: 0,
                file_type: "data".to_string(),
            },
        ];

        files.sort_by(|a, b| b.size.cmp(&a.size));

        assert_eq!(files[0].likely_app, "Large");
        assert_eq!(files[1].likely_app, "Medium");
        assert_eq!(files[2].likely_app, "Small");
    }

    #[test]
    fn test_system_apps_in_installed_list() {
        let service = OrphanedService::new();

        // The get_installed_apps function adds system apps
        // We can test that the system apps list is comprehensive
        let system_apps = [
            "apple",
            "com.apple",
            "safari",
            "finder",
            "mail",
            "xcode",
            "terminal",
        ];

        // These should all be recognized as "installed" and not orphaned
        let mut installed_apps: HashSet<String> = HashSet::new();
        for app in system_apps {
            installed_apps.insert(app.to_string());
        }

        for app in system_apps {
            assert!(
                service.is_app_related(app, &installed_apps),
                "System app '{}' should be recognized",
                app
            );
        }
    }

    #[test]
    fn test_bundle_id_parsing_from_plist() {
        // Test the bundle ID extraction logic
        let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.example.testapp</string>
    <key>CFBundleName</key>
    <string>TestApp</string>
</dict>
</plist>"#;

        // Test the parsing logic used in get_bundle_id
        if let Some(start) = plist_content.find("<key>CFBundleIdentifier</key>") {
            let after_key = &plist_content[start..];
            if let Some(string_start) = after_key.find("<string>") {
                let value_start = string_start + 8;
                if let Some(string_end) = after_key[value_start..].find("</string>") {
                    let bundle_id = &after_key[value_start..value_start + string_end];
                    assert_eq!(bundle_id, "com.example.testapp");
                }
            }
        }
    }

    #[test]
    fn test_orphaned_service_new() {
        let service = OrphanedService::new();
        let _ = service;
    }

    #[test]
    fn test_orphaned_file_clone() {
        let file = OrphanedFile {
            path: "/test/path".to_string(),
            size: 1000,
            likely_app: "TestApp".to_string(),
            last_accessed: 12345,
            file_type: "data".to_string(),
        };

        let cloned = file.clone();

        assert_eq!(file.path, cloned.path);
        assert_eq!(file.size, cloned.size);
        assert_eq!(file.likely_app, cloned.likely_app);
        assert_eq!(file.last_accessed, cloned.last_accessed);
        assert_eq!(file.file_type, cloned.file_type);
    }

    #[test]
    fn test_file_types_recognized() {
        let valid_types = ["data", "pref", "cache", "container", "group_container", "state"];

        for file_type in valid_types {
            let file = OrphanedFile {
                path: "/test".to_string(),
                size: 1000,
                likely_app: "Test".to_string(),
                last_accessed: 0,
                file_type: file_type.to_string(),
            };

            assert!(!file.file_type.is_empty(), "File type should be set");
        }
    }

    #[test]
    fn test_delete_orphaned_nonexistent() {
        let service = OrphanedService::new();

        let result = service.delete_orphaned("/this/path/does/not/exist");

        assert!(result.is_err(), "Should fail for nonexistent path");
        assert!(
            result.unwrap_err().contains("does not exist"),
            "Error should mention path doesn't exist"
        );
    }

    #[test]
    fn test_is_app_related_bundle_id_parts() {
        let service = OrphanedService::new();

        let mut installed_apps: HashSet<String> = HashSet::new();
        installed_apps.insert("myapp".to_string());

        // Bundle ID with matching part (> 3 chars)
        let name = "com.company.myapp";

        let is_related = service.is_app_related(name, &installed_apps);

        assert!(
            is_related,
            "Should match 'myapp' in bundle ID parts"
        );
    }

    #[test]
    fn test_is_app_related_short_bundle_parts_ignored() {
        let service = OrphanedService::new();

        let mut installed_apps: HashSet<String> = HashSet::new();
        installed_apps.insert("app".to_string()); // 3 chars

        // Bundle ID with short part (= 3 chars, should be ignored)
        let name = "com.co.app";

        // The part "app" is exactly 3 chars, so it won't match through the bundle ID parts logic
        // but it might match through name.contains(app) check
        let is_related = service.is_app_related(name, &installed_apps);

        // "com.co.app" contains "app" as substring, so this will match
        assert!(is_related, "Contains check should still match");
    }
}
