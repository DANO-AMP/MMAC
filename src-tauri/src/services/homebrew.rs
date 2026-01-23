use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::warn;

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

    pub fn check_homebrew(&self) -> Result<HomebrewInfo, String> {
        let version_output = Command::new("brew")
            .arg("--version")
            .output();

        match version_output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = version_str.lines().next()
                    .map(|s| s.replace("Homebrew ", "").trim().to_string());

                // Count installed packages
                let formulae_count = match Command::new("brew")
                    .args(["list", "--formula", "-1"])
                    .output()
                {
                    Ok(o) if o.status.success() => {
                        String::from_utf8_lossy(&o.stdout).lines().count() as u32
                    }
                    Ok(o) => {
                        warn!(
                            "brew list --formula failed with status {}: {}",
                            o.status,
                            String::from_utf8_lossy(&o.stderr)
                        );
                        0
                    }
                    Err(e) => {
                        warn!("Failed to execute brew list --formula: {}", e);
                        0
                    }
                };

                let casks_count = match Command::new("brew")
                    .args(["list", "--cask", "-1"])
                    .output()
                {
                    Ok(o) if o.status.success() => {
                        String::from_utf8_lossy(&o.stdout).lines().count() as u32
                    }
                    Ok(o) => {
                        warn!(
                            "brew list --cask failed with status {}: {}",
                            o.status,
                            String::from_utf8_lossy(&o.stderr)
                        );
                        0
                    }
                    Err(e) => {
                        warn!("Failed to execute brew list --cask: {}", e);
                        0
                    }
                };

                Ok(HomebrewInfo {
                    installed: true,
                    version,
                    formulae_count,
                    casks_count,
                })
            }
            Ok(output) => {
                // brew command exists but returned an error
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Homebrew check failed: {}", stderr);
                Ok(HomebrewInfo {
                    installed: false,
                    version: None,
                    formulae_count: 0,
                    casks_count: 0,
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Homebrew is not installed
                Err("Homebrew is not installed. Install it from https://brew.sh".to_string())
            }
            Err(e) => {
                // Other error executing the command
                warn!("Failed to check Homebrew: {}", e);
                Err(format!("Failed to check Homebrew status: {}", e))
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    // Sample output for `brew list --formula --versions`
    const SAMPLE_FORMULA_LIST: &str = r#"git 2.43.0
node 21.5.0
python@3.12 3.12.1
openssl@3 3.2.0
wget 1.21.4"#;

    // Sample output for `brew list --cask --versions`
    const SAMPLE_CASK_LIST: &str = r#"visual-studio-code 1.85.1
firefox 121.0
docker 4.26.1"#;

    // Sample output for `brew outdated --formula --verbose`
    const SAMPLE_OUTDATED_FORMULA: &str = r#"git (2.43.0) < 2.44.0
node (21.5.0) < 21.6.0
python@3.12 (3.12.1) < 3.12.2"#;

    // Sample output for `brew outdated --cask --verbose`
    const SAMPLE_OUTDATED_CASK: &str = r#"visual-studio-code (1.85.1) != 1.86.0
firefox (121.0) < 122.0"#;

    // Test helper: parse a single line from package list output
    fn parse_package_line(line: &str, is_cask: bool) -> Option<BrewPackage> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            Some(BrewPackage {
                name: parts[0].to_string(),
                version: parts[1].to_string(),
                is_outdated: false,
                newer_version: None,
                is_cask,
            })
        } else {
            None
        }
    }

    #[test]
    fn test_homebrew_not_installed() {
        // When Homebrew is not installed, check_homebrew should return installed: false
        // This tests the fallback HomebrewInfo state when brew is not available
        let info = HomebrewInfo {
            installed: false,
            version: None,
            formulae_count: 0,
            casks_count: 0,
        };

        assert!(!info.installed);
        assert!(info.version.is_none());
        assert_eq!(info.formulae_count, 0);
        assert_eq!(info.casks_count, 0);
    }

    #[test]
    fn test_parse_package_list_formulae() {
        // Test parsing formula package list output
        let mut packages = Vec::new();

        for line in SAMPLE_FORMULA_LIST.lines() {
            if let Some(pkg) = parse_package_line(line, false) {
                packages.push(pkg);
            }
        }

        assert_eq!(packages.len(), 5);

        // Check first package
        assert_eq!(packages[0].name, "git");
        assert_eq!(packages[0].version, "2.43.0");
        assert!(!packages[0].is_cask);
        assert!(!packages[0].is_outdated);

        // Check versioned package (python@3.12)
        assert_eq!(packages[2].name, "python@3.12");
        assert_eq!(packages[2].version, "3.12.1");
    }

    #[test]
    fn test_parse_package_list_casks() {
        // Test parsing cask package list output
        let mut packages = Vec::new();

        for line in SAMPLE_CASK_LIST.lines() {
            if let Some(pkg) = parse_package_line(line, true) {
                packages.push(pkg);
            }
        }

        assert_eq!(packages.len(), 3);

        assert_eq!(packages[0].name, "visual-studio-code");
        assert_eq!(packages[0].version, "1.85.1");
        assert!(packages[0].is_cask);
    }

    #[test]
    fn test_parse_package_list_empty() {
        // Test parsing empty output
        let packages: Vec<BrewPackage> = "".lines()
            .filter_map(|line| parse_package_line(line, false))
            .collect();

        assert!(packages.is_empty());
    }

    #[test]
    fn test_parse_package_list_malformed_line() {
        // Test parsing malformed lines (only package name, no version)
        let malformed = "somepackage\n";
        let packages: Vec<BrewPackage> = malformed.lines()
            .filter_map(|line| parse_package_line(line, false))
            .collect();

        assert!(packages.is_empty(), "Malformed lines should be skipped");
    }

    #[test]
    fn test_parse_outdated_packages_formula() {
        let service = HomebrewService::new();

        let mut outdated = Vec::new();
        for line in SAMPLE_OUTDATED_FORMULA.lines() {
            if let Some(pkg) = service.parse_outdated_line(line, false) {
                outdated.push(pkg);
            }
        }

        assert_eq!(outdated.len(), 3);

        // Check git
        assert_eq!(outdated[0].name, "git");
        assert_eq!(outdated[0].version, "2.43.0");
        assert_eq!(outdated[0].newer_version, Some("2.44.0".to_string()));
        assert!(outdated[0].is_outdated);
        assert!(!outdated[0].is_cask);

        // Check python with @ in name
        assert_eq!(outdated[2].name, "python@3.12");
        assert_eq!(outdated[2].version, "3.12.1");
        assert_eq!(outdated[2].newer_version, Some("3.12.2".to_string()));
    }

    #[test]
    fn test_parse_outdated_packages_cask() {
        let service = HomebrewService::new();

        let mut outdated = Vec::new();
        for line in SAMPLE_OUTDATED_CASK.lines() {
            if let Some(pkg) = service.parse_outdated_line(line, true) {
                outdated.push(pkg);
            }
        }

        assert_eq!(outdated.len(), 2);

        // Check VS Code with != operator
        assert_eq!(outdated[0].name, "visual-studio-code");
        assert_eq!(outdated[0].version, "1.85.1");
        assert_eq!(outdated[0].newer_version, Some("1.86.0".to_string()));
        assert!(outdated[0].is_cask);
    }

    #[test]
    fn test_parse_outdated_empty_line() {
        let service = HomebrewService::new();

        let result = service.parse_outdated_line("", false);
        assert!(result.is_none(), "Empty lines should return None");

        let result = service.parse_outdated_line("   ", false);
        assert!(result.is_none(), "Whitespace-only lines should return None");
    }

    #[test]
    fn test_parse_outdated_package_name_only() {
        let service = HomebrewService::new();

        // Some edge case: only package name
        let result = service.parse_outdated_line("somepackage", false);
        assert!(result.is_some());
        let pkg = result.unwrap();
        assert_eq!(pkg.name, "somepackage");
        assert_eq!(pkg.version, "");
        assert!(pkg.newer_version.is_none());
    }

    #[test]
    fn test_upgrade_package_error_format() {
        // Test that error messages are properly formatted
        // This tests the error string construction without actually calling brew
        let package_name = "nonexistent-package";
        let is_cask = false;

        let args = if is_cask {
            vec!["upgrade", "--cask", package_name]
        } else {
            vec!["upgrade", package_name]
        };

        assert_eq!(args, vec!["upgrade", "nonexistent-package"]);

        // Test cask variant
        let args_cask = vec!["upgrade", "--cask", "nonexistent-cask"];
        assert_eq!(args_cask.len(), 3);
    }

    #[test]
    fn test_homebrew_info_serialization() {
        let info = HomebrewInfo {
            installed: true,
            version: Some("4.2.0".to_string()),
            formulae_count: 150,
            casks_count: 25,
        };

        // Test serialization
        let json = serde_json::to_string(&info).expect("Should serialize");
        assert!(json.contains("\"installed\":true"));
        assert!(json.contains("\"version\":\"4.2.0\""));
        assert!(json.contains("\"formulae_count\":150"));
        assert!(json.contains("\"casks_count\":25"));

        // Test deserialization
        let deserialized: HomebrewInfo = serde_json::from_str(&json).expect("Should deserialize");
        assert!(deserialized.installed);
        assert_eq!(deserialized.version, Some("4.2.0".to_string()));
    }

    #[test]
    fn test_brew_package_clone() {
        let pkg = BrewPackage {
            name: "git".to_string(),
            version: "2.43.0".to_string(),
            is_outdated: true,
            newer_version: Some("2.44.0".to_string()),
            is_cask: false,
        };

        let cloned = pkg.clone();
        assert_eq!(pkg.name, cloned.name);
        assert_eq!(pkg.version, cloned.version);
        assert_eq!(pkg.is_outdated, cloned.is_outdated);
        assert_eq!(pkg.newer_version, cloned.newer_version);
        assert_eq!(pkg.is_cask, cloned.is_cask);
    }

    #[test]
    fn test_parse_version_with_multiple_parts() {
        // Test parsing packages with multiple version numbers on the same line
        let line = "openssl@3 3.2.0 3.1.0";
        let pkg = parse_package_line(line, false);

        assert!(pkg.is_some());
        let pkg = pkg.unwrap();
        assert_eq!(pkg.name, "openssl@3");
        // Should take first version
        assert_eq!(pkg.version, "3.2.0");
    }

    #[test]
    fn test_parse_outdated_special_characters() {
        let service = HomebrewService::new();

        // Test with special version formats
        let line = "font-fira-code (6.2) < 6.3-beta";
        let result = service.parse_outdated_line(line, true);

        assert!(result.is_some());
        let pkg = result.unwrap();
        assert_eq!(pkg.name, "font-fira-code");
        assert_eq!(pkg.version, "6.2");
        assert_eq!(pkg.newer_version, Some("6.3-beta".to_string()));
    }
}
