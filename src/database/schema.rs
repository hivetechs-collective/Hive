//! Database schema definitions and table creation
//!
//! This module defines the SQLite schema for all Hive AI tables,
//! matching the TypeScript implementation for 100% compatibility.

use anyhow::Result;
use rusqlite::{Connection, params};
use tracing::info;

/// Create or update default consensus profiles
pub fn create_default_profiles(conn: &Connection) -> Result<()> {
    info!("Creating default consensus profiles");

    // Check if profiles already exist
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM consensus_profiles",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        info!("Consensus profiles already exist, skipping creation");
        return Ok(());
    }

    // Create default profiles matching TypeScript implementation
    let profiles = vec![
        (
            "Consensus_Elite",
            "anthropic/claude-3-opus-20240229",
            "openai/gpt-4o",
            "anthropic/claude-3-opus-20240229",
            "openai/gpt-4o",
        ),
        (
            "Consensus_Balanced",
            "anthropic/claude-3-5-sonnet-20241022",
            "openai/gpt-4-turbo",
            "anthropic/claude-3-opus-20240229",
            "openai/gpt-4o",
        ),
        (
            "Consensus_Speed",
            "anthropic/claude-3-haiku-20240307",
            "openai/gpt-3.5-turbo",
            "anthropic/claude-3-haiku-20240307",
            "openai/gpt-3.5-turbo",
        ),
        (
            "Consensus_Cost",
            "meta-llama/llama-3.2-3b-instruct",
            "mistralai/mistral-7b-instruct",
            "meta-llama/llama-3.2-3b-instruct",
            "mistralai/mistral-7b-instruct",
        ),
    ];

    let profile_count = profiles.len();
    
    for (name, generator, refiner, validator, curator) in profiles {
        conn.execute(
            "INSERT INTO consensus_profiles (
                id, profile_name, generator_model, refiner_model, 
                validator_model, curator_model
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &uuid::Uuid::new_v4().to_string(),
                name,
                generator,
                refiner,
                validator,
                curator,
            ],
        )?;
    }

    // Set Consensus_Balanced as the default active profile
    conn.execute(
        "INSERT OR REPLACE INTO consensus_settings (key, value) VALUES ('active_profile', 'Consensus_Balanced')",
        [],
    )?;

    info!("Created {} default consensus profiles", profile_count);
    Ok(())
}

/// Create default OpenRouter models for consensus profiles
pub fn seed_openrouter_models(conn: &Connection) -> Result<()> {
    info!("Seeding OpenRouter models for consensus profiles");

    // Check if models already exist
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM openrouter_models",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        info!("OpenRouter models already exist, skipping seeding");
        return Ok(());
    }

    // Insert essential models used by consensus profiles
    let models = vec![
        // Anthropic models
        ("anthropic/claude-3-opus-20240229", "Claude 3 Opus", "anthropic", 15.0, 75.0, 131072),
        ("anthropic/claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", "anthropic", 3.0, 15.0, 200000),
        ("anthropic/claude-3-haiku-20240307", "Claude 3 Haiku", "anthropic", 0.25, 1.25, 200000),
        // OpenAI models
        ("openai/gpt-4o", "GPT-4o", "openai", 5.0, 15.0, 128000),
        ("openai/gpt-4-turbo", "GPT-4 Turbo", "openai", 10.0, 30.0, 128000),
        ("openai/gpt-3.5-turbo", "GPT-3.5 Turbo", "openai", 0.5, 1.5, 16385),
        // Open source models
        ("meta-llama/llama-3.2-3b-instruct", "Llama 3.2 3B", "meta-llama", 0.06, 0.06, 131072),
        ("mistralai/mistral-7b-instruct", "Mistral 7B Instruct", "mistralai", 0.06, 0.06, 32768),
    ];

    // First, ensure providers exist
    let providers = vec![
        ("anthropic", "Anthropic", "Anthropic"),
        ("openai", "OpenAI", "OpenAI"),
        ("meta-llama", "Meta", "Meta Llama"),
        ("mistralai", "Mistral AI", "Mistral AI"),
    ];

    for (id, name, display_name) in providers {
        conn.execute(
            "INSERT OR IGNORE INTO openrouter_providers (
                id, name, display_name, last_updated
            ) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
            params![id, name, display_name],
        )?;
    }

    let model_count = models.len();
    
    // Then insert models
    for (id, name, provider_id, input_cost, output_cost, context_window) in models {
        conn.execute(
            "INSERT OR IGNORE INTO openrouter_models (
                openrouter_id, name, provider_id, provider_name,
                pricing_input, pricing_output, context_window,
                created_at, last_updated, is_active
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%s', 'now'), CURRENT_TIMESTAMP, 1)",
            params![
                id,
                name,
                provider_id,
                provider_id, // Using provider_id as provider_name for now
                input_cost / 1_000_000.0, // Convert to per-token pricing
                output_cost / 1_000_000.0,
                context_window,
            ],
        )?;
    }

    info!("Seeded {} OpenRouter models", model_count);
    Ok(())
}

/// Create a default user for the system
pub fn create_default_user(conn: &Connection) -> Result<String> {
    info!("Creating default user");

    let user_id = uuid::Uuid::new_v4().to_string();
    
    conn.execute(
        "INSERT OR IGNORE INTO users (id, tier) VALUES (?1, 'FREE')",
        params![&user_id],
    )?;

    // Set as current user
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('current_user_id', ?1)",
        params![&user_id],
    )?;

    Ok(user_id)
}

/// Initialize database with default data
pub fn initialize_default_data(conn: &Connection) -> Result<()> {
    info!("Initializing database with default data");

    // Create default user
    create_default_user(conn)?;

    // Seed OpenRouter models
    seed_openrouter_models(conn)?;

    // Create default consensus profiles
    create_default_profiles(conn)?;

    info!("Database initialization complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_create_default_profiles() -> Result<()> {
        let mut conn = Connection::open_in_memory()?;

        // Create necessary tables
        conn.execute(
            "CREATE TABLE consensus_profiles (
                id TEXT PRIMARY KEY,
                profile_name TEXT NOT NULL UNIQUE,
                pipeline_profile_id TEXT,
                generator_model TEXT NOT NULL,
                refiner_model TEXT NOT NULL,
                validator_model TEXT NOT NULL,
                curator_model TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE consensus_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create profiles
        create_default_profiles(&conn)?;

        // Verify profiles were created
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM consensus_profiles",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(count, 4);

        // Verify active profile was set
        let active: String = conn.query_row(
            "SELECT value FROM consensus_settings WHERE key = 'active_profile'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(active, "Consensus_Balanced");

        Ok(())
    }
}