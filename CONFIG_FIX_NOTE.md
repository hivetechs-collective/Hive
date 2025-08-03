# Consensus Engine Configuration Fix

## Issue
The consensus engine was failing to initialize with the error:
```
Failed to create consensus manager: TOML parse error at line 264, column 1
missing field `extract_project_info`
```

## Root Cause
The user's config file at `~/.hive/config.toml` was missing required fields in the `[repository_discovery]` section.

## Fix Applied
Added the following fields to the `[repository_discovery]` section in `~/.hive/config.toml`:
```toml
# Extract project info
extract_project_info = true

# Operation timeout in seconds
operation_timeout_seconds = 10
```

## Result
- Consensus engine now initializes properly
- The 4-stage consensus pipeline (Generator → Refiner → Validator → Curator) works correctly
- Users can now use the consensus chat interface with their configured profiles

## Note
The `use_consensus_with_version` function in `src/desktop/consensus_integration.rs` was NOT removed during the TUI/CLI cleanup. It exists and works correctly - it was just returning `None` due to the config parsing error.