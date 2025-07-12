-- Fix consensus_profiles to match TypeScript schema
-- The TypeScript version stores OpenRouter model IDs directly as TEXT, not as INTEGER foreign keys

-- First, drop the existing table
DROP TABLE IF EXISTS consensus_profiles;

-- Recreate with the correct schema matching TypeScript
CREATE TABLE consensus_profiles (
    id TEXT PRIMARY KEY,
    profile_name TEXT NOT NULL,
    generator_model TEXT NOT NULL,
    refiner_model TEXT NOT NULL,
    validator_model TEXT NOT NULL,
    curator_model TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Insert the default profiles with proper OpenRouter model IDs
INSERT INTO consensus_profiles (id, profile_name, generator_model, refiner_model, validator_model, curator_model) VALUES
('lightning-fast', 'Lightning Fast', 'anthropic/claude-3-haiku', 'openai/gpt-3.5-turbo', 'google/gemini-flash', 'anthropic/claude-3-haiku'),
('precision-architect', 'Precision Architect', 'anthropic/claude-3.5-sonnet', 'openai/gpt-4o', 'google/gemini-pro-1.5', 'anthropic/claude-3-opus'),
('budget-optimizer', 'Budget Optimizer', 'meta-llama/llama-3.1-8b-instruct', 'mistralai/mistral-7b-instruct', 'google/gemini-flash', 'meta-llama/llama-3.1-8b-instruct'),
('research-deep-dive', 'Research Deep Dive', 'anthropic/claude-3-opus', 'openai/gpt-4-turbo', 'google/gemini-pro-1.5', 'anthropic/claude-3-opus'),
('startup-mvp', 'Startup MVP', 'anthropic/claude-3-haiku', 'openai/gpt-4o-mini', 'google/gemini-flash', 'openai/gpt-3.5-turbo'),
('enterprise-grade', 'Enterprise Grade', 'anthropic/claude-3.5-sonnet', 'openai/gpt-4o', 'google/gemini-pro-1.5', 'anthropic/claude-3-opus'),
('creative-innovator', 'Creative Innovator', 'anthropic/claude-3-sonnet', 'openai/gpt-4o', 'google/gemini-pro', 'anthropic/claude-3-opus'),
('security-focused', 'Security Focused', 'anthropic/claude-3-opus', 'openai/gpt-4-turbo', 'anthropic/claude-3.5-sonnet', 'openai/gpt-4o'),
('ml-ai-specialist', 'ML/AI Specialist', 'anthropic/claude-3.5-sonnet', 'openai/gpt-4o', 'google/gemini-pro-1.5', 'anthropic/claude-3-opus'),
('debugging-detective', 'Debugging Detective', 'anthropic/claude-3.5-sonnet', 'openai/gpt-4o', 'anthropic/claude-3-haiku', 'openai/gpt-4-turbo');

-- Set lightning-fast as the default in consensus_settings
INSERT OR REPLACE INTO consensus_settings (key, value) VALUES ('active_profile_id', 'lightning-fast');