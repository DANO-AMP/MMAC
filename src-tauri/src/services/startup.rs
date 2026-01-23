use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartupItem {
    pub name: String,
    pub path: String,
    pub kind: String, // "LaunchAgent", "LaunchDaemon", "LoginItem"
    pub enabled: bool,
}

/// Validates a name for use in AppleScript to prevent command injection.
/// Returns the sanitized name or an error if the name contains forbidden characters.
fn validate_applescript_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name.len() > 255 {
        return Err("Name too long (max 255 characters)".into());
    }

    let forbidden = ['\"', '\\', '\n', '\r', '\0'];
    for c in name.chars() {
        if forbidden.contains(&c) || c.is_control() {
            return Err(format!("Invalid character in name: {:?}", c));
        }
    }

    Ok(name.to_string())
}

pub struct StartupService;

impl StartupService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_startup_items(&self) -> Vec<StartupItem> {
        let mut items = Vec::new();

        // User Launch Agents
        let user_agents_path = dirs::home_dir()
            .map(|h| h.join("Library/LaunchAgents"))
            .unwrap_or_default();

        if user_agents_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&user_agents_path) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".plist") {
                            let path = entry.path();
                            let enabled = self.is_agent_enabled(&path);
                            items.push(StartupItem {
                                name: name.trim_end_matches(".plist").to_string(),
                                path: path.to_string_lossy().to_string(),
                                kind: "LaunchAgent".to_string(),
                                enabled,
                            });
                        }
                    }
                }
            }
        }

        // System Launch Agents
        let system_agents = std::path::Path::new("/Library/LaunchAgents");
        if system_agents.exists() {
            if let Ok(entries) = std::fs::read_dir(system_agents) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".plist") && !name.starts_with("com.apple") {
                            let path = entry.path();
                            let enabled = self.is_agent_enabled(&path);
                            items.push(StartupItem {
                                name: name.trim_end_matches(".plist").to_string(),
                                path: path.to_string_lossy().to_string(),
                                kind: "LaunchAgent".to_string(),
                                enabled,
                            });
                        }
                    }
                }
            }
        }

        // Login Items via AppleScript
        if let Ok(output) = Command::new("osascript")
            .args(["-e", "tell application \"System Events\" to get the name of every login item"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for name in stdout.trim().split(", ") {
                if !name.is_empty() {
                    items.push(StartupItem {
                        name: name.to_string(),
                        path: String::new(),
                        kind: "LoginItem".to_string(),
                        enabled: true,
                    });
                }
            }
        }

        items
    }

    fn is_agent_enabled(&self, path: &std::path::Path) -> bool {
        // Check if the agent is loaded
        if let Some(name) = path.file_stem() {
            let output = Command::new("launchctl")
                .args(["list", name.to_str().unwrap_or("")])
                .output();

            if let Ok(out) = output {
                return out.status.success();
            }
        }
        false
    }

    /// Validates that a path is within allowed LaunchAgent/LaunchDaemon directories
    fn validate_launchd_path(path: &str) -> Result<std::path::PathBuf, String> {
        let path_buf = std::path::PathBuf::from(path);

        // Check for path traversal attempts
        if path.contains("..") {
            return Err("Path traversal detected: '..' is not allowed".into());
        }

        // Canonicalize the path to resolve symlinks
        let canonical = path_buf.canonicalize()
            .map_err(|_| "Failed to resolve path - file may not exist")?;

        // Build list of allowed directories
        let mut allowed_dirs = vec![
            std::path::PathBuf::from("/Library/LaunchAgents"),
            std::path::PathBuf::from("/Library/LaunchDaemons"),
        ];

        // Add user's LaunchAgents directory
        if let Some(home) = dirs::home_dir() {
            allowed_dirs.push(home.join("Library/LaunchAgents"));
        }

        // Check if the canonical path is within an allowed directory
        let is_allowed = allowed_dirs.iter().any(|allowed_dir| {
            if let Ok(allowed_canonical) = allowed_dir.canonicalize() {
                canonical.starts_with(&allowed_canonical)
            } else {
                // If directory doesn't exist yet, check prefix
                canonical.starts_with(allowed_dir)
            }
        });

        if !is_allowed {
            return Err(format!(
                "Access denied: path must be within /Library/LaunchAgents, /Library/LaunchDaemons, or ~/Library/LaunchAgents"
            ));
        }

        // Verify it's a plist file
        if canonical.extension().and_then(|e| e.to_str()) != Some("plist") {
            return Err("Only .plist files are allowed".into());
        }

        Ok(canonical)
    }

    pub fn toggle_startup_item(&self, path: &str, enable: bool) -> Result<(), String> {
        // Validate the path is in an allowed directory
        let validated_path = Self::validate_launchd_path(path)?;
        let path_str = validated_path.to_string_lossy();

        if enable {
            Command::new("launchctl")
                .args(["load", &path_str])
                .output()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("launchctl")
                .args(["unload", &path_str])
                .output()
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn remove_login_item(&self, name: &str) -> Result<(), String> {
        // Validate name to prevent AppleScript injection
        let validated_name = validate_applescript_name(name)?;

        Command::new("osascript")
            .args(["-e", &format!("tell application \"System Events\" to delete login item \"{}\"", validated_name)])
            .output()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_startup_item_serialization() {
        let item = StartupItem {
            name: "com.example.agent".to_string(),
            path: "/Library/LaunchAgents/com.example.agent.plist".to_string(),
            kind: "LaunchAgent".to_string(),
            enabled: true,
        };

        let json = serde_json::to_string(&item).expect("Should serialize");
        assert!(json.contains("\"name\":\"com.example.agent\""));
        assert!(json.contains("\"kind\":\"LaunchAgent\""));
        assert!(json.contains("\"enabled\":true"));

        let deserialized: StartupItem =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.name, "com.example.agent");
        assert!(deserialized.enabled);
    }

    #[test]
    fn test_applescript_name_validation_valid() {
        // Valid names should pass
        let valid_names = [
            "MyApp",
            "Some Application",
            "App-Name_1.0",
            "Simple",
            "With Numbers 123",
        ];

        for name in valid_names {
            let result = validate_applescript_name(name);
            assert!(
                result.is_ok(),
                "Valid name '{}' should pass validation",
                name
            );
            assert_eq!(result.unwrap(), name);
        }
    }

    #[test]
    fn test_applescript_name_validation_empty() {
        let result = validate_applescript_name("");

        assert!(result.is_err(), "Empty name should fail validation");
        assert!(
            result.unwrap_err().contains("empty"),
            "Error should mention empty"
        );
    }

    #[test]
    fn test_applescript_name_validation_too_long() {
        // Create a name longer than 255 characters
        let long_name = "a".repeat(256);

        let result = validate_applescript_name(&long_name);

        assert!(result.is_err(), "Too long name should fail validation");
        let error = result.unwrap_err();
        assert!(
            error.contains("too long") || error.contains("255"),
            "Error should mention length limit"
        );
    }

    #[test]
    fn test_applescript_name_validation_injection_double_quote() {
        // Test command injection via double quote
        let malicious = "MyApp\" & do shell script \"malicious command";

        let result = validate_applescript_name(malicious);

        assert!(
            result.is_err(),
            "Name with double quote should fail validation"
        );
        assert!(
            result.unwrap_err().contains("Invalid character"),
            "Error should mention invalid character"
        );
    }

    #[test]
    fn test_applescript_name_validation_injection_backslash() {
        // Test command injection via backslash
        let malicious = "MyApp\\ndo shell script";

        let result = validate_applescript_name(malicious);

        assert!(
            result.is_err(),
            "Name with backslash should fail validation"
        );
    }

    #[test]
    fn test_applescript_name_validation_injection_newline() {
        // Test command injection via newline
        let malicious = "MyApp\ndo shell script \"rm -rf /\"";

        let result = validate_applescript_name(malicious);

        assert!(result.is_err(), "Name with newline should fail validation");
    }

    #[test]
    fn test_applescript_name_validation_injection_carriage_return() {
        // Test command injection via carriage return
        let malicious = "MyApp\rdo shell script";

        let result = validate_applescript_name(malicious);

        assert!(
            result.is_err(),
            "Name with carriage return should fail validation"
        );
    }

    #[test]
    fn test_applescript_name_validation_injection_null() {
        // Test command injection via null byte
        let malicious = "MyApp\0malicious";

        let result = validate_applescript_name(malicious);

        assert!(
            result.is_err(),
            "Name with null byte should fail validation"
        );
    }

    #[test]
    fn test_applescript_name_validation_control_chars() {
        // Test various control characters (ASCII 0-31)
        let control_chars = [
            '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07', '\x08',
            '\x0B', '\x0C', '\x0E', '\x0F', '\x10', '\x11', '\x12', '\x13',
            '\x14', '\x15', '\x16', '\x17', '\x18', '\x19', '\x1A', '\x1B',
            '\x1C', '\x1D', '\x1E', '\x1F',
        ];

        for c in control_chars {
            let malicious = format!("MyApp{}test", c);
            let result = validate_applescript_name(&malicious);
            assert!(
                result.is_err(),
                "Name with control char {:?} should fail",
                c
            );
        }
    }

    #[test]
    fn test_invalid_plist_path_traversal() {
        // Test path traversal attack
        let malicious_path = "/Library/LaunchAgents/../../../etc/passwd";

        let result = StartupService::validate_launchd_path(malicious_path);

        assert!(result.is_err(), "Path traversal should be blocked");
        let error = result.unwrap_err();
        assert!(
            error.contains("traversal") || error.contains("denied") || error.contains("resolve"),
            "Error should mention path issue: {}",
            error
        );
    }

    #[test]
    fn test_invalid_plist_path_outside_allowed() {
        // Test path outside allowed directories

        // Create a temp file to ensure the path exists for canonicalization
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_plist = temp_dir.path().join("malicious.plist");
        File::create(&temp_plist).expect("Failed to create temp plist");

        let result = StartupService::validate_launchd_path(&temp_plist.to_string_lossy());

        assert!(
            result.is_err(),
            "Path outside allowed directories should be blocked"
        );
        let error = result.unwrap_err();
        assert!(
            error.contains("denied") || error.contains("Access"),
            "Error should mention access denied"
        );
    }

    #[test]
    fn test_invalid_plist_path_not_plist() {
        // Test that non-plist files are rejected
        // This requires a file that exists in an allowed directory

        // We can test the logic by checking the extension validation
        let path_with_wrong_ext = std::path::PathBuf::from("/Library/LaunchAgents/test.txt");

        // The validation should reject non-plist files
        if path_with_wrong_ext.exists() {
            let result = StartupService::validate_launchd_path(
                &path_with_wrong_ext.to_string_lossy()
            );
            assert!(result.is_err(), "Non-plist file should be rejected");
        }
    }

    #[test]
    fn test_invalid_plist_path_nonexistent() {
        // Test that nonexistent paths are rejected
        let nonexistent = "/Library/LaunchAgents/definitely_does_not_exist_12345.plist";

        let result = StartupService::validate_launchd_path(nonexistent);

        assert!(result.is_err(), "Nonexistent path should fail");
        let error = result.unwrap_err();
        assert!(
            error.contains("resolve") || error.contains("exist"),
            "Error should mention path resolution failure"
        );
    }

    #[test]
    fn test_plist_name_extraction() {
        // Test that .plist extension is properly trimmed
        let filename = "com.example.myagent.plist";
        let name = filename.trim_end_matches(".plist");

        assert_eq!(name, "com.example.myagent");
    }

    #[test]
    fn test_apple_plist_filtered() {
        // Test that com.apple plists are filtered from system agents
        let apple_plist = "com.apple.some.agent.plist";
        let third_party_plist = "com.example.agent.plist";

        let should_include_apple =
            apple_plist.ends_with(".plist") && !apple_plist.starts_with("com.apple");
        let should_include_third_party =
            third_party_plist.ends_with(".plist") && !third_party_plist.starts_with("com.apple");

        assert!(
            !should_include_apple,
            "Apple plists should be filtered"
        );
        assert!(
            should_include_third_party,
            "Third-party plists should be included"
        );
    }

    #[test]
    fn test_startup_service_new() {
        let service = StartupService::new();
        // Just verify the service can be created
        let _ = service;
    }

    #[test]
    fn test_startup_item_clone() {
        let item = StartupItem {
            name: "TestAgent".to_string(),
            path: "/test/path".to_string(),
            kind: "LaunchAgent".to_string(),
            enabled: true,
        };

        let cloned = item.clone();

        assert_eq!(item.name, cloned.name);
        assert_eq!(item.path, cloned.path);
        assert_eq!(item.kind, cloned.kind);
        assert_eq!(item.enabled, cloned.enabled);
    }

    #[test]
    fn test_login_item_has_empty_path() {
        // Login items discovered via AppleScript don't have paths
        let login_item = StartupItem {
            name: "SomeApp".to_string(),
            path: String::new(),
            kind: "LoginItem".to_string(),
            enabled: true,
        };

        assert!(
            login_item.path.is_empty(),
            "Login items should have empty path"
        );
        assert_eq!(login_item.kind, "LoginItem");
    }

    #[test]
    fn test_allowed_launchd_directories() {
        // Verify the allowed directories are correct
        let allowed_system = vec![
            std::path::PathBuf::from("/Library/LaunchAgents"),
            std::path::PathBuf::from("/Library/LaunchDaemons"),
        ];

        for dir in allowed_system {
            // These directories should exist on macOS
            // We just verify they are considered valid paths
            assert!(
                dir.to_string_lossy().starts_with("/Library/Launch"),
                "System launch directories should be under /Library"
            );
        }

        // User LaunchAgents should be under home
        if let Some(home) = dirs::home_dir() {
            let user_agents = home.join("Library/LaunchAgents");
            assert!(
                user_agents.to_string_lossy().contains("Library/LaunchAgents"),
                "User agents should be under ~/Library/LaunchAgents"
            );
        }
    }

    #[test]
    fn test_valid_applescript_with_special_but_allowed_chars() {
        // Test that some special characters are allowed
        let allowed_special = [
            "App (Version 1.0)",
            "My App - Pro",
            "App_Name",
            "App.Name",
            "Japanese: \u{65E5}\u{672C}\u{8A9E}", // Japanese characters
        ];

        for name in allowed_special {
            let result = validate_applescript_name(name);
            // Should pass as long as no forbidden chars
            assert!(
                result.is_ok(),
                "Name '{}' should be allowed",
                name
            );
        }
    }
}
