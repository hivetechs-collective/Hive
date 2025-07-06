# Database Usage Report - Agent 1

## Summary

The codebase has **TWO different database files** being used inconsistently:
1. **hive-ai.db** - The original/default database name
2. **hive.db** - A new database name used in some components

## Database Files by Component

### Using `hive-ai.db`:
- **src/core/database.rs** - Default path: `~/.hive/hive-ai.db`
- **src/core/database_working.rs** - Default path: `~/.hive/hive-ai.db`
- **src/core/database_simple.rs** - Default path: `~/.hive/hive-ai.db`
- **src/desktop/dialogs.rs** - Uses `get_hive_config_dir().join("hive-ai.db")`
- **src/desktop/dialogs_backup.rs** - Uses `get_hive_config_dir().join("hive-ai.db")`
- **src/desktop/simple_db.rs** - Uses `home.join(".hive").join("hive-ai.db")`
- **src/migration/analyzer.rs** - Multiple paths including `.hive/hive-ai.db`
- **src/migration/rollback.rs** - Source DB: `source_path.join("hive-ai.db")`
- **src/commands/memory.rs** - References `~/.hive/hive-ai.db` in user message

### Using `hive.db`:
- **src/core/config.rs** - Uses `hive_dir.join("data").join("hive.db")`
- **src/commands/quickstart.rs** - Uses `config_dir.join("hive.db")` (4 occurrences)
- **src/commands/migrate.rs** - Uses `.hive/hive.db`
- **src/cli/commands.rs** - Uses `hive_dir.join("hive.db")`
- **src/migration/validator.rs** - Uses `.hive/hive.db`
- **src/migration/database.rs** - Uses `.hive/hive.db`
- **src/migration/rollback.rs** - Target DB: `.hive/hive.db`
- **tests/migration/integration/mod.rs** - Uses `.join("hive.db")`

### Using `conversations.db` (Different pattern):
- **src/cli/interactive.rs** - Uses `config_dir.join("conversations.db")`
- **src/cli/banner.rs** - Uses `config_dir.join("conversations.db")`
- **src/cli/commands.rs** - References `~/.hive/conversations.db`
- **src/main-full.rs** - Uses `config_dir.join("conversations.db")`
- **src/core/migrator.rs** - Uses `rust_config_dir.join("conversations.db")`
- **src/core/uninstaller.rs** - References `conversations.db` and related files
- **essential_commands.rs** - Uses `.hive/conversations.db`

## Database Connection Patterns

### DatabaseManager Usage:
- Primary database manager in `src/core/database.rs`
- Also defined in `src/core/database_working.rs` and `src/core/database_simple.rs`
- Used in ~31 files throughout the codebase
- Global static instance: `static DATABASE: OnceCell<Arc<DatabaseManager>>`

### Direct rusqlite::Connection Usage:
- **src/migration/performance.rs**
- **src/core/migrator.rs**
- **src/core/schema.rs**
- **src/database/migrations.rs**
- **src/database/schema.rs**
- **tests/migration/integration/mod.rs**
- **tests/database_migration_validation.rs**

### r2d2 Connection Pool Usage:
- **src/startup/fast_boot.rs** - Uses `r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>`
- **src/database/optimize.rs** - Uses `r2d2_sqlite::SqliteConnectionManager`
- **src/core/database.rs** - Main implementation with connection pooling
- **src/core/database_working.rs** - Alternative implementation

## Initialization Functions

### Primary Functions:
1. **initialize_database()** - Found in:
   - `src/core/database.rs`
   - `src/core/database_working.rs`
   - `src/core/database_simple.rs`

2. **get_database()** - Found in:
   - `src/core/database.rs`
   - `src/core/database_working.rs`
   - Used by ~30+ files for database access

3. **DatabaseManager::new()** - Direct instantiation in:
   - `src/commands/quickstart.rs`
   - `src/desktop/dialogs.rs`
   - `src/desktop/dialogs_backup.rs`
   - Various test files

## Configuration Locations

### Default Paths:
1. **DatabaseConfig default** (in database.rs): `~/.hive/hive-ai.db`
2. **Quickstart command**: `config_dir.join("hive.db")`
3. **Migration tools**: `.hive/hive.db`
4. **Config module**: `hive_dir.join("data").join("hive.db")`

## Critical Issues

1. **Inconsistent Database Names**: Some components use `hive-ai.db` while others use `hive.db`
2. **Multiple Database Types**: `conversations.db` appears to be a separate database
3. **Direct Connection Usage**: Some modules bypass DatabaseManager and use rusqlite directly
4. **Path Construction Variations**: Different ways of constructing the database path

## Recommendations for Unification

1. **Standardize on ONE database filename** (recommend: `hive.db`)
2. **Use DatabaseManager everywhere** instead of direct rusqlite connections
3. **Centralize path configuration** in DatabaseConfig
4. **Remove duplicate database module implementations**
5. **Ensure all components use the same initialization pattern**

## Files Requiring Updates

### High Priority (Core Database Modules):
- `src/core/database.rs` - Change default from `hive-ai.db` to `hive.db`
- `src/core/database_working.rs` - Same change needed
- `src/core/database_simple.rs` - Same change needed

### Medium Priority (Desktop/Migration):
- `src/desktop/dialogs.rs` - Update all 6 occurrences
- `src/desktop/dialogs_backup.rs` - Update all 6 occurrences
- `src/desktop/simple_db.rs` - Update path
- `src/migration/analyzer.rs` - Update multiple references
- `src/migration/rollback.rs` - Update source database reference

### Low Priority (User Messages):
- `src/commands/memory.rs` - Update user-facing message

### Already Correct:
- `src/commands/quickstart.rs` - Already uses `hive.db`
- `src/cli/commands.rs` - Already uses `hive.db`
- `src/migration/database.rs` - Already uses `hive.db`