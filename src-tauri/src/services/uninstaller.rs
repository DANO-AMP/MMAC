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

    /// Check if a path has the .app extension. Extracted for testability.
    #[cfg(test)]
    fn is_app_bundle(path: &PathBuf) -> bool {
        path.extension().map(|e| e == "app").unwrap_or(false)
    }

    /// Generate patterns used for remnant matching from bundle_id and app_name.
    #[cfg(test)]
    fn remnant_patterns(bundle_id: &str, app_name: &str) -> Vec<String> {
        vec![
            bundle_id.to_string(),
            app_name.to_lowercase().replace(' ', ""),
            app_name.to_lowercase().replace(' ', "."),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ---- Struct serialization tests ----

    #[test]
    fn test_remnant_file_serialization_roundtrip() {
        let remnant = RemnantFile {
            path: "/Users/me/Library/Caches/com.example.app".to_string(),
            size: 50_000_000,
            remnant_type: "cache".to_string(),
        };

        let json = serde_json::to_string(&remnant).expect("serialize");
        let deserialized: RemnantFile = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.path, "/Users/me/Library/Caches/com.example.app");
        assert_eq!(deserialized.size, 50_000_000);
        // Note: the field is renamed to "type" in JSON
        assert_eq!(deserialized.remnant_type, "cache");
    }

    #[test]
    fn test_remnant_file_json_uses_type_key() {
        let remnant = RemnantFile {
            path: "/test".to_string(),
            size: 100,
            remnant_type: "pref".to_string(),
        };

        let json_value: serde_json::Value = serde_json::to_value(&remnant).expect("to value");
        // Verify the serde rename is applied
        assert_eq!(json_value.get("type").and_then(|v| v.as_str()), Some("pref"));
        assert!(json_value.get("remnant_type").is_none(), "Should not have remnant_type key in JSON");
    }

    #[test]
    fn test_app_info_serialization_roundtrip() {
        let app = AppInfo {
            name: "Visual Studio Code".to_string(),
            bundle_id: "com.microsoft.VSCode".to_string(),
            path: "/Applications/Visual Studio Code.app".to_string(),
            size: 500_000_000,
            version: Some("1.85.0".to_string()),
            remnants: vec![RemnantFile {
                path: "/Users/me/Library/Caches/com.microsoft.VSCode".to_string(),
                size: 100_000,
                remnant_type: "cache".to_string(),
            }],
            remnants_size: 100_000,
        };

        let json = serde_json::to_string(&app).expect("serialize");
        let deserialized: AppInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, "Visual Studio Code");
        assert_eq!(deserialized.bundle_id, "com.microsoft.VSCode");
        assert_eq!(deserialized.path, "/Applications/Visual Studio Code.app");
        assert_eq!(deserialized.size, 500_000_000);
        assert_eq!(deserialized.version, Some("1.85.0".to_string()));
        assert_eq!(deserialized.remnants.len(), 1);
        assert_eq!(deserialized.remnants_size, 100_000);
    }

    #[test]
    fn test_app_info_serialization_with_none_version() {
        let app = AppInfo {
            name: "SomeApp".to_string(),
            bundle_id: "com.example.someapp".to_string(),
            path: "/Applications/SomeApp.app".to_string(),
            size: 1000,
            version: None,
            remnants: vec![],
            remnants_size: 0,
        };

        let json = serde_json::to_string(&app).expect("serialize");
        let deserialized: AppInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.version, None);
        assert!(deserialized.remnants.is_empty());
        assert_eq!(deserialized.remnants_size, 0);
    }

    // ---- is_app_bundle tests ----

    #[test]
    fn test_is_app_bundle_with_app_extension() {
        let path = PathBuf::from("/Applications/Safari.app");
        assert!(UninstallerService::is_app_bundle(&path));
    }

    #[test]
    fn test_is_app_bundle_without_extension() {
        let path = PathBuf::from("/Applications/Safari");
        assert!(!UninstallerService::is_app_bundle(&path));
    }

    #[test]
    fn test_is_app_bundle_with_wrong_extension() {
        let path = PathBuf::from("/tmp/file.txt");
        assert!(!UninstallerService::is_app_bundle(&path));
    }

    #[test]
    fn test_is_app_bundle_with_dmg_extension() {
        let path = PathBuf::from("/tmp/installer.dmg");
        assert!(!UninstallerService::is_app_bundle(&path));
    }

    #[test]
    fn test_is_app_bundle_with_app_in_name_but_not_extension() {
        let path = PathBuf::from("/Applications/myapp/binary");
        assert!(!UninstallerService::is_app_bundle(&path));
    }

    #[test]
    fn test_is_app_bundle_spaces_in_name() {
        let path = PathBuf::from("/Applications/Visual Studio Code.app");
        assert!(UninstallerService::is_app_bundle(&path));
    }

    // ---- remnant_patterns tests ----

    #[test]
    fn test_remnant_patterns_basic() {
        let patterns = UninstallerService::remnant_patterns("com.example.myapp", "My App");

        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0], "com.example.myapp");
        assert_eq!(patterns[1], "myapp"); // lowercase, spaces removed
        assert_eq!(patterns[2], "my.app"); // lowercase, spaces replaced with dots
    }

    #[test]
    fn test_remnant_patterns_no_spaces() {
        let patterns = UninstallerService::remnant_patterns("com.apple.safari", "Safari");

        assert_eq!(patterns[0], "com.apple.safari");
        assert_eq!(patterns[1], "safari");
        assert_eq!(patterns[2], "safari");
    }

    #[test]
    fn test_remnant_patterns_multiple_spaces() {
        let patterns = UninstallerService::remnant_patterns(
            "com.example.app",
            "My Great App",
        );

        assert_eq!(patterns[1], "mygreatapp");
        assert_eq!(patterns[2], "my.great.app");
    }

    // ---- calculate_dir_size tests ----

    #[test]
    fn test_calculate_dir_size_single_file() {
        let dir = TempDir::new().expect("create tempdir");
        std::fs::write(dir.path().join("file.bin"), &vec![0u8; 2048]).expect("write");

        let service = UninstallerService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 2048);
    }

    #[test]
    fn test_calculate_dir_size_nested() {
        let dir = TempDir::new().expect("create tempdir");
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).expect("mkdir");

        std::fs::write(dir.path().join("a.txt"), &vec![0u8; 100]).expect("write a");
        std::fs::write(sub.join("b.txt"), &vec![0u8; 200]).expect("write b");

        let service = UninstallerService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 300);
    }

    #[test]
    fn test_calculate_dir_size_empty() {
        let dir = TempDir::new().expect("create tempdir");

        let service = UninstallerService::new();
        let size = service.calculate_dir_size(&dir.path().to_path_buf());

        assert_eq!(size, 0);
    }

    // ---- find_remnants tests with tempdir ----

    #[test]
    fn test_find_remnants_finds_matching_files() {
        let home = TempDir::new().expect("create tempdir");
        let home_path = home.path().to_path_buf();

        // Create Library/Caches directory with a matching entry
        let caches_dir = home_path.join("Library/Caches");
        std::fs::create_dir_all(&caches_dir).expect("create caches dir");
        std::fs::write(caches_dir.join("com.example.testapp"), b"cache data").expect("write cache");

        // Create Library/Preferences directory with a matching plist
        let prefs_dir = home_path.join("Library/Preferences");
        std::fs::create_dir_all(&prefs_dir).expect("create prefs dir");
        std::fs::write(
            prefs_dir.join("com.example.testapp.plist"),
            b"plist data",
        ).expect("write plist");

        let service = UninstallerService::new();
        let remnants = service.find_remnants("com.example.testapp", "TestApp", &home_path);

        assert!(!remnants.is_empty(), "Should find at least one remnant");

        // Should find the cache entry
        let cache_remnant = remnants.iter().find(|r| r.remnant_type == "cache");
        assert!(cache_remnant.is_some(), "Should find cache remnant");

        // Should find the plist entry
        let pref_remnant = remnants.iter().find(|r| r.remnant_type == "pref");
        assert!(pref_remnant.is_some(), "Should find pref remnant");
    }

    #[test]
    fn test_find_remnants_no_matches() {
        let home = TempDir::new().expect("create tempdir");
        let home_path = home.path().to_path_buf();

        // Create Library/Caches with unrelated entries
        let caches_dir = home_path.join("Library/Caches");
        std::fs::create_dir_all(&caches_dir).expect("create caches dir");
        std::fs::write(caches_dir.join("com.other.app"), b"data").expect("write");

        let service = UninstallerService::new();
        let remnants = service.find_remnants("com.example.testapp", "TestApp", &home_path);

        assert!(remnants.is_empty(), "Should find no remnants for unrelated bundle");
    }

    #[test]
    fn test_find_remnants_matches_by_app_name() {
        let home = TempDir::new().expect("create tempdir");
        let home_path = home.path().to_path_buf();

        // Create Application Support directory with a name-based match
        let support_dir = home_path.join("Library/Application Support");
        std::fs::create_dir_all(&support_dir).expect("create support dir");
        let app_data = support_dir.join("testapp");
        std::fs::create_dir(&app_data).expect("create app data dir");
        std::fs::write(app_data.join("data.db"), b"database content").expect("write data");

        let service = UninstallerService::new();
        let remnants = service.find_remnants("com.example.notfound", "TestApp", &home_path);

        let data_remnant = remnants.iter().find(|r| r.remnant_type == "data");
        assert!(data_remnant.is_some(), "Should find remnant by app name match");
    }

    #[test]
    fn test_find_remnants_handles_missing_library_dirs() {
        let home = TempDir::new().expect("create tempdir");
        let home_path = home.path().to_path_buf();
        // No Library directories created

        let service = UninstallerService::new();
        let remnants = service.find_remnants("com.example.app", "App", &home_path);

        assert!(remnants.is_empty(), "Should gracefully handle missing directories");
    }

    #[test]
    fn test_find_remnants_remnant_size_for_directory() {
        let home = TempDir::new().expect("create tempdir");
        let home_path = home.path().to_path_buf();

        let caches_dir = home_path.join("Library/Caches");
        std::fs::create_dir_all(&caches_dir).expect("create caches dir");

        // Create a directory remnant with files inside
        let cache_app_dir = caches_dir.join("com.example.sizetest");
        std::fs::create_dir(&cache_app_dir).expect("create cache app dir");
        std::fs::write(cache_app_dir.join("file1.dat"), &vec![0u8; 500]).expect("write");
        std::fs::write(cache_app_dir.join("file2.dat"), &vec![0u8; 300]).expect("write");

        let service = UninstallerService::new();
        let remnants = service.find_remnants("com.example.sizetest", "SizeTest", &home_path);

        let cache_remnant = remnants.iter().find(|r| r.remnant_type == "cache");
        assert!(cache_remnant.is_some());
        assert_eq!(cache_remnant.unwrap().size, 800, "Size should be sum of files in directory");
    }

    // ---- Search location coverage test ----

    #[test]
    fn test_find_remnants_checks_all_user_library_locations() {
        let home = TempDir::new().expect("create tempdir");
        let home_path = home.path().to_path_buf();

        let locations = vec![
            ("Library/Application Support", "data"),
            ("Library/Caches", "cache"),
            ("Library/Containers", "container"),
            ("Library/Saved Application State", "state"),
            ("Library/Cookies", "cookies"),
            ("Library/LaunchAgents", "agent"),
            ("Library/HTTPStorages", "storage"),
            ("Library/WebKit", "webkit"),
            ("Library/Logs", "logs"),
        ];

        for (dir, _) in &locations {
            let full_path = home_path.join(dir);
            std::fs::create_dir_all(&full_path).expect("create dir");
            std::fs::write(
                full_path.join("com.example.locationtest"),
                b"data",
            ).expect("write");
        }

        let service = UninstallerService::new();
        let remnants = service.find_remnants("com.example.locationtest", "LocationTest", &home_path);

        // Should find one remnant per location
        assert!(
            remnants.len() >= locations.len(),
            "Expected at least {} remnants, found {}",
            locations.len(),
            remnants.len()
        );
    }
}
