-- @ name: add_licenses_table
-- Migration: Add licenses table to replace JSON storage
-- This table stores validated license information that was previously in license.json

CREATE TABLE IF NOT EXISTS licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,
    tier TEXT NOT NULL,
    daily_limit INTEGER NOT NULL,
    features TEXT, -- JSON array stored as text
    user_id TEXT NOT NULL,
    email TEXT,
    expires_at TEXT,
    validated_at TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_licenses_user_id ON licenses(user_id);
CREATE INDEX IF NOT EXISTS idx_licenses_key ON licenses(key);

-- Migration will be tracked by the migration system, no need to insert manually