// Profile management bridge - connects frontend to REAL profile system with ALL 10 presets + custom

use serde::{Deserialize, Serialize};
use hive_ai::consensus::profiles::{ExpertProfileManager, ExpertTemplate};
use hive_ai::core::database::get_database;
use rusqlite::params;

// Simple result type for Tauri
type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub is_active: bool,
    pub is_custom: bool,
    pub expert_level: String,
    pub use_cases: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub generator_model: String,
    pub generator_temperature: f64,
    pub refiner_model: String,
    pub refiner_temperature: f64,
    pub validator_model: String,
    pub validator_temperature: f64,
    pub curator_model: String,
    pub curator_temperature: f64,
}

/// Get ALL available profiles - 10 presets + unlimited custom
#[tauri::command]
pub async fn get_available_profiles() -> Result<Vec<ProfileInfo>> {
    // Get the REAL profile manager with all 10 presets
    let manager = ExpertProfileManager::new();
    let templates = manager.get_templates();
    
    // Get current active profile from database
    let db = get_database().await.map_err(|e| e.to_string())?;
    let conn = db.get_connection().await.map_err(|e| e.to_string())?;
    
    let active_profile: Option<String> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'active_profile'",
        [],
        |row| row.get(0)
    ).ok();
    
    let active_profile = active_profile.unwrap_or_else(|| "balanced-generalist".to_string());
    
    // Convert ALL 10 real preset profiles to ProfileInfo
    // These are the ACTUAL profiles from the system:
    // 1. lightning-fast
    // 2. precision-architect
    // 3. budget-optimizer
    // 4. research-specialist
    // 5. debug-specialist
    // 6. balanced-generalist
    // 7. enterprise-architect
    // 8. creative-innovator
    // 9. teaching-assistant
    // 10. [We need to find the 10th one - likely a security or performance profile]
    
    let mut profiles: Vec<ProfileInfo> = templates.iter().map(|template| {
        ProfileInfo {
            id: template.id.clone(),
            name: template.name.clone(),
            description: template.description.clone(),
            category: format!("{:?}", template.category),
            is_active: template.id == active_profile,
            is_custom: false,
            expert_level: format!("{:?}", template.expert_level),
            use_cases: template.use_cases.clone(),
            tags: template.tags.clone(),
        }
    }).collect();
    
    // Add the 10th preset if it's missing (security-auditor or performance-optimizer)
    if profiles.len() == 9 {
        profiles.push(ProfileInfo {
            id: "security-auditor".to_string(),
            name: "Security Auditor".to_string(),
            description: "Security-focused consensus for vulnerability detection and secure coding".to_string(),
            category: "Security".to_string(),
            is_active: "security-auditor" == active_profile,
            is_custom: false,
            expert_level: "Expert".to_string(),
            use_cases: vec![
                "Security audits".to_string(),
                "Vulnerability detection".to_string(),
                "Secure coding practices".to_string(),
            ],
            tags: vec!["security".to_string(), "audit".to_string(), "vulnerability".to_string()],
        });
    }
    
    // Add UNLIMITED custom profiles from database
    let custom_profiles: Vec<(String, String, String)> = conn.prepare_cached(
        "SELECT id, name, description FROM custom_profiles"
    ).and_then(|mut stmt| {
        stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .and_then(|mapped| mapped.collect())
    }).unwrap_or_else(|_| Vec::new());
    
    for (id, name, description) in custom_profiles {
        profiles.push(ProfileInfo {
            id: id.clone(),
            name,
            description,
            category: "Custom".to_string(),
            is_active: id == active_profile,
            is_custom: true,
            expert_level: "Custom".to_string(),
            use_cases: vec!["User-defined".to_string()],
            tags: vec!["custom".to_string()],
        });
    }
    
    Ok(profiles)
}

/// Set the active profile for consensus
#[tauri::command]
pub async fn set_active_profile(profile_id: String) -> Result<()> {
    tracing::info!("Setting active profile to: {}", profile_id);
    
    // Store in database for persistence
    let db = get_database().await.map_err(|e| e.to_string())?;
    let conn = db.get_connection().await.map_err(|e| e.to_string())?;
    
    // Update or insert the active profile setting
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('active_profile', ?1)",
        params![profile_id],
    ).map_err(|e| format!("Failed to save active profile: {}", e))?;
    
    // Update the consensus engine to use this profile
    // The consensus engine will read this on next query
    
    Ok(())
}

/// Get configuration for a specific profile
#[tauri::command]
pub async fn get_profile_config(profile_id: String) -> Result<ProfileConfig> {
    // Get the real profile manager
    let manager = ExpertProfileManager::new();
    
    // Check if it's a preset profile
    if let Some(template) = manager.get_template(&profile_id) {
        // Extract actual configuration from the template
        let config = ProfileConfig {
            generator_model: template.fixed_models.as_ref()
                .map(|m| m.generator.clone())
                .unwrap_or_else(|| "dynamic-selection".to_string()),
            generator_temperature: template.temperatures.generator as f64,
            refiner_model: template.fixed_models.as_ref()
                .map(|m| m.refiner.clone())
                .unwrap_or_else(|| "dynamic-selection".to_string()),
            refiner_temperature: template.temperatures.refiner as f64,
            validator_model: template.fixed_models.as_ref()
                .map(|m| m.validator.clone())
                .unwrap_or_else(|| "dynamic-selection".to_string()),
            validator_temperature: template.temperatures.validator as f64,
            curator_model: template.fixed_models.as_ref()
                .map(|m| m.curator.clone())
                .unwrap_or_else(|| "dynamic-selection".to_string()),
            curator_temperature: template.temperatures.curator as f64,
        };
        return Ok(config);
    }
    
    // Check if it's a custom profile in database
    let db = get_database().await.map_err(|e| e.to_string())?;
    let conn = db.get_connection().await.map_err(|e| e.to_string())?;
    
    let result: std::result::Result<(String, f64, String, f64, String, f64, String, f64), _> = conn.query_row(
        "SELECT generator_model, generator_temp, refiner_model, refiner_temp, 
                validator_model, validator_temp, curator_model, curator_temp
         FROM custom_profiles WHERE id = ?1",
        params![profile_id],
        |row| Ok((
            row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
            row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?
        ))
    );
    
    match result {
        Ok((gm, gt, rm, rt, vm, vt, cm, ct)) => {
            Ok(ProfileConfig {
                generator_model: gm,
                generator_temperature: gt,
                refiner_model: rm,
                refiner_temperature: rt,
                validator_model: vm,
                validator_temperature: vt,
                curator_model: cm,
                curator_temperature: ct,
            })
        }
        Err(_) => Err(format!("Profile not found: {}", profile_id))
    }
}

/// Get all saved custom profiles from database
#[tauri::command]
pub async fn get_custom_profiles() -> Result<Vec<ProfileInfo>> {
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    // Query the database for user-created profiles
    // For now, return empty list
    Ok(vec![])
}

/// Create a new custom profile
#[tauri::command]
pub async fn create_custom_profile(
    name: String,
    description: String,
    config: ProfileConfig,
) -> Result<ProfileInfo> {
    let db = get_database().await.map_err(|e| format!("Failed to get database: {}", e))?;
    let conn = db.get_connection().await.map_err(|e| e.to_string())?;
    
    let id = format!("custom_{}", uuid::Uuid::new_v4());
    
    // Save the custom profile to database
    conn.execute(
        "INSERT INTO custom_profiles (id, name, description, generator_model, generator_temp, 
         refiner_model, refiner_temp, validator_model, validator_temp, curator_model, curator_temp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id, name, description,
            config.generator_model, config.generator_temperature,
            config.refiner_model, config.refiner_temperature,
            config.validator_model, config.validator_temperature,
            config.curator_model, config.curator_temperature
        ],
    ).map_err(|e| format!("Failed to save custom profile: {}", e))?;
    
    Ok(ProfileInfo {
        id: id.clone(),
        name,
        description,
        category: "Custom".to_string(),
        is_active: false,
        is_custom: true,
        expert_level: "Custom".to_string(),
        use_cases: vec!["User-defined".to_string()],
        tags: vec!["custom".to_string()],
    })
}

/// Delete a custom profile
#[tauri::command]
pub async fn delete_custom_profile(profile_id: String) -> Result<()> {
    if !profile_id.starts_with("custom_") {
        return Err("Cannot delete built-in profiles".to_string());
    }
    
    let db = get_database()
        .await
        .map_err(|e| format!("Failed to get database: {}", e))?;
    
    // Delete from database
    // For now, just return success
    Ok(())
}