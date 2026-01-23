use crate::services::cleaner::{CleaningService, ScanResult};
use tauri::command;

/// Whitelist of allowed cleaning categories to prevent arbitrary input
const ALLOWED_CATEGORIES: &[&str] = &[
    "cache",
    "logs",
    "browser",
    "trash",
    "crash_reports",
    "xcode",
    "package_caches",
];

/// Validates that a category is in the allowed whitelist
fn validate_category(category: &str) -> Result<&str, String> {
    if ALLOWED_CATEGORIES.contains(&category) {
        Ok(category)
    } else {
        Err(format!(
            "Invalid category '{}'. Allowed categories: {}",
            category,
            ALLOWED_CATEGORIES.join(", ")
        ))
    }
}

#[command]
pub async fn scan_system() -> Result<Vec<ScanResult>, String> {
    let service = CleaningService::new();
    service.scan_all().await.map_err(|e| e.to_string())
}

#[command]
pub async fn clean_category(category: String) -> Result<u64, String> {
    // Validate category against whitelist before processing
    let validated_category = validate_category(&category)?;

    let service = CleaningService::new();
    service
        .clean_category(validated_category)
        .await
        .map_err(|e| e.to_string())
}
