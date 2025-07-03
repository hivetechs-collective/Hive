//! Command-line argument parsing for Hive AI
//!
//! This module defines the CLI structure using clap with comprehensive
//! command support and Claude Code-style argument patterns.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::io::IsTerminal;

/// Hive AI - AI-powered codebase intelligence platform
#[derive(Parser)]
#[command(name = "hive")]
#[command(about = "AI-powered codebase intelligence platform with multi-model consensus")]
#[command(long_about = "
HiveTechs Consensus provides AI-powered code understanding, analysis, and transformation
using a revolutionary 4-stage consensus engine with 323+ AI models.

Features:
  • Multi-model consensus for reliable AI responses
  • Repository intelligence and semantic understanding  
  • Planning mode for complex development workflows
  • TUI interface with VS Code-like experience
  • Enterprise hooks and automation
  • Real-time collaboration and memory

Visit https://docs.hivetechs.com for documentation and examples.
")]
#[command(version)]
pub struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Configuration file path
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,
    
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
    
    /// Suppress output (overrides verbose)
    #[arg(short, long, global = true)]
    pub quiet: bool,
    
    /// Output format (text, json, yaml)
    #[arg(long, global = true, default_value = "text")]
    pub format: String,
    
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Hive in the current project
    #[command(alias = "i")]
    Initialize {
        /// Project path (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Force initialization even if already initialized
        #[arg(short, long)]
        force: bool,
        
        /// Skip interactive setup
        #[arg(long)]
        non_interactive: bool,
    },
    
    /// Ask the AI consensus a question
    Ask {
        /// The question to ask
        #[arg(value_name = "QUESTION")]
        question: String,
        
        /// Consensus profile (speed, balanced, cost, elite)
        #[arg(short, long, default_value = "balanced")]
        profile: String,
        
        /// Enable planning mode for complex queries
        #[arg(long)]
        plan: bool,
        
        /// Include current file context
        #[arg(short, long)]
        context: Option<PathBuf>,
        
        /// Maximum tokens in response
        #[arg(long)]
        max_tokens: Option<u32>,
        
        /// Stream response in real-time
        #[arg(long, default_value = "true")]
        stream: bool,
    },
    
    /// Run 4-stage consensus analysis
    Consensus {
        /// Query for consensus analysis
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Consensus profile to use
        #[arg(short, long, default_value = "balanced")]
        profile: String,
        
        /// Show detailed stage breakdown
        #[arg(long)]
        detailed: bool,
        
        /// Save consensus result to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Analyze and understand any repository
    #[command(alias = "a")]
    Analyze {
        /// Repository path or URL
        #[arg(value_name = "TARGET")]
        target: Option<String>,
        
        /// Analysis depth (quick, standard, comprehensive)
        #[arg(short, long, default_value = "standard")]
        depth: String,
        
        /// Focus areas (architecture, quality, security, performance)
        #[arg(long, value_delimiter = ',')]
        focus: Vec<String>,
        
        /// Save analysis to file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Include dependency analysis
        #[arg(long)]
        dependencies: bool,
        
        /// Generate recommendations
        #[arg(long)]
        recommendations: bool,
    },
    
    /// Search for symbols in the codebase with sub-millisecond performance
    #[command(alias = "s")]
    Search {
        /// Search query (supports FTS5 syntax)
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Filter by symbol kind (function, class, struct, etc.)
        #[arg(short, long)]
        kind: Option<String>,
        
        /// Search in specific path
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Maximum results to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
        
        /// Enable fuzzy matching
        #[arg(short, long)]
        fuzzy: bool,
    },
    
    /// Find all references to a symbol
    #[command(alias = "refs")]
    References {
        /// Symbol name to find references for
        #[arg(value_name = "SYMBOL")]
        symbol: String,
        
        /// File containing the symbol
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Line number of the symbol
        #[arg(short, long)]
        line: Option<usize>,
        
        /// Include declaration as well as references
        #[arg(long)]
        include_declaration: bool,
        
        /// Group results by file
        #[arg(long)]
        group_by_file: bool,
    },
    
    /// Show call graph for a function
    #[command(alias = "calls")]
    CallGraph {
        /// Function name to analyze
        #[arg(value_name = "FUNCTION")]
        function: String,
        
        /// Maximum depth to traverse
        #[arg(short, long, default_value = "3")]
        depth: usize,
        
        /// Output format (text, dot, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Show incoming calls (callers)
        #[arg(long)]
        incoming: bool,
        
        /// Show outgoing calls (callees)
        #[arg(long)]
        outgoing: bool,
    },
    
    /// Find circular dependencies in the codebase
    #[command(alias = "circular")]
    FindCircularDeps {
        /// Root path to analyze
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Output format (text, dot, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Show only severe circular dependencies
        #[arg(long)]
        severe_only: bool,
        
        /// Include breaking point suggestions
        #[arg(long)]
        suggest_fixes: bool,
    },
    
    /// Analyze dependency layers and architecture
    #[command(alias = "layers")]
    DependencyLayers {
        /// Root path to analyze
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Visualization format (text, dot, mermaid)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Show layer violations
        #[arg(long)]
        show_violations: bool,
        
        /// Maximum layers to show
        #[arg(long, default_value = "10")]
        max_layers: usize,
    },
    
    /// Enter planning mode for complex tasks
    #[command(alias = "p")]
    Plan {
        /// Description of what you want to accomplish
        #[arg(value_name = "GOAL")]
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
        
        /// Include risk assessment
        #[arg(long)]
        risks: bool,
        
        /// Generate timeline estimates
        #[arg(long)]
        timeline: bool,
    },
    
    /// Execute a previously created plan
    Execute {
        /// Plan file or plan ID
        #[arg(value_name = "PLAN")]
        plan: String,
        
        /// Auto-execute without confirmation
        #[arg(short, long)]
        auto: bool,
        
        /// Validation level (basic, standard, strict)
        #[arg(short, long, default_value = "standard")]
        validation: String,
        
        /// Dry run (show what would be executed)
        #[arg(long)]
        dry_run: bool,
        
        /// Continue on errors
        #[arg(long)]
        continue_on_error: bool,
    },
    
    /// Decompose a task into subtasks
    #[command(alias = "d")]
    Decompose {
        /// Task description to decompose
        #[arg(value_name = "TASK")]
        task: String,
        
        /// Maximum decomposition depth
        #[arg(short, long)]
        depth: Option<usize>,
        
        /// Include time estimates
        #[arg(short, long)]
        estimate: bool,
    },
    
    /// Analyze risks for a project or plan
    #[command(alias = "ar", name = "analyze-risks")]
    AnalyzeRisks {
        /// Project or plan file to analyze
        #[arg(value_name = "PROJECT")]
        project: Option<String>,
        
        /// Include mitigation strategies
        #[arg(short, long)]
        mitigation: bool,
    },
    
    /// Generate timeline estimates
    #[command(alias = "t")]
    Timeline {
        /// Project or plan to estimate
        #[arg(value_name = "PROJECT")]
        project: Option<String>,
        
        /// Show dependency graph
        #[arg(short, long)]
        dependencies: bool,
    },
    
    /// Enable collaborative planning
    #[command(alias = "collab")]
    Collaborate {
        /// Plan to collaborate on
        #[arg(value_name = "PLAN")]
        plan: String,
        
        /// Team members to invite
        #[arg(short, long)]
        team: Option<Vec<String>>,
        
        /// Share plan via secure link
        #[arg(short, long)]
        share: bool,
    },
    
    // /// Manage intelligent mode switching (planning/execution/hybrid)
    // #[command(alias = "m")]
    // Mode {
    //     #[command(subcommand)]
    //     command: crate::commands::mode::ModeCommands,
    // },
    
    /// Apply AI-suggested improvements to files
    Improve {
        /// File to improve
        #[arg(value_name = "FILE")]
        file: PathBuf,
        
        /// Aspect to improve (e.g., error-handling, performance, readability)
        #[arg(short, long, value_name = "ASPECT")]
        aspect: String,
        
        /// Preview changes without applying
        #[arg(long)]
        preview: bool,
        
        /// Apply changes (required if --preview not specified)
        #[arg(long)]
        apply: bool,
        
        /// Include related files in the transformation
        #[arg(long)]
        multi_file: bool,
        
        /// Additional context for the improvement
        #[arg(short, long)]
        context: Option<String>,
        
        /// List available improvement aspects
        #[arg(long)]
        list_aspects: bool,
    },
    
    /// Apply AI-suggested changes to files
    Apply {
        /// Changes to apply (file or JSON)
        #[arg(value_name = "CHANGES")]
        changes: String,
        
        /// Preview changes before applying
        #[arg(long)]
        preview: bool,
        
        /// Approve all changes without prompting
        #[arg(long)]
        approve: bool,
    },
    
    /// Preview code transformations without applying
    Preview {
        /// File to preview transformations for
        #[arg(value_name = "FILE")]
        file: PathBuf,
        
        /// Changes to preview (optional)
        #[arg(long)]
        changes: Option<String>,
    },
    
    /// AI-guided code transformations
    Transform {
        /// Query describing the transformation
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Target files or directory
        #[arg(long)]
        target: Option<PathBuf>,
        
        /// Safe mode - extra validation
        #[arg(long)]
        safe: bool,
    },
    
    /// Undo the last applied transformation
    Undo {
        /// Specific transaction ID to undo
        #[arg(value_name = "TRANSACTION_ID")]
        transaction: Option<String>,
    },
    
    /// Redo the last undone transformation
    Redo {
        /// Specific transaction ID to redo
        #[arg(value_name = "TRANSACTION_ID")]
        transaction: Option<String>,
    },
    
    /// View transformation history
    TransformHistory {
        /// Number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Generate comprehensive analytics reports
    Analytics {
        #[command(subcommand)]
        command: AnalyticsCommands,
    },
    
    /// Manage long-term memory and conversations
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    
    /// Execute tools and tool chains
    Tool {
        /// Tool name
        #[arg(value_name = "TOOL")]
        name: String,
        
        /// Tool parameters (JSON format)
        #[arg(short, long)]
        params: Option<String>,
        
        /// Execute as part of a tool chain
        #[arg(short, long)]
        chain: Option<String>,
        
        /// List available tools
        #[arg(long)]
        list: bool,
    },
    
    /// Start IDE integration servers
    Serve {
        /// Server mode (mcp, lsp, both)
        #[arg(short, long, default_value = "mcp")]
        mode: String,
        
        /// Port to listen on
        #[arg(short, long, default_value = "7777")]
        port: u16,
        
        /// Bind address
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        
        /// Enable CORS for web clients
        #[arg(long)]
        cors: bool,
    },
    
    /// Manage semantic indices for fast search
    Index {
        #[command(subcommand)]
        command: IndexCommands,
    },
    
    /// Detect language of a file or from stdin
    #[command(alias = "detect", visible_alias = "lang")]
    DetectLanguage {
        /// File path to analyze (reads from stdin if not provided)
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        
        /// Show confidence score
        #[arg(short, long)]
        confidence: bool,
        
        /// Show detailed analysis
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Test incremental parsing performance
    #[command(alias = "edit-perf")]
    EditPerformanceTest {
        /// Number of test iterations
        #[arg(short, long, default_value = "100")]
        iterations: u32,
        
        /// File to use for testing (default: generates test content)
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Language to test
        #[arg(short, long, default_value = "rust")]
        language: String,
        
        /// Show detailed timing breakdown
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    
    /// Manage directory trust and security settings
    Trust {
        #[command(subcommand)]
        command: TrustCommands,
    },
    
    /// Manage enterprise hooks and automation
    Hooks {
        #[command(subcommand)]
        command: HookCommands,
    },
    
    /// Start interactive mode
    Interactive {
        /// Starting mode (planning, execution, hybrid)
        #[arg(short, long, default_value = "hybrid")]
        mode: String,
        
        /// Disable TUI even if available
        #[arg(long)]
        no_tui: bool,
    },
    
    /// Launch full TUI interface (VS Code-like)
    Tui {
        /// Force TUI even if not detected as capable
        #[arg(long)]
        force: bool,
        
        /// TUI layout (default, minimal, custom)
        #[arg(long, default_value = "default")]
        layout: String,
    },
    
    /// Show system status and health
    Status {
        /// Show detailed system information
        #[arg(short, long)]
        detailed: bool,
        
        /// Check connectivity and APIs
        #[arg(long)]
        check_apis: bool,
        
        /// Show performance metrics
        #[arg(long)]
        performance: bool,
    },
    
    /// Generate shell completions
    Completion {
        /// Shell to generate completions for (bash, zsh, fish, powershell)
        #[arg(value_name = "SHELL")]
        shell: String,
        
        /// Output file (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Manage shell integration and environment setup
    Shell {
        #[command(subcommand)]
        command: crate::commands::ShellCommands,
    },
    
    /// Self-update Hive AI binary
    SelfUpdate {
        /// Check for updates only (don't install)
        #[arg(long)]
        check_only: bool,
        
        /// Force update even if current version is newer
        #[arg(long)]
        force: bool,
        
        /// Update to specific version
        #[arg(long)]
        version: Option<String>,
        
        /// Rollback to previous version
        #[arg(long)]
        rollback: bool,
        
        /// Show available versions
        #[arg(long)]
        list_versions: bool,
    },
    
    /// Uninstall Hive AI completely
    Uninstall {
        /// Show what would be removed without actually removing
        #[arg(long)]
        dry_run: bool,
        
        /// Preserve configuration files
        #[arg(long)]
        preserve_config: bool,
        
        /// Preserve conversation data and history
        #[arg(long)]
        preserve_data: bool,
        
        /// Force removal without confirmation prompts
        #[arg(long)]
        force: bool,
        
        /// Create backup before uninstalling
        #[arg(long)]
        backup: bool,
    },
    
    /// Migrate from TypeScript Hive AI installation
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
    
    /// Language Server Protocol (LSP) server for IDE integration
    Lsp {
        #[command(subcommand)]
        command: crate::commands::LspCommands,
    },
    
    /// Enterprise security and compliance management
    Security {
        #[command(subcommand)]
        command: crate::commands::SecurityCommands,
    },
}

/// Index management subcommands
#[derive(Subcommand)]
pub enum IndexCommands {
    /// Build semantic index for a project
    Build {
        /// Path to index (defaults to current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Force rebuild existing indices
        #[arg(short, long)]
        force: bool,
        
        /// Include test files
        #[arg(long)]
        include_tests: bool,
        
        /// Exclude patterns (glob syntax)
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,
        
        /// Show indexing progress
        #[arg(long)]
        progress: bool,
    },
    
    /// Show index statistics and health
    Stats {
        /// Show detailed statistics
        #[arg(short, long)]
        detailed: bool,
        
        /// Check index health
        #[arg(long)]
        health: bool,
    },
    
    /// Rebuild index for specific files
    Rebuild {
        /// Files to rebuild (defaults to all)
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,
        
        /// Force rebuild even if files haven't changed
        #[arg(short, long)]
        force: bool,
    },
    
    /// Clear all indices
    Clear {
        /// Confirm deletion without prompt
        #[arg(long)]
        confirm: bool,
    },
}

/// Migration subcommands
#[derive(Subcommand)]
pub enum MigrateCommands {
    /// Run interactive migration wizard
    Wizard {
        /// TypeScript installation path (auto-detect if not specified)
        #[arg(long)]
        from: Option<PathBuf>,
        
        /// Enable professional UI theme
        #[arg(long)]
        professional: bool,
        
        /// Skip pre-migration checks
        #[arg(long)]
        skip_checks: bool,
    },
    
    /// Quick migration with minimal UI
    Quick {
        /// TypeScript installation directory
        #[arg(long)]
        from: PathBuf,
        
        /// Migration type (upgrade, parallel, fresh, staged)
        #[arg(long, default_value = "upgrade")]
        migration_type: String,
        
        /// Validation level (basic, standard, strict, paranoid)
        #[arg(long, default_value = "standard")]
        validation: String,
        
        /// Create backup before migration
        #[arg(long)]
        backup: bool,
    },
    
    /// Test migration with live TypeScript installation
    Test {
        /// TypeScript installation path
        #[arg(long)]
        from: Option<PathBuf>,
        
        /// Test database size (small, medium, large)
        #[arg(long, default_value = "small")]
        size: String,
        
        /// Test scenarios to run
        #[arg(long, value_delimiter = ',')]
        scenarios: Vec<String>,
        
        /// Timeout in minutes
        #[arg(long, default_value = "10")]
        timeout: u32,
        
        /// Enable performance profiling
        #[arg(long)]
        profile: bool,
    },
    
    /// Analyze TypeScript installation for migration readiness
    Analyze {
        /// TypeScript installation path
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        
        /// Show detailed analysis
        #[arg(short, long)]
        detailed: bool,
        
        /// Export analysis report
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Check compatibility issues
        #[arg(long)]
        compatibility: bool,
    },
    
    /// Benchmark migration performance
    Benchmark {
        /// TypeScript installation path
        #[arg(long)]
        from: Option<PathBuf>,
        
        /// Number of conversations to test with
        #[arg(long, default_value = "1000")]
        conversations: u32,
        
        /// Test different batch sizes
        #[arg(long)]
        batch_sizes: bool,
        
        /// Test parallel processing effectiveness
        #[arg(long)]
        parallel: bool,
        
        /// Export benchmark results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Validate completed migration
    Validate {
        /// Source database path
        #[arg(long)]
        source: PathBuf,
        
        /// Target database path  
        #[arg(long)]
        target: PathBuf,
        
        /// Validation level (basic, standard, strict, paranoid)
        #[arg(long, default_value = "standard")]
        level: String,
        
        /// Sample percentage for validation
        #[arg(long, default_value = "10.0")]
        sample: f64,
        
        /// Generate validation report
        #[arg(short, long)]
        report: Option<PathBuf>,
    },
    
    /// Preview migration changes without applying
    Preview {
        /// TypeScript installation path
        #[arg(value_name = "PATH")]
        path: PathBuf,
        
        /// Show database changes
        #[arg(long)]
        database: bool,
        
        /// Show configuration changes
        #[arg(long)]
        config: bool,
        
        /// Estimate migration time
        #[arg(long)]
        timing: bool,
        
        /// Export preview to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Optimize migration performance
    Optimize {
        /// TypeScript installation path for testing
        #[arg(long)]
        from: Option<PathBuf>,
        
        /// Target performance factor (e.g., 25 for 25x improvement)
        #[arg(long, default_value = "25")]
        target: f64,
        
        /// Memory limit in MB
        #[arg(long, default_value = "512")]
        memory: u32,
        
        /// CPU cores to use
        #[arg(long)]
        cores: Option<u32>,
        
        /// Enable all optimizations
        #[arg(long)]
        max_performance: bool,
    },
    
    /// Rollback migration to TypeScript version
    Rollback {
        /// Backup path to restore from
        #[arg(long)]
        backup: PathBuf,
        
        /// Confirm rollback without prompt
        #[arg(long)]
        confirm: bool,
        
        /// Preserve Rust-specific data
        #[arg(long)]
        preserve_rust_data: bool,
    },
    
    /// Show migration status and statistics
    Status {
        /// Show detailed migration history
        #[arg(short, long)]
        detailed: bool,
        
        /// Check system health post-migration
        #[arg(long)]
        health: bool,
        
        /// Show performance metrics
        #[arg(long)]
        metrics: bool,
    },
}

/// Analytics subcommands
#[derive(Subcommand)]
pub enum AnalyticsCommands {
    /// Generate usage analytics
    Usage {
        /// Time period (day, week, month, quarter)
        #[arg(short, long, default_value = "week")]
        period: String,
        
        /// Include detailed breakdowns
        #[arg(long)]
        detailed: bool,
    },
    
    /// Performance analysis
    Performance {
        /// Time period
        #[arg(short, long, default_value = "week")]
        period: String,
        
        /// Include system metrics
        #[arg(long)]
        system: bool,
    },
    
    /// Cost analysis and budgeting
    Cost {
        /// Time period
        #[arg(short, long, default_value = "month")]
        period: String,
        
        /// Show cost breakdown by model
        #[arg(long)]
        by_model: bool,
        
        /// Budget alerts
        #[arg(long)]
        alerts: bool,
    },
    
    /// Quality metrics and trends
    Quality {
        /// Time period
        #[arg(short, long, default_value = "week")]
        period: String,
        
        /// Include code quality metrics
        #[arg(long)]
        code_quality: bool,
    },
    
    /// Comprehensive analytics report
    Report {
        /// Report type (executive, operational, performance, cost)
        #[arg(short, long, default_value = "executive")]
        report_type: String,
        
        /// Time period
        #[arg(short, long, default_value = "month")]
        period: String,
        
        /// Include charts and visualizations
        #[arg(long)]
        charts: bool,
        
        /// Save report to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Trend analysis
    Trends {
        /// Metric to analyze
        #[arg(value_name = "METRIC")]
        metric: String,
        
        /// Time period
        #[arg(short, long, default_value = "quarter")]
        period: String,
        
        /// Prediction horizon (days)
        #[arg(long)]
        predict: Option<u32>,
    },
}

/// Memory management subcommands
#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Search conversation history
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Search in specific timeframe
        #[arg(long)]
        since: Option<String>,
        
        /// Include context around matches
        #[arg(long)]
        context: bool,
    },
    
    /// Show memory statistics
    Stats {
        /// Show detailed statistics
        #[arg(long)]
        detailed: bool,
        
        /// Show memory health
        #[arg(long)]
        health: bool,
    },
    
    /// Export conversation history
    Export {
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Export format (json, csv, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Date range for export
        #[arg(long)]
        since: Option<String>,
        
        /// Include private data
        #[arg(long)]
        include_private: bool,
    },
    
    /// Import conversation history
    Import {
        /// Input file
        #[arg(value_name = "FILE")]
        file: PathBuf,
        
        /// Import format (auto-detect if not specified)
        #[arg(short, long)]
        format: Option<String>,
        
        /// Merge with existing data
        #[arg(long)]
        merge: bool,
    },
    
    /// Clear memory (with confirmation)
    Clear {
        /// Clear all data
        #[arg(long)]
        all: bool,
        
        /// Clear conversations older than days
        #[arg(long)]
        older_than: Option<u32>,
        
        /// Skip confirmation prompts
        #[arg(long)]
        force: bool,
    },
    
    /// Manage knowledge graph
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommands,
    },
}

/// Knowledge graph subcommands
#[derive(Subcommand)]
pub enum KnowledgeCommands {
    /// View knowledge graph statistics
    Stats {
        /// Show detailed node/edge breakdown
        #[arg(long)]
        detailed: bool,
    },
    
    /// Query the knowledge graph
    Query {
        /// Query string
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Maximum results
        #[arg(short, long, default_value = "20")]
        limit: usize,
        
        /// Include relationship paths
        #[arg(long)]
        paths: bool,
    },
    
    /// Export knowledge graph
    Export {
        /// Output file
        #[arg(value_name = "FILE")]
        output: PathBuf,
        
        /// Format (graphml, json, dot, cypher)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Include node attributes
        #[arg(long)]
        attributes: bool,
    },
    
    /// Visualize knowledge graph
    Visualize {
        /// Output image file
        #[arg(value_name = "FILE")]
        output: PathBuf,
        
        /// Layout algorithm (force, circular, hierarchical)
        #[arg(short, long, default_value = "force")]
        layout: String,
        
        /// Focus on specific nodes
        #[arg(long, value_delimiter = ',')]
        focus: Vec<String>,
    },
}

/// Configuration subcommands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Show specific section only
        #[arg(value_name = "SECTION")]
        section: Option<String>,
        
        /// Show sensitive values (API keys, etc.)
        #[arg(long)]
        show_sensitive: bool,
    },
    
    /// Set a configuration value
    Set {
        /// Configuration key (dot notation)
        #[arg(value_name = "KEY")]
        key: String,
        
        /// Configuration value
        #[arg(value_name = "VALUE")]
        value: String,
        
        /// Set globally (not per-project)
        #[arg(short, long)]
        global: bool,
    },
    
    /// Get a configuration value
    Get {
        /// Configuration key (dot notation)
        #[arg(value_name = "KEY")]
        key: String,
        
        /// Check global config only
        #[arg(short, long)]
        global: bool,
    },
    
    /// Validate configuration
    Validate {
        /// Validate specific file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,
        
        /// Fix validation errors automatically
        #[arg(long)]
        fix: bool,
    },
    
    /// Reset configuration to defaults
    Reset {
        /// Reset specific section only
        #[arg(value_name = "SECTION")]
        section: Option<String>,
        
        /// Confirm reset without prompting
        #[arg(long)]
        confirm: bool,
        
        /// Reset global config
        #[arg(short, long)]
        global: bool,
    },
    
    /// Edit configuration in default editor
    Edit {
        /// Edit global config
        #[arg(short, long)]
        global: bool,
    },
}

/// Enterprise hooks subcommands
#[derive(Subcommand)]
pub enum HookCommands {
    /// List all configured hooks
    List {
        /// Filter by event type
        #[arg(short, long)]
        event: Option<String>,
        
        /// Show only enabled hooks
        #[arg(long)]
        enabled_only: bool,
        
        /// Show hook details
        #[arg(long)]
        detailed: bool,
    },
    
    /// Add a new hook
    Add {
        /// Hook configuration file (JSON/YAML)
        #[arg(value_name = "CONFIG")]
        config: PathBuf,
        
        /// Enable hook immediately
        #[arg(long)]
        enable: bool,
    },
    
    /// Remove a hook
    Remove {
        /// Hook ID to remove
        #[arg(value_name = "HOOK_ID")]
        hook_id: String,
        
        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },
    
    /// Enable or disable a hook
    Toggle {
        /// Hook ID to toggle
        #[arg(value_name = "HOOK_ID")]
        hook_id: String,
        
        /// Explicitly enable (vs toggle)
        #[arg(long)]
        enable: bool,
        
        /// Explicitly disable (vs toggle)
        #[arg(long)]
        disable: bool,
    },
    
    /// Test a hook configuration
    Test {
        /// Hook configuration file or hook ID
        #[arg(value_name = "HOOK")]
        hook: String,
        
        /// Mock event to trigger
        #[arg(value_name = "EVENT")]
        event: String,
        
        /// Test data (JSON)
        #[arg(long)]
        data: Option<String>,
    },
    
    /// Validate all hook configurations
    Validate {
        /// Validate specific hook only
        #[arg(value_name = "HOOK_ID")]
        hook_id: Option<String>,
        
        /// Fix validation errors
        #[arg(long)]
        fix: bool,
    },
    
    /// Show hook execution history
    History {
        /// Number of recent executions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Filter by hook ID
        #[arg(long)]
        hook_id: Option<String>,
        
        /// Show only failures
        #[arg(long)]
        failures_only: bool,
    },
}

/// Trust management subcommands
#[derive(Subcommand)]
pub enum TrustCommands {
    /// List all trusted directories
    List {
        /// Show detailed trust information
        #[arg(short, long)]
        detailed: bool,
        
        /// Show trust entries by status
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        
        /// Show expired trust entries
        #[arg(long)]
        expired: bool,
    },
    
    /// Add a directory to trusted paths
    Add {
        /// Directory path to trust
        #[arg(value_name = "PATH")]
        path: PathBuf,
        
        /// Trust level (trusted, untrusted, blocked)
        #[arg(short, long, default_value = "trusted")]
        level: String,
        
        /// Reason for trusting this directory
        #[arg(short, long)]
        reason: Option<String>,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    
    /// Remove a directory from trusted paths
    Remove {
        /// Directory path to remove trust
        #[arg(value_name = "PATH")]
        path: PathBuf,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    
    /// Clear all trusted paths
    Clear {
        /// Confirm clearing all trusted paths
        #[arg(long)]
        confirm: bool,
        
        /// Clear only expired trust entries
        #[arg(long)]
        expired_only: bool,
    },
    
    /// Check trust status of a directory
    Check {
        /// Directory path to check
        #[arg(value_name = "PATH")]
        path: PathBuf,
        
        /// Show detailed security information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Manage security configuration
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },
    
    /// Import trust settings from file
    Import {
        /// JSON file containing trust settings
        #[arg(value_name = "FILE")]
        file: PathBuf,
        
        /// Merge with existing settings
        #[arg(short, long)]
        merge: bool,
        
        /// Skip validation warnings
        #[arg(long)]
        skip_validation: bool,
    },
    
    /// Export trust settings to file
    Export {
        /// Output file for trust settings
        #[arg(value_name = "FILE")]
        file: PathBuf,
        
        /// Export format (json, toml)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// Include sensitive data
        #[arg(long)]
        include_sensitive: bool,
    },
}

/// Security configuration subcommands
#[derive(Subcommand)]
pub enum SecurityCommands {
    /// Show current security configuration
    Show {
        /// Show specific section only
        #[arg(value_name = "SECTION")]
        section: Option<String>,
        
        /// Show sensitive values
        #[arg(long)]
        show_sensitive: bool,
    },
    
    /// Validate security configuration
    Validate {
        /// Fix validation warnings automatically
        #[arg(long)]
        fix: bool,
        
        /// Show detailed validation information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Set security profile (development, production, enterprise)
    SetProfile {
        /// Security profile to use
        #[arg(value_name = "PROFILE")]
        profile: String,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    
    /// Reset security configuration to defaults
    Reset {
        /// Confirm reset operation
        #[arg(long)]
        confirm: bool,
        
        /// Reset to specific profile
        #[arg(long)]
        profile: Option<String>,
    },
    
    /// View security audit logs
    Audit {
        /// Number of recent entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
        
        /// Filter by event type
        #[arg(long)]
        event_type: Option<String>,
        
        /// Show events from specific date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,
        
        /// Follow log in real-time
        #[arg(short, long)]
        follow: bool,
    },
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
    
    /// Get verbosity level
    pub fn verbosity(&self) -> tracing::Level {
        if self.quiet {
            tracing::Level::ERROR
        } else {
            match self.verbose {
                0 => tracing::Level::INFO,
                1 => tracing::Level::DEBUG,
                _ => tracing::Level::TRACE,
            }
        }
    }
    
    /// Check if output should be colored
    pub fn use_color(&self) -> bool {
        !self.no_color && std::io::IsTerminal::is_terminal(&std::io::stdout())
    }
}