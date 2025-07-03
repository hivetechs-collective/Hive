//! Migration Guide Module
//! 
//! Provides interactive guidance, documentation, and help for users
//! performing TypeScript to Rust migration.

use crate::core::error::HiveError;
use crate::migration::{MigrationConfig, MigrationType, ValidationLevel};
use crate::migration::analyzer::TypeScriptAnalysis;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Migration guide with personalized recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationGuide {
    pub scenario: MigrationScenario,
    pub recommended_approach: MigrationApproach,
    pub preparation_steps: Vec<PreparationStep>,
    pub migration_steps: Vec<MigrationStep>,
    pub post_migration_steps: Vec<PostMigrationStep>,
    pub troubleshooting: Vec<TroubleshootingTip>,
    pub estimated_timeline: Timeline,
    pub risk_assessment: RiskAssessment,
}

/// Migration scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationScenario {
    FirstTimeUser,
    ExistingLightUser,
    ExistingHeavyUser,
    EnterpriseUser,
    DeveloperUser,
    TeamMigration,
}

/// Migration approaches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationApproach {
    QuickMigration,     // Fast, automated migration
    SafeMigration,      // Conservative, step-by-step
    ParallelMigration,  // Run both versions temporarily
    StagedMigration,    // Gradual feature migration
    CustomMigration,    // Tailored approach
}

/// Preparation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationStep {
    pub step_number: u32,
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub expected_outcome: String,
    pub troubleshooting: Option<String>,
    pub estimated_time: std::time::Duration,
    pub required: bool,
}

/// Migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub step_number: u32,
    pub phase: String,
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub verification: Vec<String>,
    pub rollback_info: Option<String>,
    pub estimated_time: std::time::Duration,
    pub dependencies: Vec<u32>,
}

/// Post-migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMigrationStep {
    pub step_number: u32,
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub verification: Vec<String>,
    pub optional: bool,
    pub benefits: String,
}

/// Troubleshooting tip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleshootingTip {
    pub issue: String,
    pub symptoms: Vec<String>,
    pub solutions: Vec<String>,
    pub prevention: Option<String>,
    pub severity: TroubleshootingSeverity,
}

/// Troubleshooting severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TroubleshootingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Migration timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub preparation_time: std::time::Duration,
    pub migration_time: std::time::Duration,
    pub verification_time: std::time::Duration,
    pub total_time: std::time::Duration,
    pub milestones: Vec<Milestone>,
}

/// Timeline milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub description: String,
    pub completion_criteria: Vec<String>,
    pub estimated_completion: std::time::Duration,
}

/// Risk assessment for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risks: Vec<IdentifiedRisk>,
    pub mitigation_strategies: Vec<MitigationStrategy>,
    pub contingency_plans: Vec<ContingencyPlan>,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Identified risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedRisk {
    pub risk_type: RiskType,
    pub description: String,
    pub probability: f32,
    pub impact: RiskImpact,
    pub mitigation: String,
}

/// Risk types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    DataLoss,
    ServiceDowntime,
    ConfigurationLoss,
    PerformanceDegradation,
    CompatibilityIssue,
    UserExperience,
}

/// Risk impact levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskImpact {
    Minimal,
    Low,
    Medium,
    High,
    Severe,
}

/// Mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub strategy_name: String,
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub effectiveness: f32,
}

/// Contingency plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContingencyPlan {
    pub scenario: String,
    pub trigger_conditions: Vec<String>,
    pub response_steps: Vec<String>,
    pub recovery_time: std::time::Duration,
}

/// Generate personalized migration guide
pub async fn generate_migration_guide(
    analysis: &TypeScriptAnalysis,
    config: &MigrationConfig,
) -> Result<MigrationGuide, HiveError> {
    log::info!("Generating personalized migration guide");
    
    // Determine migration scenario
    let scenario = determine_migration_scenario(analysis, config).await?;
    
    // Recommend approach based on scenario and analysis
    let recommended_approach = recommend_migration_approach(&scenario, analysis, config).await?;
    
    // Generate preparation steps
    let preparation_steps = generate_preparation_steps(&scenario, &recommended_approach, analysis).await?;
    
    // Generate migration steps
    let migration_steps = generate_migration_steps(&recommended_approach, config).await?;
    
    // Generate post-migration steps
    let post_migration_steps = generate_post_migration_steps(&scenario).await?;
    
    // Generate troubleshooting tips
    let troubleshooting = generate_troubleshooting_tips(&scenario, analysis).await?;
    
    // Estimate timeline
    let estimated_timeline = estimate_migration_timeline(&migration_steps, &preparation_steps).await?;
    
    // Assess risks
    let risk_assessment = assess_migration_risks(analysis, config, &scenario).await?;
    
    Ok(MigrationGuide {
        scenario,
        recommended_approach,
        preparation_steps,
        migration_steps,
        post_migration_steps,
        troubleshooting,
        estimated_timeline,
        risk_assessment,
    })
}

/// Determine migration scenario based on analysis
async fn determine_migration_scenario(
    analysis: &TypeScriptAnalysis,
    config: &MigrationConfig,
) -> Result<MigrationScenario, HiveError> {
    // Analyze usage patterns to determine scenario
    let conversation_count = analysis.database_info.conversation_count;
    let custom_profiles = analysis.custom_profiles.len();
    let has_enterprise_features = analysis.config_files.len() > 3;
    
    let scenario = if conversation_count == 0 {
        MigrationScenario::FirstTimeUser
    } else if conversation_count < 50 && custom_profiles == 0 {
        MigrationScenario::ExistingLightUser
    } else if conversation_count > 500 || custom_profiles > 5 {
        MigrationScenario::ExistingHeavyUser
    } else if has_enterprise_features {
        MigrationScenario::EnterpriseUser
    } else if config.migration_type == MigrationType::Staged {
        MigrationScenario::TeamMigration
    } else {
        MigrationScenario::DeveloperUser
    };
    
    log::info!("Determined migration scenario: {:?}", scenario);
    Ok(scenario)
}

/// Recommend migration approach
async fn recommend_migration_approach(
    scenario: &MigrationScenario,
    analysis: &TypeScriptAnalysis,
    config: &MigrationConfig,
) -> Result<MigrationApproach, HiveError> {
    let approach = match scenario {
        MigrationScenario::FirstTimeUser => MigrationApproach::QuickMigration,
        MigrationScenario::ExistingLightUser => {
            if analysis.migration_readiness() > 0.8 {
                MigrationApproach::QuickMigration
            } else {
                MigrationApproach::SafeMigration
            }
        },
        MigrationScenario::ExistingHeavyUser => {
            if config.preserve_original {
                MigrationApproach::ParallelMigration
            } else {
                MigrationApproach::SafeMigration
            }
        },
        MigrationScenario::EnterpriseUser => MigrationApproach::StagedMigration,
        MigrationScenario::TeamMigration => MigrationApproach::StagedMigration,
        MigrationScenario::DeveloperUser => MigrationApproach::CustomMigration,
    };
    
    log::info!("Recommended migration approach: {:?}", approach);
    Ok(approach)
}

/// Generate preparation steps
async fn generate_preparation_steps(
    scenario: &MigrationScenario,
    approach: &MigrationApproach,
    analysis: &TypeScriptAnalysis,
) -> Result<Vec<PreparationStep>, HiveError> {
    let mut steps = Vec::new();
    
    // Common preparation steps
    steps.push(PreparationStep {
        step_number: 1,
        title: "Backup Existing Installation".to_string(),
        description: "Create a complete backup of your TypeScript Hive AI installation".to_string(),
        commands: vec![
            "hive backup create --full".to_string(),
            "cp -r ~/.hive ~/.hive.backup".to_string(),
        ],
        expected_outcome: "Backup created successfully".to_string(),
        troubleshooting: Some("If backup fails, check disk space and permissions".to_string()),
        estimated_time: std::time::Duration::from_secs(300),
        required: true,
    });
    
    steps.push(PreparationStep {
        step_number: 2,
        title: "Verify System Requirements".to_string(),
        description: "Ensure your system meets requirements for Rust Hive AI".to_string(),
        commands: vec![
            "hive migrate analyze".to_string(),
            "hive migrate preview".to_string(),
        ],
        expected_outcome: "System requirements satisfied".to_string(),
        troubleshooting: Some("Install missing dependencies or update system".to_string()),
        estimated_time: std::time::Duration::from_secs(120),
        required: true,
    });
    
    // Scenario-specific steps
    match scenario {
        MigrationScenario::ExistingHeavyUser | MigrationScenario::EnterpriseUser => {
            steps.push(PreparationStep {
                step_number: 3,
                title: "Export Custom Configurations".to_string(),
                description: "Export custom profiles and advanced configurations".to_string(),
                commands: vec![
                    "hive profiles export --all".to_string(),
                    "hive config export --include-secrets".to_string(),
                ],
                expected_outcome: "Configurations exported".to_string(),
                troubleshooting: Some("Manually backup configuration files if export fails".to_string()),
                estimated_time: std::time::Duration::from_secs(180),
                required: true,
            });
        },
        _ => {}
    }
    
    // Approach-specific steps
    match approach {
        MigrationApproach::ParallelMigration => {
            steps.push(PreparationStep {
                step_number: steps.len() as u32 + 1,
                title: "Prepare Parallel Environment".to_string(),
                description: "Set up environment to run both TypeScript and Rust versions".to_string(),
                commands: vec![
                    "hive migrate setup-parallel".to_string(),
                ],
                expected_outcome: "Parallel environment ready".to_string(),
                troubleshooting: Some("Ensure sufficient disk space for both installations".to_string()),
                estimated_time: std::time::Duration::from_secs(240),
                required: true,
            });
        },
        _ => {}
    }
    
    Ok(steps)
}

/// Generate migration steps
async fn generate_migration_steps(
    approach: &MigrationApproach,
    config: &MigrationConfig,
) -> Result<Vec<MigrationStep>, HiveError> {
    let mut steps = Vec::new();
    
    match approach {
        MigrationApproach::QuickMigration => {
            steps.push(MigrationStep {
                step_number: 1,
                phase: "Automated Migration".to_string(),
                title: "Run Automated Migration".to_string(),
                description: "Execute automated migration process".to_string(),
                commands: vec!["hive migrate full --auto".to_string()],
                verification: vec!["hive status".to_string(), "hive memory stats".to_string()],
                rollback_info: Some("Use 'hive migrate rollback' if issues occur".to_string()),
                estimated_time: std::time::Duration::from_secs(300),
                dependencies: Vec::new(),
            });
        },
        MigrationApproach::SafeMigration => {
            steps.extend(generate_safe_migration_steps().await?);
        },
        MigrationApproach::ParallelMigration => {
            steps.extend(generate_parallel_migration_steps().await?);
        },
        MigrationApproach::StagedMigration => {
            steps.extend(generate_staged_migration_steps().await?);
        },
        MigrationApproach::CustomMigration => {
            steps.extend(generate_custom_migration_steps(config).await?);
        },
    }
    
    Ok(steps)
}

/// Generate safe migration steps
async fn generate_safe_migration_steps() -> Result<Vec<MigrationStep>, HiveError> {
    Ok(vec![
        MigrationStep {
            step_number: 1,
            phase: "Configuration".to_string(),
            title: "Migrate Configuration".to_string(),
            description: "Convert TypeScript configuration to Rust format".to_string(),
            commands: vec!["hive migrate config --verify".to_string()],
            verification: vec!["hive config validate".to_string()],
            rollback_info: Some("Configuration backup available".to_string()),
            estimated_time: std::time::Duration::from_secs(120),
            dependencies: Vec::new(),
        },
        MigrationStep {
            step_number: 2,
            phase: "Database".to_string(),
            title: "Migrate Database".to_string(),
            description: "Transfer conversation data and history".to_string(),
            commands: vec!["hive migrate database --verify".to_string()],
            verification: vec!["hive migrate verify --database".to_string()],
            rollback_info: Some("Database backup available".to_string()),
            estimated_time: std::time::Duration::from_secs(600),
            dependencies: vec![1],
        },
        MigrationStep {
            step_number: 3,
            phase: "Validation".to_string(),
            title: "Verify Migration".to_string(),
            description: "Comprehensive validation of migrated data".to_string(),
            commands: vec!["hive migrate verify --comprehensive".to_string()],
            verification: vec!["hive test --integration".to_string()],
            rollback_info: Some("Full rollback available if validation fails".to_string()),
            estimated_time: std::time::Duration::from_secs(300),
            dependencies: vec![1, 2],
        },
    ])
}

/// Generate parallel migration steps
async fn generate_parallel_migration_steps() -> Result<Vec<MigrationStep>, HiveError> {
    Ok(vec![
        MigrationStep {
            step_number: 1,
            phase: "Setup".to_string(),
            title: "Initialize Rust Installation".to_string(),
            description: "Install Rust version alongside TypeScript".to_string(),
            commands: vec!["hive-rust install --parallel".to_string()],
            verification: vec!["hive-rust --version".to_string()],
            rollback_info: Some("Can remove Rust installation".to_string()),
            estimated_time: std::time::Duration::from_secs(180),
            dependencies: Vec::new(),
        },
        MigrationStep {
            step_number: 2,
            phase: "Migration".to_string(),
            title: "Migrate Data to Rust Version".to_string(),
            description: "Copy data to Rust installation".to_string(),
            commands: vec!["hive-rust migrate import --from-typescript".to_string()],
            verification: vec!["hive-rust memory stats".to_string()],
            rollback_info: Some("TypeScript version remains untouched".to_string()),
            estimated_time: std::time::Duration::from_secs(420),
            dependencies: vec![1],
        },
        MigrationStep {
            step_number: 3,
            phase: "Testing".to_string(),
            title: "Test Both Versions".to_string(),
            description: "Verify both versions work correctly".to_string(),
            commands: vec![
                "hive ask 'Test question'".to_string(),
                "hive-rust ask 'Test question'".to_string(),
            ],
            verification: vec!["Compare responses for consistency".to_string()],
            rollback_info: Some("Continue using TypeScript if issues found".to_string()),
            estimated_time: std::time::Duration::from_secs(300),
            dependencies: vec![1, 2],
        },
    ])
}

/// Generate staged migration steps
async fn generate_staged_migration_steps() -> Result<Vec<MigrationStep>, HiveError> {
    Ok(vec![
        MigrationStep {
            step_number: 1,
            phase: "Phase 1".to_string(),
            title: "Migrate Core Configuration".to_string(),
            description: "Start with basic configuration migration".to_string(),
            commands: vec!["hive migrate config --stage=1".to_string()],
            verification: vec!["hive config validate".to_string()],
            rollback_info: Some("Phase 1 rollback available".to_string()),
            estimated_time: std::time::Duration::from_secs(180),
            dependencies: Vec::new(),
        },
        MigrationStep {
            step_number: 2,
            phase: "Phase 2".to_string(),
            title: "Migrate Essential Data".to_string(),
            description: "Transfer recent conversations and profiles".to_string(),
            commands: vec!["hive migrate database --stage=2 --recent-only".to_string()],
            verification: vec!["hive memory stats --verify-recent".to_string()],
            rollback_info: Some("Phase 2 rollback available".to_string()),
            estimated_time: std::time::Duration::from_secs(300),
            dependencies: vec![1],
        },
        MigrationStep {
            step_number: 3,
            phase: "Phase 3".to_string(),
            title: "Migrate Historical Data".to_string(),
            description: "Transfer complete conversation history".to_string(),
            commands: vec!["hive migrate database --stage=3 --historical".to_string()],
            verification: vec!["hive migrate verify --complete".to_string()],
            rollback_info: Some("Complete rollback available".to_string()),
            estimated_time: std::time::Duration::from_secs(900),
            dependencies: vec![1, 2],
        },
    ])
}

/// Generate custom migration steps
async fn generate_custom_migration_steps(config: &MigrationConfig) -> Result<Vec<MigrationStep>, HiveError> {
    let mut steps = Vec::new();
    
    // Customize based on configuration
    if config.validation_level == ValidationLevel::Paranoid {
        steps.push(MigrationStep {
            step_number: 1,
            phase: "Validation".to_string(),
            title: "Pre-Migration Validation".to_string(),
            description: "Extensive pre-migration testing".to_string(),
            commands: vec!["hive migrate validate --paranoid".to_string()],
            verification: vec!["Review validation report".to_string()],
            rollback_info: Some("No changes made yet".to_string()),
            estimated_time: std::time::Duration::from_secs(600),
            dependencies: Vec::new(),
        });
    }
    
    steps.push(MigrationStep {
        step_number: steps.len() as u32 + 1,
        phase: "Custom Migration".to_string(),
        title: "Execute Custom Migration Plan".to_string(),
        description: "Run migration with custom parameters".to_string(),
        commands: vec!["hive migrate custom --config=migration_plan.toml".to_string()],
        verification: vec!["hive migrate verify --custom".to_string()],
        rollback_info: Some("Custom rollback plan available".to_string()),
        estimated_time: std::time::Duration::from_secs(480),
        dependencies: if steps.is_empty() { Vec::new() } else { vec![1] },
    });
    
    Ok(steps)
}

/// Generate post-migration steps
async fn generate_post_migration_steps(scenario: &MigrationScenario) -> Result<Vec<PostMigrationStep>, HiveError> {
    let mut steps = vec![
        PostMigrationStep {
            step_number: 1,
            title: "Verify Migration Success".to_string(),
            description: "Confirm all data and configurations migrated correctly".to_string(),
            commands: vec![
                "hive status".to_string(),
                "hive memory stats".to_string(),
                "hive test --integration".to_string(),
            ],
            verification: vec!["All commands should complete successfully".to_string()],
            optional: false,
            benefits: "Ensures migration completed without data loss".to_string(),
        },
        PostMigrationStep {
            step_number: 2,
            title: "Update Shell Configuration".to_string(),
            description: "Update shell aliases and PATH if needed".to_string(),
            commands: vec![
                "hive completions bash > ~/.hive_completions".to_string(),
                "source ~/.hive_completions".to_string(),
            ],
            verification: vec!["Test tab completion with 'hive <TAB>'".to_string()],
            optional: true,
            benefits: "Improved command-line experience".to_string(),
        },
        PostMigrationStep {
            step_number: 3,
            title: "Performance Optimization".to_string(),
            description: "Configure performance settings for your usage".to_string(),
            commands: vec!["hive config optimize".to_string()],
            verification: vec!["hive benchmark --quick".to_string()],
            optional: true,
            benefits: "Optimized performance for your hardware".to_string(),
        },
    ];
    
    // Add scenario-specific post-migration steps
    match scenario {
        MigrationScenario::EnterpriseUser | MigrationScenario::TeamMigration => {
            steps.push(PostMigrationStep {
                step_number: 4,
                title: "Setup Team Features".to_string(),
                description: "Configure advanced features for team usage".to_string(),
                commands: vec![
                    "hive config set analytics.enabled=true".to_string(),
                    "hive hooks enable --team-features".to_string(),
                ],
                verification: vec!["hive analytics status".to_string()],
                optional: true,
                benefits: "Enhanced team collaboration and analytics".to_string(),
            });
        },
        _ => {}
    }
    
    Ok(steps)
}

/// Generate troubleshooting tips
async fn generate_troubleshooting_tips(
    scenario: &MigrationScenario,
    analysis: &TypeScriptAnalysis,
) -> Result<Vec<TroubleshootingTip>, HiveError> {
    let mut tips = vec![
        TroubleshootingTip {
            issue: "Migration fails with permission error".to_string(),
            symptoms: vec![
                "Permission denied when accessing files".to_string(),
                "Cannot create backup directory".to_string(),
            ],
            solutions: vec![
                "Check file permissions: chmod 755 ~/.hive".to_string(),
                "Ensure sufficient disk space".to_string(),
                "Run with appropriate privileges".to_string(),
            ],
            prevention: Some("Set correct permissions before migration".to_string()),
            severity: TroubleshootingSeverity::High,
        },
        TroubleshootingTip {
            issue: "Database migration fails".to_string(),
            symptoms: vec![
                "Database corruption detected".to_string(),
                "Row count mismatch".to_string(),
            ],
            solutions: vec![
                "Run database integrity check".to_string(),
                "Restore from backup and retry".to_string(),
                "Use manual data export/import".to_string(),
            ],
            prevention: Some("Regular database backups".to_string()),
            severity: TroubleshootingSeverity::Critical,
        },
    ];
    
    // Add scenario-specific tips
    match scenario {
        MigrationScenario::ExistingHeavyUser => {
            tips.push(TroubleshootingTip {
                issue: "Migration takes too long".to_string(),
                symptoms: vec!["Migration process exceeds estimated time".to_string()],
                solutions: vec![
                    "Use staged migration approach".to_string(),
                    "Migrate in smaller batches".to_string(),
                    "Check system resources".to_string(),
                ],
                prevention: Some("Estimate time based on data volume".to_string()),
                severity: TroubleshootingSeverity::Medium,
            });
        },
        _ => {}
    }
    
    // Add analysis-specific tips
    if !analysis.database_info.integrity_check {
        tips.push(TroubleshootingTip {
            issue: "Source database integrity issues".to_string(),
            symptoms: vec!["Database corruption in source".to_string()],
            solutions: vec![
                "Repair database before migration".to_string(),
                "Export/import specific tables".to_string(),
                "Contact support for assistance".to_string(),
            ],
            prevention: Some("Regular database maintenance".to_string()),
            severity: TroubleshootingSeverity::Critical,
        });
    }
    
    Ok(tips)
}

/// Estimate migration timeline
async fn estimate_migration_timeline(
    migration_steps: &[MigrationStep],
    preparation_steps: &[PreparationStep],
) -> Result<Timeline, HiveError> {
    let preparation_time: std::time::Duration = preparation_steps.iter()
        .map(|s| s.estimated_time)
        .sum();
    
    let migration_time: std::time::Duration = migration_steps.iter()
        .map(|s| s.estimated_time)
        .sum();
    
    let verification_time = std::time::Duration::from_secs(300); // 5 minutes
    let total_time = preparation_time + migration_time + verification_time;
    
    let milestones = vec![
        Milestone {
            name: "Preparation Complete".to_string(),
            description: "All preparation steps finished".to_string(),
            completion_criteria: vec!["Backups created".to_string(), "Requirements verified".to_string()],
            estimated_completion: preparation_time,
        },
        Milestone {
            name: "Migration Complete".to_string(),
            description: "Data migration finished".to_string(),
            completion_criteria: vec!["All data transferred".to_string(), "Configuration migrated".to_string()],
            estimated_completion: preparation_time + migration_time,
        },
        Milestone {
            name: "Verification Complete".to_string(),
            description: "Migration validated successfully".to_string(),
            completion_criteria: vec!["All checks passed".to_string(), "System operational".to_string()],
            estimated_completion: total_time,
        },
    ];
    
    Ok(Timeline {
        preparation_time,
        migration_time,
        verification_time,
        total_time,
        milestones,
    })
}

/// Assess migration risks
async fn assess_migration_risks(
    analysis: &TypeScriptAnalysis,
    config: &MigrationConfig,
    scenario: &MigrationScenario,
) -> Result<RiskAssessment, HiveError> {
    let mut risks = Vec::new();
    
    // Data loss risk
    if !config.preserve_original {
        risks.push(IdentifiedRisk {
            risk_type: RiskType::DataLoss,
            description: "Original data will be replaced".to_string(),
            probability: 0.1,
            impact: RiskImpact::High,
            mitigation: "Create comprehensive backups".to_string(),
        });
    }
    
    // Database integrity risk
    if !analysis.database_info.integrity_check {
        risks.push(IdentifiedRisk {
            risk_type: RiskType::DataLoss,
            description: "Source database has integrity issues".to_string(),
            probability: 0.8,
            impact: RiskImpact::Severe,
            mitigation: "Repair database before migration".to_string(),
        });
    }
    
    // Configuration loss risk
    if analysis.custom_profiles.len() > 5 {
        risks.push(IdentifiedRisk {
            risk_type: RiskType::ConfigurationLoss,
            description: "Complex custom configurations may not migrate perfectly".to_string(),
            probability: 0.3,
            impact: RiskImpact::Medium,
            mitigation: "Manual verification of custom profiles".to_string(),
        });
    }
    
    // Service downtime risk
    match scenario {
        MigrationScenario::EnterpriseUser | MigrationScenario::TeamMigration => {
            risks.push(IdentifiedRisk {
                risk_type: RiskType::ServiceDowntime,
                description: "Migration may cause temporary service interruption".to_string(),
                probability: 0.6,
                impact: RiskImpact::Medium,
                mitigation: "Use parallel migration approach".to_string(),
            });
        },
        _ => {}
    }
    
    // Determine overall risk level
    let max_impact = risks.iter()
        .map(|r| match r.impact {
            RiskImpact::Minimal => 1,
            RiskImpact::Low => 2,
            RiskImpact::Medium => 3,
            RiskImpact::High => 4,
            RiskImpact::Severe => 5,
        })
        .max()
        .unwrap_or(1);
    
    let overall_risk = match max_impact {
        1 => RiskLevel::Low,
        2 => RiskLevel::Low,
        3 => RiskLevel::Medium,
        4 => RiskLevel::High,
        5 => RiskLevel::Critical,
        _ => RiskLevel::Low,
    };
    
    let mitigation_strategies = vec![
        MitigationStrategy {
            strategy_name: "Comprehensive Backup".to_string(),
            description: "Create full backup before any changes".to_string(),
            implementation_steps: vec![
                "Backup database".to_string(),
                "Backup configuration".to_string(),
                "Test backup restore".to_string(),
            ],
            effectiveness: 0.9,
        },
        MitigationStrategy {
            strategy_name: "Staged Approach".to_string(),
            description: "Migrate in stages to reduce risk".to_string(),
            implementation_steps: vec![
                "Start with configuration".to_string(),
                "Migrate recent data first".to_string(),
                "Full migration after validation".to_string(),
            ],
            effectiveness: 0.8,
        },
    ];
    
    let contingency_plans = vec![
        ContingencyPlan {
            scenario: "Migration failure".to_string(),
            trigger_conditions: vec!["Data corruption detected".to_string(), "Validation fails".to_string()],
            response_steps: vec![
                "Stop migration immediately".to_string(),
                "Restore from backup".to_string(),
                "Investigate root cause".to_string(),
                "Plan alternative approach".to_string(),
            ],
            recovery_time: std::time::Duration::from_secs(1800), // 30 minutes
        },
    ];
    
    Ok(RiskAssessment {
        overall_risk,
        risks,
        mitigation_strategies,
        contingency_plans,
    })
}

/// Display migration guide in a user-friendly format
pub fn display_migration_guide(guide: &MigrationGuide) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("# Hive AI Migration Guide\n\n"));
    output.push_str(&format!("**Scenario**: {:?}\n", guide.scenario));
    output.push_str(&format!("**Recommended Approach**: {:?}\n", guide.recommended_approach));
    output.push_str(&format!("**Estimated Time**: {:?}\n\n", guide.estimated_timeline.total_time));
    
    output.push_str("## Risk Assessment\n");
    output.push_str(&format!("**Overall Risk**: {:?}\n", guide.risk_assessment.overall_risk));
    for risk in &guide.risk_assessment.risks {
        output.push_str(&format!("- {}: {} (Probability: {:.1}%)\n", 
            format!("{:?}", risk.risk_type), risk.description, risk.probability * 100.0));
    }
    output.push_str("\n");
    
    output.push_str("## Preparation Steps\n");
    for step in &guide.preparation_steps {
        output.push_str(&format!("{}. **{}** ({:?})\n", 
            step.step_number, step.title, step.estimated_time));
        output.push_str(&format!("   {}\n", step.description));
        for command in &step.commands {
            output.push_str(&format!("   ```\n   {}\n   ```\n", command));
        }
        output.push_str("\n");
    }
    
    output.push_str("## Migration Steps\n");
    for step in &guide.migration_steps {
        output.push_str(&format!("{}. **{}** - {} ({:?})\n", 
            step.step_number, step.title, step.phase, step.estimated_time));
        output.push_str(&format!("   {}\n", step.description));
        for command in &step.commands {
            output.push_str(&format!("   ```\n   {}\n   ```\n", command));
        }
        if let Some(rollback) = &step.rollback_info {
            output.push_str(&format!("   **Rollback**: {}\n", rollback));
        }
        output.push_str("\n");
    }
    
    output.push_str("## Post-Migration Steps\n");
    for step in &guide.post_migration_steps {
        let optional = if step.optional { " (Optional)" } else { "" };
        output.push_str(&format!("{}. **{}**{}\n", 
            step.step_number, step.title, optional));
        output.push_str(&format!("   {}\n", step.description));
        output.push_str(&format!("   **Benefits**: {}\n", step.benefits));
        output.push_str("\n");
    }
    
    output.push_str("## Troubleshooting\n");
    for tip in &guide.troubleshooting {
        output.push_str(&format!("**{}** ({:?})\n", tip.issue, tip.severity));
        output.push_str("   Symptoms:\n");
        for symptom in &tip.symptoms {
            output.push_str(&format!("   - {}\n", symptom));
        }
        output.push_str("   Solutions:\n");
        for solution in &tip.solutions {
            output.push_str(&format!("   - {}\n", solution));
        }
        if let Some(prevention) = &tip.prevention {
            output.push_str(&format!("   Prevention: {}\n", prevention));
        }
        output.push_str("\n");
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_migration_guide() {
        let guide = MigrationGuide {
            scenario: MigrationScenario::ExistingLightUser,
            recommended_approach: MigrationApproach::QuickMigration,
            preparation_steps: vec![
                PreparationStep {
                    step_number: 1,
                    title: "Test Step".to_string(),
                    description: "Test description".to_string(),
                    commands: vec!["test command".to_string()],
                    expected_outcome: "Test outcome".to_string(),
                    troubleshooting: None,
                    estimated_time: std::time::Duration::from_secs(60),
                    required: true,
                }
            ],
            migration_steps: Vec::new(),
            post_migration_steps: Vec::new(),
            troubleshooting: Vec::new(),
            estimated_timeline: Timeline {
                preparation_time: std::time::Duration::from_secs(60),
                migration_time: std::time::Duration::from_secs(120),
                verification_time: std::time::Duration::from_secs(30),
                total_time: std::time::Duration::from_secs(210),
                milestones: Vec::new(),
            },
            risk_assessment: RiskAssessment {
                overall_risk: RiskLevel::Low,
                risks: Vec::new(),
                mitigation_strategies: Vec::new(),
                contingency_plans: Vec::new(),
            },
        };
        
        let output = display_migration_guide(&guide);
        assert!(output.contains("Hive AI Migration Guide"));
        assert!(output.contains("ExistingLightUser"));
        assert!(output.contains("Test Step"));
    }
}