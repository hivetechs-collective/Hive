-- @ version: 007
-- @ name: seed_initial_models
-- @ description: Add initial OpenRouter models for profile creation
-- @ rollback: DELETE FROM openrouter_models WHERE internal_id IN (1,2,3,4,5,6,7,8,9,10);

-- Insert initial providers if they don't exist
INSERT OR IGNORE INTO openrouter_providers (id, name, display_name, last_updated, is_active)
VALUES 
    ('anthropic', 'anthropic', 'Anthropic', datetime('now'), 1),
    ('openai', 'openai', 'OpenAI', datetime('now'), 1),
    ('google', 'google', 'Google', datetime('now'), 1),
    ('meta-llama', 'meta-llama', 'Meta Llama', datetime('now'), 1),
    ('mistralai', 'mistralai', 'Mistral AI', datetime('now'), 1);

-- Insert initial models with proper internal IDs
INSERT OR IGNORE INTO openrouter_models (
    internal_id, openrouter_id, name, provider_id, provider_name, 
    description, pricing_input, pricing_output, context_window, 
    created_at, is_active, last_updated
)
VALUES 
    -- Anthropic models
    (1, 'anthropic/claude-3.5-sonnet', 'Claude 3.5 Sonnet', 'anthropic', 'anthropic',
     'Most intelligent Claude model', 0.003, 0.015, 200000, 
     strftime('%s', 'now'), 1, datetime('now')),
    
    (6, 'anthropic/claude-3-opus', 'Claude 3 Opus', 'anthropic', 'anthropic',
     'Powerful Claude model for complex tasks', 0.015, 0.075, 200000,
     strftime('%s', 'now'), 1, datetime('now')),
    
    (7, 'anthropic/claude-3-sonnet', 'Claude 3 Sonnet', 'anthropic', 'anthropic',
     'Balanced Claude model', 0.003, 0.015, 200000,
     strftime('%s', 'now'), 1, datetime('now')),
    
    (8, 'anthropic/claude-3-haiku', 'Claude 3 Haiku', 'anthropic', 'anthropic',
     'Fast and efficient Claude model', 0.00025, 0.00125, 200000,
     strftime('%s', 'now'), 1, datetime('now')),
    
    -- OpenAI models
    (2, 'openai/gpt-4o', 'GPT-4o', 'openai', 'openai',
     'OpenAI flagship multimodal model', 0.005, 0.015, 128000,
     strftime('%s', 'now'), 1, datetime('now')),
    
    (3, 'openai/gpt-4-turbo', 'GPT-4 Turbo', 'openai', 'openai',
     'Latest GPT-4 Turbo model', 0.01, 0.03, 128000,
     strftime('%s', 'now'), 1, datetime('now')),
    
    (4, 'openai/gpt-4', 'GPT-4', 'openai', 'openai',
     'Original GPT-4 model', 0.03, 0.06, 8192,
     strftime('%s', 'now'), 1, datetime('now')),
    
    (9, 'openai/gpt-3.5-turbo', 'GPT-3.5 Turbo', 'openai', 'openai',
     'Fast and cost-effective model', 0.0005, 0.0015, 16385,
     strftime('%s', 'now'), 1, datetime('now')),
    
    -- Google models
    (5, 'google/gemini-pro-1.5', 'Gemini Pro 1.5', 'google', 'google',
     'Google Gemini Pro with large context', 0.00125, 0.005, 2097152,
     strftime('%s', 'now'), 1, datetime('now')),
    
    (10, 'google/gemini-pro', 'Gemini Pro', 'google', 'google',
     'Google Gemini Pro model', 0.000125, 0.000375, 32760,
     strftime('%s', 'now'), 1, datetime('now')),
    
    (11, 'google/gemini-flash', 'Gemini Flash', 'google', 'google',
     'Fast Gemini model', 0.000075, 0.0003, 1048576,
     strftime('%s', 'now'), 1, datetime('now')),
    
    -- Meta models
    (12, 'meta-llama/llama-3-8b-instruct', 'Llama 3 8B', 'meta-llama', 'meta-llama',
     'Open source Llama 3 model', 0.00006, 0.00006, 8192,
     strftime('%s', 'now'), 1, datetime('now')),
    
    -- Mistral models
    (13, 'mistralai/mistral-7b-instruct', 'Mistral 7B', 'mistralai', 'mistralai',
     'Efficient open model', 0.00006, 0.00006, 32768,
     strftime('%s', 'now'), 1, datetime('now'));