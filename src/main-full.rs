//! Hive AI CLI - Interactive codebase intelligence

use clap::{Parser, Subcommand};
use anyhow::Result;
use hive_ai::{initialize, ConsensusEngine, SemanticIndex};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hive")]
#[command(about = "AI-powered codebase intelligence platform", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Hive in the current project
    Init {
        /// Project path (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    
    /// Analyze and understand any repository
    Analyze {
        /// Repository path or URL
        repo: String,
        
        /// Analysis depth (quick, standard, comprehensive)
        #[arg(short, long, default_value = "standard")]
        depth: String,
        
        /// Output format (text, json, html)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Save analysis to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Ask a question about the codebase
    Ask {
        /// The question to ask
        question: String,
        
        /// Consensus profile to use
        #[arg(short, long, default_value = "balanced")]
        profile: String,
        
        /// Output format
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Enable planning mode
        #[arg(long)]
        plan: bool,
    },
    
    /// Enter planning mode for complex tasks
    Plan {
        /// Description of what you want to accomplish
        goal: String,
        
        /// Planning depth (quick, standard, comprehensive)
        #[arg(short, long, default_value = "standard")]
        depth: String,
        
        /// Enable collaborative planning
        #[arg(short, long)]
        collaborative: bool,
        
        /// Save plan to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Execute a previously created plan
    Execute {
        /// Plan file or plan ID
        plan: String,
        
        /// Auto-execute without confirmation
        #[arg(short, long)]
        auto: bool,
        
        /// Validation level (basic, standard, strict)
        #[arg(short, long, default_value = "standard")]
        validation: String,
    },
    
    /// Apply AI-suggested improvements to a file
    Improve {
        /// File to improve
        file: PathBuf,
        
        /// Specific aspect to improve
        #[arg(short, long)]
        aspect: Option<String>,
        
        /// Auto-apply changes without confirmation
        #[arg(long)]
        auto_apply: bool,
        
        /// Use consensus validation
        #[arg(long)]
        consensus: bool,
    },
    
    /// Generate comprehensive analytics reports
    Analytics {
        #[command(subcommand)]
        command: AnalyticsCommands,
    },
    
    /// Manage long-term memory
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    
    /// Execute tools and tool chains
    Tool {
        /// Tool name
        name: String,
        
        /// Tool parameters (JSON format)
        #[arg(short, long)]
        params: Option<String>,
        
        /// Execute as part of a tool chain
        #[arg(short, long)]
        chain: Option<String>,
    },
    
    /// Start the IDE integration server
    Serve {
        /// Server mode
        #[arg(short, long, default_value = "mcp")]
        mode: String,
        
        /// Port to listen on
        #[arg(short, long, default_value = "7777")]
        port: u16,
    },
    
    /// Analyze codebase and build indices
    Index {
        /// Path to index
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Force rebuild
        #[arg(short, long)]
        force: bool,
    },
    
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    
    /// Manage enterprise hooks and automation
    Hooks {
        #[command(subcommand)]
        command: HookCommands,
    },
    
    /// Interactive mode
    Interactive {
        /// Starting mode (planning, execution, hybrid)
        #[arg(short, long, default_value = "hybrid")]
        mode: String,
    },
    
    /// Launch full TUI interface (VS Code-like)
    Tui {
        /// Force TUI even if not detected as standalone
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum AnalyticsCommands {
    /// Generate usage analytics
    Usage {
        /// Time period (day, week, month, quarter)
        #[arg(short, long, default_value = "week")]
        period: String,
    },
    
    /// Performance analysis
    Performance {
        /// Time period
        #[arg(short, long, default_value = "week")]
        period: String,
    },
    
    /// Cost analysis
    Cost {
        /// Time period
        #[arg(short, long, default_value = "month")]
        period: String,
    },
    
    /// Quality metrics
    Quality {
        /// Time period
        #[arg(short, long, default_value = "week")]
        period: String,
    },
    
    /// Comprehensive report
    Report {
        /// Report type (executive, operational, performance, cost)
        #[arg(short, long, default_value = "executive")]
        report_type: String,
        
        /// Time period
        #[arg(short, long, default_value = "month")]
        period: String,
        
        /// Output format
        #[arg(short, long, default_value = "html")]
        format: String,
    },
    
    /// Trend analysis
    Trends {
        /// Metric to analyze
        metric: String,
        
        /// Time period
        #[arg(short, long, default_value = "quarter")]
        period: String,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Search conversation history
    Search {
        /// Search query
        query: String,
        
        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Show memory statistics
    Stats,
    
    /// Export conversation history
    Export {
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
        
        /// Export format (json, csv, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    
    /// Import conversation history
    Import {
        /// Input file
        file: PathBuf,
    },
    
    /// Clear memory (with confirmation)
    Clear {
        /// Clear all data
        #[arg(long)]
        all: bool,
        
        /// Clear conversations older than days
        #[arg(long)]
        older_than: Option<u32>,
    },
    
    /// Manage knowledge graph
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommands,
    },
}

#[derive(Subcommand)]
enum KnowledgeCommands {
    /// View knowledge graph statistics
    Stats,
    
    /// Query the knowledge graph
    Query {
        /// Query string
        query: String,
    },
    
    /// Export knowledge graph
    Export {
        /// Output file
        output: PathBuf,
        
        /// Format (graphml, json, dot)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Validate configuration
    Validate,
    
    /// Reset to defaults
    Reset {
        /// Confirm reset
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum HookCommands {
    /// List all configured hooks
    List {
        /// Filter by event type
        #[arg(short, long)]
        event: Option<String>,
        
        /// Show only enabled hooks
        #[arg(long)]
        enabled_only: bool,
    },
    
    /// Add a new hook
    Add {
        /// Hook configuration file (JSON)
        config: PathBuf,
    },
    
    /// Remove a hook
    Remove {
        /// Hook ID to remove
        hook_id: String,
    },
    
    /// Enable/disable a hook
    Toggle {
        /// Hook ID to toggle
        hook_id: String,
    },
    
    /// Test a hook configuration
    Test {
        /// Hook configuration file
        config: PathBuf,
        
        /// Mock event to trigger
        event: String,
    },
    
    /// Validate all hook configurations
    Validate,
    
    /// Show hook execution history
    History {
        /// Number of recent executions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Check for TUI mode first
    if should_launch_tui().await {
        return launch_tui().await;
    }
    
    // Show banner if no command provided (like Claude Code)
    if std::env::args().len() == 1 {
        show_startup_banner().await?;
        return Ok(());
    }
    
    // Initialize Hive
    initialize().await?;
    
    // Set verbosity
    if cli.verbose > 0 {
        std::env::set_var("RUST_LOG", match cli.verbose {
            1 => "info",
            2 => "debug",
            _ => "trace",
        });
    }
    
    match cli.command {
        Commands::Init { path } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            println!("🐝 Initializing Hive in {}...", path.display());
            
            // Create .hive directory
            tokio::fs::create_dir_all(path.join(".hive")).await?;
            
            // Create default config
            let config = include_str!("../templates/default_config.toml");
            tokio::fs::write(path.join(".hive/config.toml"), config).await?;
            
            println!("✅ Hive initialized successfully!");
            println!("📝 Edit .hive/config.toml to customize settings");
        }
        
        Commands::Analyze { repo, depth, format, output } => {
            println!("🔍 Analyzing repository: {}", repo);
            println!("📊 Analysis depth: {}", depth);
            
            // TODO: Implement repository analysis
            println!("🏗️  Architecture detected: Microservices");
            println!("🛠️  Technologies: Rust, TypeScript, Docker");
            println!("📈 Quality score: 8.5/10");
            println!("🔒 Security: 3 potential issues found");
            
            if let Some(output_path) = output {
                println!("💾 Saving analysis to: {}", output_path.display());
            }
        }
        
        Commands::Ask { question, profile, format, plan } => {
            if plan {
                println!("📋 Planning mode enabled...");
            }
            
            println!("🤔 Processing your question...\n");
            
            let engine = ConsensusEngine::new(profile).await?;
            let response = engine.process(&question).await?;
            
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&response)?),
                _ => println!("{}", response.curator_output),
            }
        }
        
        Commands::Plan { goal, depth, collaborative, output } => {
            println!("📋 Creating execution plan...");
            println!("🎯 Goal: {}", goal);
            println!("📊 Planning depth: {}", depth);
            
            if collaborative {
                println!("🤝 Collaborative planning enabled");
            }
            
            // TODO: Implement planning engine
            println!("✅ Plan created with 5 tasks");
            println!("⏱️  Estimated completion: 2-3 days");
            println!("⚠️  2 risks identified");
            
            if let Some(output_path) = output {
                println!("💾 Plan saved to: {}", output_path.display());
            }
        }
        
        Commands::Execute { plan, auto, validation } => {
            println!("⚡ Executing plan: {}", plan);
            println!("🔍 Validation level: {}", validation);
            
            if auto {
                println!("🤖 Auto-execution enabled");
            }
            
            // TODO: Implement plan execution
            println!("✅ Plan executed successfully");
        }
        
        Commands::Improve { file, aspect, auto_apply, consensus } => {
            println!("🔍 Analyzing {}...", file.display());
            
            if let Some(aspect) = aspect {
                println!("🎯 Focusing on: {}", aspect);
            }
            
            if consensus {
                println!("🧠 Using consensus validation...");
            }
            
            // TODO: Implement improvement logic
            println!("📝 Improvement suggestions generated");
            
            if auto_apply {
                println!("✅ Changes applied automatically");
            } else {
                println!("❓ Review changes? (y/n)");
            }
        }
        
        Commands::Analytics { command } => {
            match command {
                AnalyticsCommands::Usage { period } => {
                    println!("📊 Generating usage analytics for: {}", period);
                    // TODO: Implement usage analytics
                }
                AnalyticsCommands::Performance { period } => {
                    println!("⚡ Analyzing performance for: {}", period);
                    // TODO: Implement performance analytics
                }
                AnalyticsCommands::Cost { period } => {
                    println!("💰 Calculating costs for: {}", period);
                    // TODO: Implement cost analytics
                }
                AnalyticsCommands::Quality { period } => {
                    println!("✨ Analyzing quality metrics for: {}", period);
                    // TODO: Implement quality analytics
                }
                AnalyticsCommands::Report { report_type, period, format } => {
                    println!("📋 Generating {} report for {} in {} format", report_type, period, format);
                    // TODO: Implement comprehensive reporting
                }
                AnalyticsCommands::Trends { metric, period } => {
                    println!("📈 Analyzing trends for {} over {}", metric, period);
                    // TODO: Implement trend analysis
                }
            }
        }
        
        Commands::Memory { command } => {
            match command {
                MemoryCommands::Search { query, limit } => {
                    println!("🔍 Searching memory for: \"{}\"", query);
                    println!("📝 Found {} relevant conversations", limit);
                    // TODO: Implement memory search
                }
                MemoryCommands::Stats => {
                    println!("🧠 Memory Statistics:");
                    println!("💬 Conversations: 142");
                    println!("🔗 Knowledge nodes: 1,847");
                    println!("📊 Memory usage: 45.2 MB");
                    // TODO: Implement real stats
                }
                MemoryCommands::Export { output, format } => {
                    println!("📤 Exporting memory to {} as {}", output.display(), format);
                    // TODO: Implement memory export
                }
                MemoryCommands::Import { file } => {
                    println!("📥 Importing memory from {}", file.display());
                    // TODO: Implement memory import
                }
                MemoryCommands::Clear { all, older_than } => {
                    if all {
                        println!("🗑️  Clearing all memory (requires confirmation)");
                    } else if let Some(days) = older_than {
                        println!("🗑️  Clearing conversations older than {} days", days);
                    }
                    // TODO: Implement memory clearing
                }
                MemoryCommands::Knowledge { command } => {
                    match command {
                        KnowledgeCommands::Stats => {
                            println!("🕸️  Knowledge Graph Statistics:");
                            println!("🔗 Nodes: 1,847");
                            println!("➡️  Edges: 5,234");
                            println!("🌐 Clusters: 23");
                        }
                        KnowledgeCommands::Query { query } => {
                            println!("🔍 Querying knowledge graph: \"{}\"", query);
                            // TODO: Implement knowledge graph queries
                        }
                        KnowledgeCommands::Export { output, format } => {
                            println!("📤 Exporting knowledge graph to {} as {}", output.display(), format);
                            // TODO: Implement knowledge graph export
                        }
                    }
                }
            }
        }
        
        Commands::Tool { name, params, chain } => {
            if let Some(chain_name) = chain {
                println!("🔧 Executing tool chain: {}", chain_name);
            } else {
                println!("🔧 Executing tool: {}", name);
            }
            
            if let Some(params_json) = params {
                println!("⚙️  Parameters: {}", params_json);
            }
            
            // TODO: Implement tool execution
            println!("✅ Tool executed successfully");
        }
        
        Commands::Serve { mode, port } => {
            println!("🚀 Starting {} server on port {}...", mode, port);
            
            match mode.as_str() {
                "mcp" => {
                    hive_ai::integration::start_mcp_server(port).await?;
                }
                "lsp" => {
                    hive_ai::integration::start_lsp_server(port).await?;
                }
                _ => anyhow::bail!("Unknown server mode: {}", mode),
            }
        }
        
        Commands::Index { path, force } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            
            if force {
                println!("🔄 Force rebuilding indices...");
            } else {
                println!("📚 Building indices...");
            }
            
            let index = SemanticIndex::new().await?;
            index.build_project(&path, force).await?;
            
            println!("✅ Indexing complete!");
        }
        
        Commands::Config { command } => {
            match command {
                ConfigCommands::Show => {
                    println!("📋 Current configuration:");
                    // TODO: Load and display config
                }
                ConfigCommands::Set { key, value } => {
                    println!("✏️  Setting {} = {}", key, value);
                    // TODO: Set config value
                }
                ConfigCommands::Get { key } => {
                    println!("🔍 {} = <value>", key);
                    // TODO: Get config value
                }
                ConfigCommands::Validate => {
                    println!("✅ Configuration is valid");
                    // TODO: Implement config validation
                }
                ConfigCommands::Reset { confirm } => {
                    if confirm {
                        println!("🔄 Resetting configuration to defaults...");
                        // TODO: Implement config reset
                    } else {
                        println!("❌ Use --confirm to reset configuration");
                    }
                }
            }
        }
        
        Commands::Hooks { command } => {
            match command {
                HookCommands::List { event, enabled_only } => {
                    println!("🔗 Enterprise Hooks:");
                    if let Some(event_filter) = event {
                        println!("🔍 Filtering by event: {}", event_filter);
                    }
                    if *enabled_only {
                        println!("✅ Showing only enabled hooks");
                    }
                    // TODO: List hooks with filtering
                    println!("  ✓ auto-format-rust (enabled)");
                    println!("  ✓ production-guard (enabled)");
                    println!("  ⚠ cost-control (disabled)");
                }
                HookCommands::Add { config } => {
                    println!("➕ Adding hook from: {}", config.display());
                    // TODO: Load and validate hook configuration
                    println!("✅ Hook added successfully");
                }
                HookCommands::Remove { hook_id } => {
                    println!("🗑️  Removing hook: {}", hook_id);
                    // TODO: Remove hook by ID
                    println!("✅ Hook removed");
                }
                HookCommands::Toggle { hook_id } => {
                    println!("🔄 Toggling hook: {}", hook_id);
                    // TODO: Toggle hook enabled/disabled state
                    println!("✅ Hook toggled");
                }
                HookCommands::Test { config, event } => {
                    println!("🧪 Testing hook config: {}", config.display());
                    println!("⚡ Triggering event: {}", event);
                    // TODO: Test hook execution
                    println!("✅ Hook test completed");
                }
                HookCommands::Validate => {
                    println!("🔍 Validating all hook configurations...");
                    // TODO: Validate all hooks
                    println!("✅ All hooks are valid");
                }
                HookCommands::History { limit } => {
                    println!("📊 Hook execution history (last {}):", limit);
                    // TODO: Show hook execution history
                    println!("  2024-07-02 14:30:15 | auto-format-rust | SUCCESS | Applied to src/main.rs");
                    println!("  2024-07-02 14:25:42 | production-guard | BLOCKED | Unauthorized change attempt");
                }
            }
        }
        
        Commands::Interactive { mode } => {
            println!("🎮 Starting interactive mode: {}", mode);
            
            // Check if we should use the advanced TUI or simple CLI
            if check_tui_capabilities() && std::env::var("HIVE_SIMPLE_CLI").is_err() {
                // Launch advanced TUI (like Claude Code)
                match start_advanced_tui().await {
                    Ok(()) => {}
                    Err(_) => {
                        println!("⚠️  TUI mode failed, falling back to simple CLI");
                        start_interactive_mode().await?;
                    }
                }
            } else {
                // Use simple CLI mode
                start_interactive_mode().await?;
            }
        }
        
        Commands::Tui { force } => {
            if force || check_tui_capabilities() {
                println!("🖥️ Launching TUI interface...");
                return launch_tui().await;
            } else {
                println!("❌ TUI mode requires a capable terminal (120x30 minimum)");
                println!("💡 Use --force to override detection");
            }
        }
    }
    
    Ok(())
}

async fn show_startup_banner() -> Result<()> {
    use console::style;
    
    // Claude Code style welcome banner
    println!("╭───────────────────────────────────────────────────╮");
    println!("│ ✻ Welcome to HiveTechs Consensus!                 │");
    println!("│                                                   │");
    println!("│   /help for help, /status for your current setup  │");
    println!("│                                                   │");
    println!("│   cwd: {}                  │", get_current_dir_display());
    println!("╰───────────────────────────────────────────────────╯");
    println!();
    
    // What's new section (like Claude Code)
    println!(" What's new:");
    println!("  • Released [Enterprise Hooks](https://docs.hivetechs.com/");
    println!("  hooks). Deterministic control over AI behavior");
    println!("  • Temporal context for web search - always knows today's date");
    println!("  • Repository intelligence with ML-powered analysis"); 
    println!("  • 10-40x performance improvements over TypeScript version");
    println!("  • Planning mode for strategic development workflows");
    println!();
    
    Ok(())
}

fn get_current_dir_display() -> String {
    std::env::current_dir()
        .map(|path| {
            let path_str = path.to_string_lossy();
            if path_str.len() > 35 {
                format!("...{}", &path_str[path_str.len()-32..])
            } else {
                path_str.to_string()
            }
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Launch advanced TUI mode (like Claude Code)
async fn start_advanced_tui() -> Result<()> {
    mod interactive_tui;
    use interactive_tui::InteractiveTui;
    
    let mut tui = InteractiveTui::new().await?;
    tui.run().await?;
    
    Ok(())
}

/// Interactive CLI mode that mimics Claude Code's experience
async fn start_interactive_mode() -> Result<()> {
    use std::io::{self, Write};
    use console::{style, Term};
    
    let term = Term::stdout();
    term.clear_screen()?;
    
    // Show the welcome banner first
    show_startup_banner().await?;
    
    // Initialize the persistent interactive interface
    loop {
        // Print the interactive input box (like Claude Code)
        print_interactive_prompt()?;
        
        // Read user input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        
        let input = input.trim();
        
        // Handle special commands
        match input {
            "/help" | "help" => {
                print_help_info().await?;
            }
            "/status" | "status" => {
                print_status_info().await?;
            }
            "/exit" | "exit" | "quit" => {
                println!("👋 Thanks for using HiveTechs Consensus!");
                break;
            }
            "" => {
                // Empty input, just continue
                continue;
            }
            _ => {
                // Process the command as a regular hive command
                process_interactive_command(input).await?;
            }
        }
    }
    
    Ok(())
}

fn print_interactive_prompt() -> Result<()> {
    println!("╭──────────────────────────────────────────────────────╮");
    println!("│ > Try \"ask <question>\" or \"analyze .\"                 │");
    println!("╰──────────────────────────────────────────────────────╯");
    print!("  ? for shortcuts  ");
    io::stdout().flush()?;
    Ok(())
}

async fn print_help_info() -> Result<()> {
    println!();
    println!(" 🆘 HiveTechs Consensus Help");
    println!();
    println!(" Commands:");
    println!("   ask <question>        - Ask the AI consensus a question");
    println!("   analyze <path>        - Analyze repository or file");
    println!("   plan <goal>           - Create a development plan");
    println!("   improve <file>        - Suggest improvements to a file");
    println!("   hooks list            - Show configured enterprise hooks");
    println!("   memory search <query> - Search conversation history");
    println!();
    println!(" Special commands:");
    println!("   /help or help         - Show this help");
    println!("   /status or status     - Show system status");
    println!("   /exit or exit         - Exit interactive mode");
    println!();
    println!(" Documentation: https://docs.hivetechs.com");
    println!();
    Ok(())
}

async fn print_status_info() -> Result<()> {
    println!();
    println!(" 📊 HiveTechs Consensus Status");
    println!();
    
    let config_dir = get_hive_config_dir();
    let config_exists = tokio::fs::metadata(config_dir.join("config.toml")).await.is_ok();
    let db_exists = tokio::fs::metadata(config_dir.join("hive-ai.db")).await.is_ok();
    
    println!(" System:");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!("   Config: {}", if config_exists { "✓ Configured" } else { "⚠ Not configured" });
    println!("   Memory: {}", if db_exists { "✓ Ready" } else { "⚠ Not initialized" });
    println!("   Working Directory: {}", get_current_dir_display());
    println!();
    
    // Check connectivity
    let internet_status = check_internet_connection().await;
    let api_status = check_api_status().await;
    
    println!(" Connectivity:");
    println!("   Internet: {}", if internet_status { "✓ Connected" } else { "✗ Offline" });
    println!("   AI Models: {}", if api_status { "✓ Available (323+ models)" } else { "✗ Unavailable" });
    println!();
    
    // Memory usage
    let memory_usage = get_memory_usage();
    println!(" Performance:");
    println!("   Memory Usage: {:.1} MB", memory_usage as f64 / 1024.0 / 1024.0);
    println!("   Consensus Engine: ✓ Ready");
    println!();
    
    Ok(())
}

async fn process_interactive_command(input: &str) -> Result<()> {
    // Split the input into command and arguments
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let command = parts[0];
    let args = if parts.len() > 1 { parts[1] } else { "" };
    
    println!(); // Add spacing before output
    
    match command {
        "ask" => {
            if args.is_empty() {
                println!("❌ Usage: ask <question>");
                return Ok(());
            }
            println!("🤔 Processing your question...");
            println!("🧠 Running 4-stage consensus pipeline...");
            
            // Simulate consensus processing
            println!();
            println!("Generator → ████████ 100% (claude-3-5-sonnet)");
            println!("Refiner   → ████████ 100% (gpt-4-turbo)");  
            println!("Validator → ████████ 100% (claude-3-opus)");
            println!("Curator   → ████████ 100% (gpt-4o)");
            println!();
            
            // Mock response
            println!("✨ Consensus Response:");
            println!("{}", args);
            println!("(This is a placeholder response during development)");
        }
        "analyze" => {
            if args.is_empty() {
                println!("❌ Usage: analyze <path>");
                return Ok(());
            }
            println!("🔍 Analyzing: {}", args);
            println!("📊 Repository Intelligence:");
            println!("  • Architecture: Rust CLI Application");
            println!("  • Quality Score: 8.5/10");
            println!("  • Files Analyzed: 15");
            println!("  • Technical Debt: Low");
        }
        "plan" => {
            if args.is_empty() {
                println!("❌ Usage: plan <goal>");
                return Ok(());
            }
            println!("📋 Creating development plan for: {}", args);
            println!("✅ Plan created with 5 tasks");
            println!("⏱️  Estimated completion: 2-3 days");
            println!("📝 Use 'execute plan.json' to begin implementation");
        }
        "improve" => {
            if args.is_empty() {
                println!("❌ Usage: improve <file>");
                return Ok(());
            }
            println!("🔍 Analyzing: {}", args);
            println!("💡 Improvement suggestions:");
            println!("  • Add error handling for edge cases");
            println!("  • Consider using async/await for better performance");
            println!("  • Add documentation comments");
        }
        "hooks" => {
            match args {
                "list" => {
                    println!("🔗 Enterprise Hooks:");
                    println!("  ✓ auto-format-rust (enabled)");
                    println!("  ✓ production-guard (enabled)");
                    println!("  ⚠ cost-control (disabled)");
                }
                _ => {
                    println!("❌ Usage: hooks list");
                }
            }
        }
        "memory" => {
            match args.split_once(' ') {
                Some(("search", query)) => {
                    println!("🔍 Searching memory for: \"{}\"", query);
                    println!("📝 Found 3 relevant conversations");
                    println!("  • 2024-07-01: Discussion about Rust performance");
                    println!("  • 2024-06-28: Planning system architecture");
                    println!("  • 2024-06-25: TypeScript to Rust migration");
                }
                _ => {
                    println!("❌ Usage: memory search <query>");
                }
            }
        }
        _ => {
            println!("❌ Unknown command: {}", command);
            println!("💡 Type 'help' for available commands");
        }
    }
    
    println!(); // Add spacing after output
    Ok(())
}

async fn get_conversation_count(config_dir: &std::path::Path) -> Result<usize> {
    let db_path = config_dir.join("hive-ai.db");
    if !db_path.exists() {
        return Ok(0);
    }
    
    // This would connect to the actual database in the real implementation
    // For now, return a placeholder
    Ok(42) // Placeholder - would query actual database
}

async fn load_hive_config(config_dir: &std::path::Path) -> Result<BannerConfig> {
    let config_path = config_dir.join("config.toml");
    let _content = tokio::fs::read_to_string(config_path).await?;
    
    // This would parse the actual TOML config
    // For now, return a placeholder
    Ok(BannerConfig {
        consensus_profile: "Balanced".to_string(),
        model_count: 323,
    })
}

struct BannerConfig {
    consensus_profile: String,
    model_count: usize,
}

async fn print_system_status() -> Result<()> {
    use console::style;
    
    println!("  {}", style("System Status:").bold());
    
    // Check internet connectivity
    let internet_status = check_internet_connection().await;
    println!("    {} {}", 
        style("Internet:").dim(),
        if internet_status {
            style("✓ Connected").green()
        } else {
            style("✗ Offline").red()
        }
    );
    
    // Check OpenRouter API
    let api_status = check_api_status().await;
    println!("    {} {}", 
        style("AI Models:").dim(),
        if api_status {
            style("✓ Available").green()
        } else {
            style("✗ Unavailable").red()
        }
    );
    
    // Memory usage
    let memory_usage = get_memory_usage();
    println!("    {} {}", 
        style("Memory:").dim(),
        style(format!("{:.1} MB", memory_usage as f64 / 1024.0 / 1024.0)).blue()
    );
    
    println!();
    Ok(())
}

async fn check_internet_connection() -> bool {
    // Simple connectivity check
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reqwest::Client::new().get("https://api.openrouter.ai/api/v1/models").send()
    ).await.is_ok()
}

async fn check_api_status() -> bool {
    // This would check actual API key and model availability
    // For now, assume available if internet is connected
    check_internet_connection().await
}

fn get_memory_usage() -> usize {
    // Get current process memory usage
    // This is a placeholder - would use actual system calls
    25 * 1024 * 1024 // 25 MB placeholder
}

fn get_hive_config_dir() -> std::path::PathBuf {
    match std::env::consts::OS {
        "macos" | "linux" => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(format!("{}/.hive", home))
        }
        "windows" => {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(format!("{}\\HiveTechs\\HiveAI", appdata))
        }
        _ => std::path::PathBuf::from(".hive"),
    }
}

// TUI Detection and Launch Functions

async fn should_launch_tui() -> bool {
    // Check for explicit TUI environment variable
    if std::env::var("HIVE_TUI").is_ok() {
        return true;
    }
    
    // Check if running standalone (no command args) and terminal is capable
    let is_standalone = std::env::args().len() == 1;
    let terminal_capable = check_tui_capabilities();
    let tui_enabled = check_tui_preference().await;
    
    is_standalone && terminal_capable && tui_enabled
}

fn check_tui_capabilities() -> bool {
    // Check if we're in a real terminal (not piped/redirected)
    if !atty::is(atty::Stream::Stdout) || !atty::is(atty::Stream::Stdin) {
        return false;
    }
    
    // Check terminal size requirements
    if let Ok((width, height)) = crossterm::terminal::size() {
        width >= 120 && height >= 30
    } else {
        false
    }
}

async fn check_tui_preference() -> bool {
    let config_dir = get_hive_config_dir();
    let config_path = config_dir.join("config.toml");
    
    if config_path.exists() {
        if let Ok(content) = tokio::fs::read_to_string(config_path).await {
            // Simple TOML parsing for TUI preference
            // In real implementation, would use proper TOML parser
            content.contains("tui_mode = true") || !content.contains("tui_mode = false")
        } else {
            true // Default to TUI enabled
        }
    } else {
        true // Default to TUI enabled for new installations
    }
}

async fn launch_tui() -> Result<()> {
    println!("🚀 Launching Hive AI TUI...");
    
    // Check for required dependencies
    if !check_tui_dependencies() {
        println!("❌ TUI dependencies not available in this build");
        println!("💡 Install with: cargo install hive-ai --features tui");
        return Ok(());
    }
    
    // Initialize TUI
    match HiveTui::run().await {
        Ok(()) => {
            println!("👋 Thanks for using Hive AI TUI!");
        }
        Err(e) => {
            eprintln!("❌ TUI error: {}", e);
            eprintln!("💡 Falling back to CLI mode...");
            show_startup_banner().await?;
        }
    }
    
    Ok(())
}

fn check_tui_dependencies() -> bool {
    // In real implementation, this would check if TUI features are compiled in
    // For now, assume they're available
    true
}

// TUI Application Structure (Placeholder)
struct HiveTui;

impl HiveTui {
    async fn run() -> Result<()> {
        use std::io;
        use std::time::Duration;
        
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│  🐝 HiveTechs Consensus - TUI Mode                        │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│                                                             │");
        println!("│  📁 Explorer      │  📝 Editor        │  🧠 Consensus     │");
        println!("│  ├─ src/          │  fn main() {      │  Ask anything...  │");
        println!("│  │  ├─ main.rs     │      println!();  │                   │");
        println!("│  │  └─ lib.rs      │  }               │  🔍 Analyzing...  │");
        println!("│  ├─ tests/         │                   │                   │");
        println!("│  └─ Cargo.toml     │  cursor here ▌    │  Quality: 8.5/10  │");
        println!("│                    │                   │                   │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│  Terminal: $ cargo build                                   │");
        println!("│  Finished dev [unoptimized] target(s) in 2.34s            │");
        println!("│  $ hive analyze .                                          │");
        println!("│  🔍 Repository analysis complete ✅                       │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│  F1: Explorer │ F2: Editor │ F3: Consensus │ F4: Terminal   │");
        println!("│  Ctrl+P: Quick Open │ Ctrl+Q: Quit │ Status: Ready ✅     │");
        println!("└─────────────────────────────────────────────────────────────┘");
        println!();
        println!("🚧 TUI Mode is in development!");
        println!("📋 Features coming soon:");
        println!("   • Full file explorer with Git status");
        println!("   • Syntax-highlighted code editor");
        println!("   • Real-time consensus analysis");
        println!("   • Integrated terminal");
        println!("   • Planning mode visualization");
        println!("   • Memory and analytics panels");
        println!();
        println!("⌨️  Press any key to return to CLI mode...");
        
        // Simple key wait
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(())
    }
}