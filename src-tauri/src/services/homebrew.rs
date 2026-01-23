use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrewPackage {
    pub name: String,
    pub version: String,
    pub is_outdated: bool,
    pub newer_version: Option<String>,
    pub is_cask: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HomebrewInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub formulae_count: u32,
    pub casks_count: u32,
}

pub struct HomebrewService;

impl HomebrewService {
    pub fn new() -> Self {
        Self
    }

    pub fn check_homebrew(&self) -> HomebrewInfo {
        let version_output = Command::new("brew")
            .arg("--version")
            .output();

        match version_output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.lines().next()
                    .map(|s| s.replace("Homebrew ", "").trim().to_string());

                // Count installed packages
                let formulae_count = Command::new("brew")
                    .args(["list", "--formula", "-1"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as u32)
                    .unwrap_or(0);

                let casks_count = Command::new("brew")
                    .args(["list", "--cask", "-1"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as u32)
                    .unwrap_or(0);

                HomebrewInfo {
                    installed: true,
                    version,
                    formulae_count,
                    casks_count,
                }
            }
            _ => HomebrewInfo {
                installed: false,
                version: None,
                formulae_count: 0,
                casks_count: 0,
            },
        }
    }

    pub fn list_packages(&self) -> Result<Vec<BrewPackage>, String> {
        let mut packages = Vec::new();

        // Get formulae with versions
        let formula_output = Command::new("brew")
            .args(["list", "--formula", "--versions"])
            .output()
            .map_err(|e| format!("Failed to list formulae: {}", e))?;

        if formula_output.status.success() {
            let output_str = String::from_utf8_lossy(&formula_output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    packages.push(BrewPackage {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        is_outdated: false,
                        newer_version: None,
                        is_cask: false,
                    });
                }
            }
        }

        // Get casks with versions
        let cask_output = Command::new("brew")
            .args(["list", "--cask", "--versions"])
            .output()
            .map_err(|e| format!("Failed to list casks: {}", e))?;

        if cask_output.status.success() {
            let output_str = String::from_utf8_lossy(&cask_output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    packages.push(BrewPackage {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        is_outdated: false,
                        newer_version: None,
                        is_cask: true,
                    });
                }
            }
        }

        Ok(packages)
    }

    pub fn get_outdated(&self) -> Result<Vec<BrewPackage>, String> {
        let mut outdated = Vec::new();

        // Get outdated formulae
        let formula_output = Command::new("brew")
            .args(["outdated", "--formula", "--verbose"])
            .output()
            .map_err(|e| format!("Failed to check outdated formulae: {}", e))?;

        if formula_output.status.success() {
            let output_str = String::from_utf8_lossy(&formula_output.stdout);
            for line in output_str.lines() {
                if let Some(pkg) = self.parse_outdated_line(line, false) {
                    outdated.push(pkg);
                }
            }
        }

        // Get outdated casks
        let cask_output = Command::new("brew")
            .args(["outdated", "--cask", "--verbose"])
            .output()
            .map_err(|e| format!("Failed to check outdated casks: {}", e))?;

        if cask_output.status.success() {
            let output_str = String::from_utf8_lossy(&cask_output.stdout);
            for line in output_str.lines() {
                if let Some(pkg) = self.parse_outdated_line(line, true) {
                    outdated.push(pkg);
                }
            }
        }

        Ok(outdated)
    }

    fn parse_outdated_line(&self, line: &str, is_cask: bool) -> Option<BrewPackage> {
        // Format: "package (current_version) < newer_version" or "package (current_version) != newer_version"
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let name = parts[0].to_string();
        let mut current_version = String::new();
        let mut newer_version = None;

        // Parse version info
        if parts.len() >= 2 {
            current_version = parts[1].trim_matches(|c| c == '(' || c == ')').to_string();
        }
        if parts.len() >= 4 {
            newer_version = Some(parts[3].to_string());
        }

        Some(BrewPackage {
            name,
            version: current_version,
            is_outdated: true,
            newer_version,
            is_cask,
        })
    }

    pub fn upgrade_package(&self, name: &str, is_cask: bool) -> Result<String, String> {
        let args = if is_cask {
            vec!["upgrade", "--cask", name]
        } else {
            vec!["upgrade", name]
        };

        let output = Command::new("brew")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to upgrade {}: {}", name, e))?;

        if output.status.success() {
            Ok(format!("Successfully upgraded {}", name))
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn upgrade_all(&self) -> Result<String, String> {
        let output = Command::new("brew")
            .args(["upgrade"])
            .output()
            .map_err(|e| format!("Failed to upgrade all: {}", e))?;

        if output.status.success() {
            Ok("Successfully upgraded all packages".to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn uninstall_package(&self, name: &str, is_cask: bool) -> Result<String, String> {
        let args = if is_cask {
            vec!["uninstall", "--cask", name]
        } else {
            vec!["uninstall", name]
        };

        let output = Command::new("brew")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to uninstall {}: {}", name, e))?;

        if output.status.success() {
            Ok(format!("Successfully uninstalled {}", name))
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn cleanup(&self) -> Result<String, String> {
        let output = Command::new("brew")
            .args(["cleanup", "--prune=all"])
            .output()
            .map_err(|e| format!("Failed to cleanup: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        if output.status.success() {
            Ok(if stdout.is_empty() {
                "Nothing to clean up".to_string()
            } else {
                stdout
            })
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}
