//! Hive AI: Lightning-fast codebase intelligence platform
//! 
//! Complete Rust reimplementation with 100% feature parity plus revolutionary enhancements.

use clap::{Parser, Subcommand};
use hive_ai::tui::TuiFramework;
use hive_ai::cli::CliFramework;
use hive_ai::desktop::launch_desktop_app;
use hive_ai::core::config::{Config, load_config};
use hive_ai::core::error::HiveResult;
use hive_ai::cli::banner::show_startup_banner;
use hive_ai::cli::tui_capabilities::TuiCapabilities;
use hive_ai::core::initialize_default_logging;
use std::path::PathBuf;
use std::env;
use anyhow::Result;

/// UI Mode for the application
#[derive(Debug, Clone, PartialEq)]
enum UIMode {
    CLI,
    TUI,
    Desktop,
}

#[derive(Parser)]
#[command(name = "hive")]
#[command(version = hive_ai::VERSION)]
#[command(about = "Lightning-fast codebase intelligence platform")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable debug mode
    #[arg(short, long, global = true)]
    debug: bool,
    
    /// Force TUI mode
    #[arg(long, global = true)]
    tui: bool,
    
    /// Force CLI mode (disable TUI)
    #[arg(long, global = true)]
    no_tui: bool,
    
    /// Force Desktop GUI mode
    #[arg(long, global = true)]
    desktop: bool,
    
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    
    /// Working directory
    #[arg(short = 'D', long, global = true)]
    directory: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Ask a question to the AI
    Ask {
        /// The question to ask
        query: String,
        
        /// Mode to use (planning, execution, hybrid)
        #[arg(short, long)]
        mode: Option<String>,
    },
    
    /// Start interactive planning session
    Plan {
        /// Task description
        task: String,
        
        /// Include repository analysis
        #[arg(long)]
        repository: bool,
    },
    
    /// Execute a plan
    Execute {
        /// Plan ID to execute
        plan_id: String,
        
        /// Validate dependencies
        #[arg(long)]
        validate: bool,
    },
    
    /// Analyze repository
    Analyze {
        /// Path to analyze
        path: Option<PathBuf>,
        
        /// Analysis depth
        #[arg(short, long, default_value = "full")]
        depth: String,
    },
    
    /// Manage consensus settings
    Consensus {
        #[command(subcommand)]
        command: ConsensusCommands,
    },
    
    /// Manage security settings
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },
    
    /// Manage enterprise hooks
    Hooks {
        #[command(subcommand)]
        command: HookCommands,
    },
    
    /// Show system status
    Status,
    
    /// Show configuration
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    
    /// Show memory and conversation history
    Memory {
        #[command(subcommand)]
        command: Option<MemoryCommands>,
    },
    
    /// Show analytics and insights
    Analytics {
        #[command(subcommand)]
        command: Option<AnalyticsCommands>,
    },
    
    /// Enterprise features
    Enterprise {
        #[command(subcommand)]
        command: EnterpriseCommands,
    },
    
    /// TUI mode
    Tui,
    
    /// Desktop GUI mode (Dioxus)
    Desktop,
    
    /// Installation management
    Install {
        #[command(flatten)]
        args: hive_ai::commands::install::InstallArgs,
    },
    
    /// Migration from TypeScript
    Migrate {
        #[command(flatten)]
        args: hive_ai::commands::migrate::MigrateArgs,
    },
    
    /// Show version information
    Version,
}

#[derive(Subcommand)]
enum ConsensusCommands {
    /// Test consensus engine
    Test {
        /// Test query
        query: String,
    },
    /// Show consensus statistics
    Stats,
    /// Configure consensus settings
    Config {
        /// Model configuration
        models: Option<String>,
    },
}

#[derive(Subcommand)]
enum SecurityCommands {
    /// Show security status
    Status,
    /// Manage trust decisions
    Trust {
        #[command(subcommand)]
        command: TrustCommands,
    },
    /// Manage users and roles
    Users {
        #[command(subcommand)]
        command: UserCommands,
    },
    /// Show audit logs
    Audit {
        /// Number of recent logs to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum TrustCommands {
    /// List trust decisions
    List,
    /// Trust a directory
    Grant {
        /// Directory path
        path: PathBuf,
        /// Trust scope
        #[arg(short, long, default_value = "recursive")]
        scope: String,
    },
    /// Revoke trust
    Revoke {
        /// Directory path
        path: PathBuf,
    },
    /// Clean up expired trust decisions
    Cleanup,
}

#[derive(Subcommand)]
enum UserCommands {
    /// List users
    List,
    /// Create user
    Create {
        /// Username
        username: String,
        /// Email
        email: String,
        /// Full name
        name: String,
    },
    /// Assign role
    Assign {
        /// User ID
        user_id: String,
        /// Role name
        role: String,
    },
}

#[derive(Subcommand)]
enum HookCommands {
    /// List hooks
    List,
    /// Register hook
    Register {
        /// Hook configuration file
        config: PathBuf,
    },
    /// Test hook
    Test {
        /// Hook configuration file
        config: PathBuf,
        /// Test event
        event: String,
    },
    /// Enable hook
    Enable {
        /// Hook ID
        hook_id: String,
    },
    /// Disable hook
    Disable {
        /// Hook ID
        hook_id: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show configuration
    Show,
    /// Set configuration value
    Set {
        /// Key
        key: String,
        /// Value
        value: String,
    },
    /// Reset configuration
    Reset,
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Show memory statistics
    Stats,
    /// Search conversations
    Search {
        /// Search query
        query: String,
    },
    /// Clear memory
    Clear {
        /// Confirmation
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum AnalyticsCommands {
    /// Show analytics dashboard
    Dashboard,
    /// Generate report
    Report {
        /// Report type
        #[arg(short, long, default_value = "summary")]
        type_: String,
    },
    /// Show trends
    Trends {
        /// Time period
        #[arg(short, long, default_value = "week")]
        period: String,
    },
}

#[derive(Subcommand)]
enum EnterpriseCommands {
    /// Show enterprise status
    Status,
    /// Manage teams
    Teams {
        #[command(subcommand)]
        command: TeamCommands,
    },
    /// Compliance reporting
    Compliance {
        /// Standard (SOX, GDPR, etc.)
        standard: String,
    },
    /// Cost management
    Cost {
        #[command(subcommand)]
        command: CostCommands,
    },
}

#[derive(Subcommand)]
enum TeamCommands {
    /// List teams
    List,
    /// Create team
    Create {
        /// Team name
        name: String,
        /// Team description
        description: String,
    },
    /// Add user to team
    Add {
        /// User ID
        user_id: String,
        /// Team name
        team: String,
    },
}

#[derive(Subcommand)]
enum CostCommands {
    /// Show cost estimation
    Estimate {
        /// Query to estimate
        query: String,
    },
    /// Show cost settings
    Settings,
    /// Set budget
    Budget {
        /// Budget amount
        amount: f64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    initialize_default_logging()?;
    
    let cli = Cli::parse();
    
    // Load configuration
    let config = Config::default();
    
    // Change directory if specified
    if let Some(dir) = &cli.directory {
        env::set_current_dir(dir)?;
    }
    
    // Handle version command first
    if matches!(cli.command, Some(Commands::Version)) {
        println!("Hive AI {}", hive_ai::VERSION);
        return Ok(());
    }
    
    // Detect capabilities
    let capabilities = TuiCapabilities::detect()?;
    
    // Determine UI mode
    let ui_mode = if cli.desktop {
        UIMode::Desktop
    } else if cli.tui {
        UIMode::TUI
    } else if cli.no_tui {
        UIMode::CLI
    } else {
        match &cli.command {
            Some(Commands::Desktop) => UIMode::Desktop,
            Some(Commands::Tui) => UIMode::TUI,
            None => {
                // Auto-detect best mode: prefer desktop > TUI > CLI
                if capabilities.supports_desktop() {
                    UIMode::Desktop
                } else if capabilities.supports_simple_tui() {
                    UIMode::TUI
                } else {
                    UIMode::CLI
                }
            },
            _ => UIMode::CLI,
        }
    };
    
    // Show startup banner if no command specified
    if cli.command.is_none() {
        show_startup_banner().await?;
    }
    
    // Initialize the appropriate framework
    match ui_mode {
        UIMode::Desktop => {
            tracing::info!("Launching desktop GUI mode");
            launch_desktop_app(config).await?;
        }
        UIMode::TUI => {
            tracing::info!("Launching TUI mode");
            let mut tui = TuiFramework::new().await?;
            tui.run().await?;
        }
        UIMode::CLI => {
            tracing::info!("Using CLI mode");
            let mut cli_framework = CliFramework::new(config).await?;
            
            match cli.command {
                Some(command) => {
                    execute_command(&mut cli_framework, command).await?;
                }
                None => {
                    // Interactive mode
                    cli_framework.run_interactive().await?;
                }
            }
        }
    }
    
    Ok(())
}

async fn execute_command(cli: &mut CliFramework, command: Commands) -> HiveResult<()> {
    match command {
        Commands::Ask { query, mode } => {
            cli.ask(&query, mode.as_deref()).await?;
        }
        Commands::Plan { task, repository } => {
            cli.plan(&task, repository).await?;
        }
        Commands::Execute { plan_id, validate } => {
            cli.execute_plan(&plan_id, validate).await?;
        }
        Commands::Analyze { path, depth } => {
            cli.analyze(path.as_deref(), &depth).await?;
        }
        Commands::Consensus { command } => {
            execute_consensus_command(cli, command).await?;
        }
        Commands::Security { command } => {
            execute_security_command(cli, command).await?;
        }
        Commands::Hooks { command } => {
            execute_hooks_command(cli, command).await?;
        }
        Commands::Status => {
            cli.show_status().await?;
        }
        Commands::Config { command } => {
            execute_config_command(cli, command).await?;
        }
        Commands::Memory { command } => {
            execute_memory_command(cli, command).await?;
        }
        Commands::Analytics { command } => {
            execute_analytics_command(cli, command).await?;
        }
        Commands::Enterprise { command } => {
            execute_enterprise_command(cli, command).await?;
        }
        Commands::Install { args } => {
            hive_ai::commands::install::handle_install_command(args).await
                .map_err(|e| hive_ai::core::error::HiveError::ConfigInvalid { message: e.to_string() })?;
        }
        Commands::Migrate { args } => {
            hive_ai::commands::migrate::handle_migrate(args).await?;
        }
        Commands::Tui => {
            // This should not happen as TUI is handled above
            unreachable!("TUI command should be handled before this point");
        }
        Commands::Version => {
            // This should not happen as version is handled above
            unreachable!("Version command should be handled before this point");
        }
        Commands::Desktop => {
            // Launch desktop application
            let config = load_config().await?; // Load default config
            launch_desktop_app(config).await?;
        }
    }
    Ok(())
}

async fn execute_consensus_command(cli: &mut CliFramework, command: ConsensusCommands) -> HiveResult<()> {
    match command {
        ConsensusCommands::Test { query } => {
            cli.test_consensus(&query).await?;
        }
        ConsensusCommands::Stats => {
            cli.show_consensus_stats().await?;
        }
        ConsensusCommands::Config { models } => {
            cli.configure_consensus(models.as_deref()).await?;
        }
    }
    Ok(())
}

async fn execute_security_command(cli: &mut CliFramework, command: SecurityCommands) -> HiveResult<()> {
    match command {
        SecurityCommands::Status => {
            cli.show_security_status().await?;
        }
        SecurityCommands::Trust { command } => {
            execute_trust_command(cli, command).await?;
        }
        SecurityCommands::Users { command } => {
            execute_user_command(cli, command).await?;
        }
        SecurityCommands::Audit { limit } => {
            cli.show_audit_logs(limit).await?;
        }
    }
    Ok(())
}

async fn execute_trust_command(cli: &mut CliFramework, command: TrustCommands) -> HiveResult<()> {
    match command {
        TrustCommands::List => {
            cli.list_trust_decisions().await?;
        }
        TrustCommands::Grant { path, scope } => {
            cli.grant_trust(&path, &scope).await?;
        }
        TrustCommands::Revoke { path } => {
            cli.revoke_trust(&path).await?;
        }
        TrustCommands::Cleanup => {
            cli.cleanup_trust().await?;
        }
    }
    Ok(())
}

async fn execute_user_command(cli: &mut CliFramework, command: UserCommands) -> HiveResult<()> {
    match command {
        UserCommands::List => {
            cli.list_users().await?;
        }
        UserCommands::Create { username, email, name } => {
            cli.create_user(&username, &email, &name).await?;
        }
        UserCommands::Assign { user_id, role } => {
            cli.assign_role(&user_id, &role).await?;
        }
    }
    Ok(())
}

async fn execute_hooks_command(cli: &mut CliFramework, command: HookCommands) -> HiveResult<()> {
    match command {
        HookCommands::List => {
            cli.list_hooks().await?;
        }
        HookCommands::Register { config } => {
            cli.register_hook(&config).await?;
        }
        HookCommands::Test { config, event } => {
            cli.test_hook(&config, &event).await?;
        }
        HookCommands::Enable { hook_id } => {
            cli.enable_hook(&hook_id).await?;
        }
        HookCommands::Disable { hook_id } => {
            cli.disable_hook(&hook_id).await?;
        }
    }
    Ok(())
}

async fn execute_config_command(cli: &mut CliFramework, command: Option<ConfigCommands>) -> HiveResult<()> {
    match command {
        Some(ConfigCommands::Show) | None => {
            cli.show_config().await?;
        }
        Some(ConfigCommands::Set { key, value }) => {
            cli.set_config(&key, &value).await?;
        }
        Some(ConfigCommands::Reset) => {
            cli.reset_config().await?;
        }
    }
    Ok(())
}

async fn execute_memory_command(cli: &mut CliFramework, command: Option<MemoryCommands>) -> HiveResult<()> {
    match command {
        Some(MemoryCommands::Stats) | None => {
            cli.show_memory_stats().await?;
        }
        Some(MemoryCommands::Search { query }) => {
            cli.search_memory(&query).await?;
        }
        Some(MemoryCommands::Clear { confirm }) => {
            cli.clear_memory(confirm).await?;
        }
    }
    Ok(())
}

async fn execute_analytics_command(cli: &mut CliFramework, command: Option<AnalyticsCommands>) -> HiveResult<()> {
    match command {
        Some(AnalyticsCommands::Dashboard) | None => {
            cli.show_analytics_dashboard().await?;
        }
        Some(AnalyticsCommands::Report { type_ }) => {
            cli.generate_analytics_report(&type_).await?;
        }
        Some(AnalyticsCommands::Trends { period }) => {
            cli.show_analytics_trends(&period).await?;
        }
    }
    Ok(())
}

async fn execute_enterprise_command(cli: &mut CliFramework, command: EnterpriseCommands) -> HiveResult<()> {
    match command {
        EnterpriseCommands::Status => {
            cli.show_enterprise_status().await?;
        }
        EnterpriseCommands::Teams { command } => {
            execute_team_command(cli, command).await?;
        }
        EnterpriseCommands::Compliance { standard } => {
            cli.show_compliance_report(&standard).await?;
        }
        EnterpriseCommands::Cost { command } => {
            execute_cost_command(cli, command).await?;
        }
    }
    Ok(())
}

async fn execute_team_command(cli: &mut CliFramework, command: TeamCommands) -> HiveResult<()> {
    match command {
        TeamCommands::List => {
            cli.list_teams().await?;
        }
        TeamCommands::Create { name, description } => {
            cli.create_team(&name, &description).await?;
        }
        TeamCommands::Add { user_id, team } => {
            cli.add_user_to_team(&user_id, &team).await?;
        }
    }
    Ok(())
}

async fn execute_cost_command(cli: &mut CliFramework, command: CostCommands) -> HiveResult<()> {
    match command {
        CostCommands::Estimate { query } => {
            cli.estimate_cost(&query).await?;
        }
        CostCommands::Settings => {
            cli.show_cost_settings().await?;
        }
        CostCommands::Budget { amount } => {
            cli.set_budget(amount).await?;
        }
    }
    Ok(())
}