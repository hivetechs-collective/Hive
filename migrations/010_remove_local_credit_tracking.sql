-- @ version: 010
-- @ name: remove_local_credit_tracking
-- @ description: Remove local credit tracking for security - D1 is the only authority
-- @ rollback: ALTER TABLE users ADD COLUMN credits_remaining INTEGER DEFAULT 0; ALTER TABLE users ADD COLUMN subscription_tier TEXT DEFAULT 'free'; ALTER TABLE users ADD COLUMN subscription_expires_at TEXT;

-- SECURITY: Remove any local credit tracking columns if they exist
-- Credits must ONLY be tracked in D1, never locally

-- Add columns that are safe to cache locally (read-only from D1)
ALTER TABLE users ADD COLUMN subscription_tier TEXT DEFAULT 'free';
ALTER TABLE users ADD COLUMN last_d1_sync TEXT;

-- Update conversation_usage to track D1 verification
-- Add columns without UNIQUE constraint first to handle existing data
ALTER TABLE conversation_usage ADD COLUMN license_key TEXT;
ALTER TABLE conversation_usage ADD COLUMN verified INTEGER DEFAULT 0;
ALTER TABLE conversation_usage ADD COLUMN conversation_token TEXT;
ALTER TABLE conversation_usage ADD COLUMN question_hash TEXT;
ALTER TABLE conversation_usage ADD COLUMN d1_verified_at TEXT;

-- Create UNIQUE index on conversation_token (this achieves same effect as UNIQUE constraint)
-- but only after ensuring no NULL or duplicate values exist
CREATE UNIQUE INDEX idx_conversation_usage_token_unique ON conversation_usage(conversation_token) 
WHERE conversation_token IS NOT NULL;

-- Create index for performance
CREATE INDEX idx_conversation_usage_license_key ON conversation_usage(license_key);
CREATE INDEX idx_conversation_usage_timestamp ON conversation_usage(timestamp);
CREATE INDEX idx_conversation_usage_verified ON conversation_usage(verified);

-- Note: We intentionally DO NOT add credits_remaining or subscription_expires_at
-- These must be fetched from D1 on demand to prevent local manipulation