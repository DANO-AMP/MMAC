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

    pub fn toggle_startup_item(&self, path: &str, enable: bool) -> Result<(), String> {
        if enable {
            Command::new("launchctl")
                .args(["load", path])
                .output()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("launchctl")
                .args(["unload", path])
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
