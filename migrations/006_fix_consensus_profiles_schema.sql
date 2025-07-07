-- @ version: 006
-- @ name: fix_consensus_profiles_schema
-- @ description: Fix consensus_profiles table to match Rust code expectations
-- @ rollback: DROP TABLE IF EXISTS consensus_profiles_new; ALTER TABLE consensus_profiles_backup RENAME TO consensus_profiles;

-- First, rename the old table
ALTER TABLE consensus_profiles RENAME TO consensus_profiles_backup;

-- Create the new table with the correct schema matching Rust code
CREATE TABLE consensus_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,  -- Changed from profile_name to name
    user_id TEXT,
    generator_model_id INTEGER NOT NULL,  -- Changed from TEXT to INTEGER
    generator_temperature REAL NOT NULL DEFAULT 0.7,  -- Added temperature columns
    refiner_model_id INTEGER NOT NULL,
    refiner_temperature REAL NOT NULL DEFAULT 0.7,
    validator_model_id INTEGER NOT NULL,
    validator_temperature REAL NOT NULL DEFAULT 0.7,
    curator_model_id INTEGER NOT NULL,
    curator_temperature REAL NOT NULL DEFAULT 0.7,
    is_default BOOLEAN NOT NULL DEFAULT 0,  -- Added is_default column
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (generator_model_id) REFERENCES openrouter_models(internal_id),
    FOREIGN KEY (refiner_model_id) REFERENCES openrouter_models(internal_id),
    FOREIGN KEY (validator_model_id) REFERENCES openrouter_models(internal_id),
    FOREIGN KEY (curator_model_id) REFERENCES openrouter_models(internal_id)
);

-- Drop the backup table
DROP TABLE consensus_profiles_backup;

-- Create index for performance
CREATE INDEX idx_consensus_profiles_user_id ON consensus_profiles(user_id);
CREATE INDEX idx_consensus_profiles_is_default ON consensus_profiles(is_default);