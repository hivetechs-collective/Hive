# Consensus Engine Configuration Update

## Overview
Updated the consensus engine to load OpenRouter API keys from the configuration file instead of environment variables. This ensures the consensus engine works seamlessly after the setup flow saves the API keys.

## Changes Made

### 1. **ConsensusEngine** (`src/consensus/engine.rs`)
- Added `openrouter_api_key: Option<String>` field to store the API key
- Modified `new()` method to:
  - Load configuration using `config::get_config().await`
  - Extract OpenRouter API key from the loaded configuration
  - Pass the API key to the consensus pipeline
- Updated `validate_prerequisites()` to check the stored API key instead of environment variable
- Updated error message to suggest running 'hive setup' instead of setting environment variables

### 2. **ConsensusPipeline** (`src/consensus/pipeline.rs`)
- Added `api_key: Option<String>` parameter to the `new()` method
- Added `api_key: Option<String>` field to the struct
- Modified initialization to use the provided API key instead of checking environment variables
- Updated error message when API key is missing to suggest running 'hive setup'

### 3. **Profile Creation Methods** (`src/consensus/engine.rs`)
- Renamed methods from `get_*_profile()` to `create_*_profile()` for consistency
- Made these methods static (no `self` parameter needed)

## Benefits

1. **Consistent Configuration**: The consensus engine now uses the same configuration system as the rest of the application
2. **No Environment Variables**: Users don't need to set OPENROUTER_API_KEY manually
3. **Setup Flow Integration**: Works seamlessly with the 'hive setup' command
4. **Better Error Messages**: Clear guidance to run 'hive setup' when API key is missing

## Testing

After building the project:

```bash
# Check if configuration is loaded correctly
hive config get openrouter.api_key

# Test consensus functionality
hive ask "What is Rust?"

# Validate prerequisites
hive consensus validate
```

## Migration Notes

Users who previously set the OPENROUTER_API_KEY environment variable should:
1. Run `hive setup` to save their API key to the configuration file
2. Remove the OPENROUTER_API_KEY environment variable (optional, as it's no longer used)

The consensus engine will now automatically load the API key from `~/.hive/config.toml`.