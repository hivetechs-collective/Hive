//! Migration command handlers
//! 
//! Provides CLI commands for TypeScript to Rust migration operations.

use crate::core::error::HiveError;
use crate::migration::{
    MigrationManager, MigrationConfig, MigrationType, ValidationLevel,
    analyze_only, preview_migration,
    analyzer::TypeScriptAnalysis,
    guide::{generate_migration_guide, display_migration_guide},
    rollback::perform_rollback,
};
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Migration command arguments
#[derive(Debug, Args)]
pub struct MigrateArgs {
    #[command(subcommand)]
    pub command: MigrateCommand,
}

/// Migration subcommands
#[derive(Debug, Subcommand)]
pub enum MigrateCommand {
    /// Analyze existing TypeScript installation
    Analyze {
        /// Path to TypeScript installation
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Output format
        #[arg(long, default_value = "summary")]
        format: String,
        
        /// Save analysis to file
        #[arg(long)]
        output: Option<PathBuf>,
    },
    
    /// Preview migration changes without applying them
    Preview {
        /// Path to TypeScript installation
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Migration type
        #[arg(long, default_value = "upgrade")]
        migration_type: String,
        
        /// Validation level
        #[arg(long, default_value = "standard")]
        validation: String,
    },
    
    /// Generate migration guide
    Guide {
        /// Path to TypeScript installation
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Save guide to file
        #[arg(long)]
        output: Option<PathBuf>,
    },
    
    /// Migrate configuration only
    Config {
        /// Path to TypeScript installation
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Verify configuration after migration
        #[arg(long)]
        verify: bool,
    },
    
    /// Migrate database only
    Database {
        /// Path to TypeScript installation
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Verify database after migration
        #[arg(long)]
        verify: bool,
    },
    
    /// Full migration (configuration + database + validation)
    Full {
        /// Path to TypeScript installation
        #[arg(long, default_value = ".")]
        path: PathBuf,
        
        /// Backup path for safety
        #[arg(long)]
        backup_path: Option<PathBuf>,
        
        /// Preserve original installation
        #[arg(long)]
        preserve_original: bool,
        
        /// Migration type
        #[arg(long, default_value = "upgrade")]
        migration_type: String,
        
        /// Validation level
        #[arg(long, default_value = "standard")]
        validation: String,
        
        /// Automatic mode (no prompts)
        #[arg(long)]
        auto: bool,
    },
    
    /// Verify migration integrity
    Verify {
        /// Validation level
        #[arg(long, default_value = "standard")]
        validation: String,
        
        /// Generate detailed report
        #[arg(long)]
        detailed: bool,
    },
    
    /// Rollback migration
    Rollback {
        /// Confirm rollback operation
        #[arg(long)]
        confirm: bool,
        
        /// Path to backup
        #[arg(long)]
        backup_path: Option<PathBuf>,
    },
    
    /// Check migration status
    Status,
}

/// Handle migration commands
pub async fn handle_migrate(args: MigrateArgs) -> Result<(), HiveError> {
    match args.command {
        MigrateCommand::Analyze { path, format, output } => {
            handle_analyze_command(path, format, output).await
        },
        MigrateCommand::Preview { path, migration_type, validation } => {
            handle_preview_command(path, migration_type, validation).await
        },
        MigrateCommand::Guide { path, output } => {
            handle_guide_command(path, output).await
        },
        MigrateCommand::Config { path, verify } => {
            handle_config_migration(path, verify).await
        },
        MigrateCommand::Database { path, verify } => {
            handle_database_migration(path, verify).await
        },
        MigrateCommand::Full { 
            path, 
            backup_path, 
            preserve_original, 
            migration_type, 
            validation, 
            auto 
        } => {
            handle_full_migration(
                path, 
                backup_path, 
                preserve_original, 
                migration_type, 
                validation, 
                auto
            ).await
        },
        MigrateCommand::Verify { validation, detailed } => {
            handle_verify_command(validation, detailed).await
        },
        MigrateCommand::Rollback { confirm, backup_path } => {
            handle_rollback_command(confirm, backup_path).await
        },
        MigrateCommand::Status => {
            handle_status_command().await
        },
    }
}

/// Handle analyze command
async fn handle_analyze_command(
    path: PathBuf,
    format: String,
    output: Option<PathBuf>,
) -> Result<(), HiveError> {
    println!("🔍 Analyzing TypeScript installation at: {}", path.display());
    
    let analysis = analyze_only(&path).await?;
    
    let output_text = match format.as_str() {
        "json" => serde_json::to_string_pretty(&analysis)?,
        "summary" => format_analysis_summary(&analysis),
        "detailed" => format_analysis_detailed(&analysis),
        _ => return Err(HiveError::DatabaseMigration { version: "unknown".to_string(), target: format }),
    };
    
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &output_text).await?;
        println!("✅ Analysis saved to: {}", output_path.display());
    } else {
        println!("{}", output_text);
    }
    
    Ok(())
}

/// Handle preview command
async fn handle_preview_command(
    path: PathBuf,
    migration_type: String,
    validation: String,
) -> Result<(), HiveError> {
    println!("👀 Previewing migration changes...");
    
    let config = create_migration_config(path, None, false, migration_type, validation)?;
    let preview = preview_migration(&config).await?;
    
    println!("\n📋 Migration Preview");
    println!("==================");
    println!("Source: {}", preview.analysis.installation_path.display());
    println!("Compatibility: {:.1}%", preview.analysis.compatibility.overall_score * 100.0);
    println!("Estimated duration: {:?}", preview.estimated_duration);
    
    println!("\n📁 Configuration Changes:");
    for config_change in &preview.config_changes.source_configs {
        println!("  • {} -> {}", 
            config_change.source_path.display(), 
            config_change.target_section
        );
    }
    
    println!("\n🗄️ Database Changes:");
    println!("  • Total tables: {}", preview.database_changes.schema_changes.len());
    println!("  • Estimated size: {} MB", preview.database_changes.estimated_size / 1_000_000);
    
    if !preview.risks.is_empty() {
        println!("\n⚠️ Identified Risks:");
        for risk in &preview.risks {
            println!("  • {}", risk);
        }
    }
    
    Ok(())
}

/// Handle guide command
async fn handle_guide_command(path: PathBuf, output: Option<PathBuf>) -> Result<(), HiveError> {
    println!("📖 Generating migration guide...");
    
    let analysis = analyze_only(&path).await?;
    let config = MigrationConfig {
        source_path: path,
        backup_path: None,
        preserve_original: true,
        validation_level: ValidationLevel::Standard,
        migration_type: MigrationType::Upgrade,
    };
    
    let guide = generate_migration_guide(&analysis, &config).await?;
    let guide_text = display_migration_guide(&guide);
    
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &guide_text).await?;
        println!("✅ Migration guide saved to: {}", output_path.display());
    } else {
        println!("{}", guide_text);
    }
    
    Ok(())
}

/// Handle configuration migration
async fn handle_config_migration(path: PathBuf, verify: bool) -> Result<(), HiveError> {
    println!("⚙️ Migrating configuration...");
    
    let analysis = analyze_only(&path).await?;
    crate::migration::config::migrate_configuration(&analysis).await?;
    
    println!("✅ Configuration migrated successfully");
    
    if verify {
        println!("🔍 Verifying configuration...");
        // Verify configuration is valid
        let config_path = dirs::home_dir()
            .ok_or_else(|| HiveError::EnvVarNotSet { var: "HOME".to_string() })?
            .join(".hive").join("config.toml");
        
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            match toml::from_str::<toml::Value>(&content) {
                Ok(_) => println!("✅ Configuration verification passed"),
                Err(e) => return Err(HiveError::ConfigInvalid { message: format!("Configuration verification failed: {}", e) }),
            }
        }
    }
    
    Ok(())
}

/// Handle database migration
async fn handle_database_migration(path: PathBuf, verify: bool) -> Result<(), HiveError> {
    println!("🗄️ Migrating database...");
    
    let analysis = analyze_only(&path).await?;
    crate::migration::database::migrate_database(&analysis).await?;
    
    println!("✅ Database migrated successfully");
    
    if verify {
        println!("🔍 Verifying database...");
        // Basic database verification
        let db_path = dirs::home_dir()
            .ok_or_else(|| HiveError::EnvVarNotSet { var: "HOME".to_string() })?
            .join(".hive").join("hive.db");
        
        if db_path.exists() {
            println!("✅ Database verification passed");
        } else {
            return Err(HiveError::FileNotFound { path: db_path });
        }
    }
    
    Ok(())
}

/// Handle full migration
async fn handle_full_migration(
    path: PathBuf,
    backup_path: Option<PathBuf>,
    preserve_original: bool,
    migration_type: String,
    validation: String,
    auto: bool,
) -> Result<(), HiveError> {
    println!("🚀 Starting full migration...");
    
    if !auto {
        println!("⚠️  This will migrate your TypeScript Hive AI installation to Rust.");
        println!("   Make sure you have a backup before proceeding.");
        print!("   Continue? (y/N): ");
        
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Migration cancelled.");
            return Ok(());
        }
    }
    
    let config = create_migration_config(path, backup_path, preserve_original, migration_type, validation)?;
    let mut manager = MigrationManager::new(config);
    
    match manager.migrate().await {
        Ok(_) => {
            println!("✅ Migration completed successfully!");
            println!("   Your TypeScript Hive AI data has been migrated to Rust.");
            println!("   Run 'hive status' to verify the installation.");
        },
        Err(e) => {
            println!("❌ Migration failed: {}", e);
            println!("   Use 'hive migrate rollback --confirm' to restore the previous state.");
            return Err(e);
        }
    }
    
    Ok(())
}

/// Handle verify command
async fn handle_verify_command(validation: String, detailed: bool) -> Result<(), HiveError> {
    println!("🔍 Verifying migration...");
    
    let validation_level = parse_validation_level(&validation)?;
    let result = crate::migration::validator::validate_migration(&validation_level).await?;
    
    if result.is_valid() {
        println!("✅ Migration verification passed: {}", result.summary());
    } else {
        println!("❌ Migration verification failed: {}", result.summary());
        
        if !result.errors.is_empty() {
            println!("\n🚨 Errors:");
            for error in &result.errors {
                println!("  • {}", error);
            }
        }
        
        if !result.warnings.is_empty() {
            println!("\n⚠️ Warnings:");
            for warning in &result.warnings {
                println!("  • {}", warning);
            }
        }
    }
    
    if detailed {
        println!("\n📊 Detailed Results:");
        for check in &result.checks {
            let status_icon = match check.status {
                crate::migration::validator::ValidationCheckStatus::Passed => "✅",
                crate::migration::validator::ValidationCheckStatus::Failed => "❌",
                crate::migration::validator::ValidationCheckStatus::Warning => "⚠️",
                crate::migration::validator::ValidationCheckStatus::Skipped => "⏭️",
            };
            println!("  {} {}: {}", status_icon, check.name, check.description);
        }
        
        println!("\n⏱️ Performance Metrics:");
        println!("  • Migration duration: {:?}", result.performance_metrics.migration_duration);
        println!("  • Data transfer rate: {:.1} MB/s", result.performance_metrics.data_transfer_rate);
        println!("  • Memory usage: {} MB", result.performance_metrics.memory_usage / 1_000_000);
    }
    
    Ok(())
}

/// Handle rollback command
async fn handle_rollback_command(confirm: bool, backup_path: Option<PathBuf>) -> Result<(), HiveError> {
    if !confirm {
        println!("⚠️  This will rollback your migration and restore the TypeScript version.");
        println!("   This operation cannot be undone.");
        print!("   Are you sure? (y/N): ");
        
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Rollback cancelled.");
            return Ok(());
        }
    }
    
    println!("🔄 Rolling back migration...");
    
    let config = MigrationConfig {
        source_path: PathBuf::from("."),
        backup_path,
        preserve_original: true,
        validation_level: ValidationLevel::Standard,
        migration_type: MigrationType::Upgrade,
    };
    
    let result = perform_rollback(&config).await?;
    
    if result.success {
        println!("✅ Rollback completed successfully");
        println!("   Your TypeScript installation has been restored.");
    } else {
        println!("❌ Rollback failed");
        for error in &result.errors {
            println!("  • {}", error);
        }
        return Err(HiveError::DatabaseMigration { version: "rollback".to_string(), target: "failed".to_string() });
    }
    
    Ok(())
}

/// Handle status command
async fn handle_status_command() -> Result<(), HiveError> {
    println!("📊 Migration Status");
    println!("==================");
    
    // Check if Rust version is installed
    let rust_config_path = dirs::home_dir()
        .ok_or_else(|| HiveError::EnvVarNotSet { var: "HOME".to_string() })?
        .join(".hive").join("config.toml");
    
    let rust_db_path = dirs::home_dir()
        .ok_or_else(|| HiveError::EnvVarNotSet { var: "HOME".to_string() })?
        .join(".hive").join("hive.db");
    
    if rust_config_path.exists() || rust_db_path.exists() {
        println!("🦀 Rust installation: ✅ Found");
        
        if rust_config_path.exists() {
            println!("  • Configuration: ✅ Present");
        }
        
        if rust_db_path.exists() {
            let metadata = tokio::fs::metadata(&rust_db_path).await?;
            println!("  • Database: ✅ Present ({} bytes)", metadata.len());
        }
    } else {
        println!("🦀 Rust installation: ❌ Not found");
    }
    
    // Check for TypeScript version
    let ts_config_paths = vec![
        PathBuf::from(".hive"),
        dirs::home_dir().unwrap_or_default().join(".hive.backup"),
    ];
    
    let mut ts_found = false;
    for ts_path in ts_config_paths {
        if ts_path.exists() {
            ts_found = true;
            println!("📦 TypeScript installation: ✅ Found at {}", ts_path.display());
            break;
        }
    }
    
    if !ts_found {
        println!("📦 TypeScript installation: ❌ Not found");
    }
    
    // Check migration status
    let migration_backup_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".hive")
        .join("backups")
        .join("rollback_plan.json");
    
    if migration_backup_path.exists() {
        println!("🔄 Migration: ✅ Completed (rollback available)");
    } else if rust_config_path.exists() || rust_db_path.exists() {
        println!("🔄 Migration: ⚠️ Partial (no rollback plan)");
    } else {
        println!("🔄 Migration: ❌ Not started");
    }
    
    Ok(())
}

/// Create migration config from command line arguments
fn create_migration_config(
    path: PathBuf,
    backup_path: Option<PathBuf>,
    preserve_original: bool,
    migration_type: String,
    validation: String,
) -> Result<MigrationConfig, HiveError> {
    let migration_type = match migration_type.as_str() {
        "fresh" => MigrationType::Fresh,
        "upgrade" => MigrationType::Upgrade,
        "parallel" => MigrationType::Parallel,
        "staged" => MigrationType::Staged,
        _ => return Err(HiveError::DatabaseMigration { version: migration_type, target: "unknown".to_string() }),
    };
    
    let validation_level = parse_validation_level(&validation)?;
    
    Ok(MigrationConfig {
        source_path: path,
        backup_path,
        preserve_original,
        validation_level,
        migration_type,
    })
}

/// Parse validation level from string
fn parse_validation_level(validation: &str) -> Result<ValidationLevel, HiveError> {
    match validation {
        "basic" => Ok(ValidationLevel::Basic),
        "standard" => Ok(ValidationLevel::Standard),
        "strict" => Ok(ValidationLevel::Strict),
        "paranoid" => Ok(ValidationLevel::Paranoid),
        _ => Err(HiveError::ConfigInvalid { message: format!("Unknown validation level: {}", validation) }),
    }
}

/// Format analysis summary
fn format_analysis_summary(analysis: &TypeScriptAnalysis) -> String {
    format!(
        "📊 TypeScript Installation Analysis\n\
         ====================================\n\
         Installation: {}\n\
         Version: {}\n\
         Conversations: {}\n\
         Messages: {}\n\
         Custom Profiles: {}\n\
         Compatibility: {:.1}%\n\
         Migration Readiness: {:.1}%\n\
         Recommended Approach: {}\n",
        analysis.installation_path.display(),
        analysis.version.as_deref().unwrap_or("unknown"),
        analysis.database_info.conversation_count,
        analysis.database_info.message_count,
        analysis.custom_profiles.len(),
        analysis.compatibility.overall_score * 100.0,
        analysis.migration_readiness() * 100.0,
        analysis.compatibility.recommended_approach
    )
}

/// Format detailed analysis
fn format_analysis_detailed(analysis: &TypeScriptAnalysis) -> String {
    let mut output = format_analysis_summary(analysis);
    
    output.push_str("\n📁 Configuration Files:\n");
    for config in &analysis.config_files {
        output.push_str(&format!(
            "  • {} ({} bytes) - {}\n",
            config.path.display(),
            config.size,
            if config.valid { "✅ Valid" } else { "❌ Invalid" }
        ));
    }
    
    if !analysis.issues.is_empty() {
        output.push_str("\n⚠️ Issues Found:\n");
        for issue in &analysis.issues {
            let severity_icon = match issue.severity {
                crate::migration::analyzer::IssueSeverity::Critical => "🚨",
                crate::migration::analyzer::IssueSeverity::Error => "❌",
                crate::migration::analyzer::IssueSeverity::Warning => "⚠️",
                crate::migration::analyzer::IssueSeverity::Info => "ℹ️",
            };
            output.push_str(&format!(
                "  {} {}\n     Recommendation: {}\n",
                severity_icon, issue.description, issue.recommendation
            ));
        }
    }
    
    output
}