//! Simple database functions for the desktop app

use rusqlite::{params, Connection};
use std::path::PathBuf;
use tracing::{debug, error, info};

/// Get the database path
fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".hive").join("hive-ai.db")
}

/// Save a configuration value
pub fn save_config(key: &str, value: &str) -> Result<(), String> {
    let db_path = get_db_path();
    info!("Saving config key '{}' to database at: {:?}", key, db_path);

    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            error!("Failed to create .hive directory: {}", e);
            format!("Failed to create .hive directory: {}", e)
        })?;
    }

    // Open or create the database
    let conn = Connection::open(&db_path).map_err(|e| {
        error!("Failed to open database at {:?}: {}", db_path, e);
        format!("Failed to open database: {}", e)
    })?;

    // Create configurations table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS configurations (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| {
        error!("Failed to create configurations table: {}", e);
        format!("Failed to create configurations table: {}", e)
    })?;

    // Insert or update the configuration
    let rows_affected = conn
        .execute(
            "INSERT OR REPLACE INTO configurations (key, value, created_at, updated_at)
         VALUES (?1, ?2, datetime('now'), datetime('now'))",
            params![key, value],
        )
        .map_err(|e| {
            error!("Failed to save configuration key '{}': {}", key, e);
            format!("Failed to save configuration: {}", e)
        })?;

    info!(
        "Successfully saved config key '{}' ({} rows affected)",
        key, rows_affected
    );
    Ok(())
}

/// Get a configuration value
pub fn get_config(key: &str) -> Result<Option<String>, String> {
    let db_path = get_db_path();
    debug!(
        "Getting config key '{}' from database at: {:?}",
        key, db_path
    );

    // Check if database exists
    if !db_path.exists() {
        debug!("Database does not exist yet");
        return Ok(None);
    }

    let conn = Connection::open(&db_path).map_err(|e| {
        error!("Failed to open database: {}", e);
        format!("Failed to open database: {}", e)
    })?;

    // Check if table exists
    let table_exists: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='configurations'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if table_exists == 0 {
        debug!("Configurations table does not exist yet");
        return Ok(None);
    }

    let mut stmt = conn
        .prepare("SELECT value FROM configurations WHERE key = ?1")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let result = match stmt.query_row(params![key], |row| row.get::<_, String>(0)) {
        Ok(value) => {
            debug!("Found value for key '{}': {} chars", key, value.len());
            Some(value)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            debug!("No value found for key '{}'", key);
            None
        }
        Err(e) => {
            error!("Error querying key '{}': {}", key, e);
            return Err(e.to_string());
        }
    };

    Ok(result)
}

/// Mark onboarding as complete
pub fn mark_onboarding_complete() -> Result<(), String> {
    save_config("onboarding_completed", "true")
}

/// Check if OpenRouter key exists
pub fn has_openrouter_key() -> bool {
    match get_config("openrouter_api_key") {
        Ok(Some(key)) => !key.is_empty(),
        _ => false,
    }
}

/// Check if Hive license key exists
pub fn has_hive_key() -> bool {
    match get_config("hive_license_key") {
        Ok(Some(key)) => !key.is_empty(),
        _ => false,
    }
}

/// Check if onboarding is completed
pub fn is_onboarding_completed() -> bool {
    match get_config("onboarding_completed") {
        Ok(Some(value)) => value == "true",
        _ => false,
    }
}
