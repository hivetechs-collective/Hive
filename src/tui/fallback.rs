//! Simple CLI fallback mode
//!
//! Provides basic functionality for terminals that don't support advanced TUI

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io::{self, Write};
use crate::core::temporal::TemporalContext;
use is_terminal::IsTerminal;

/// Simple CLI interface for fallback mode
pub struct SimpleCli {
    /// Temporal context for time awareness
    temporal: TemporalContext,
    /// Current working directory
    current_dir: std::path::PathBuf,
    /// Command history
    history: Vec<String>,
    /// Exit flag
    should_exit: bool,
}

/// CLI command result
#[derive(Debug)]
pub enum CliResult {
    Continue,
    Exit,
    Error(String),
}

impl SimpleCli {
    /// Create new simple CLI
    pub fn new() -> Result<Self> {
        Ok(Self {
            temporal: TemporalContext::new(),
            current_dir: std::env::current_dir()?,
            history: Vec::new(),
            should_exit: false,
        })
    }

    /// Run the simple CLI interface
    pub async fn run(&mut self) -> Result<()> {
        self.show_welcome();
        
        loop {
            self.show_prompt();
            
            match self.read_input().await? {
                CliResult::Continue => continue,
                CliResult::Exit => break,
                CliResult::Error(err) => {
                    eprintln!("Error: {}", err);
                }
            }
        }
        
        self.show_goodbye();
        Ok(())
    }

    /// Show welcome message
    fn show_welcome(&self) {
        println!("░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░");
        println!("░                                      ░");
        println!("░      🐝 HiveTechs Consensus CLI       ░");
        println!("░                                      ░");
        println!("░  Simple Mode - Basic Functionality   ░");
        println!("░                                      ░");
        println!("░  Time: {}              ░", self.temporal.current_time_formatted());
        println!("░  Dir:  {}                             ░", 
                 self.current_dir.display().to_string().chars().take(28).collect::<String>());
        println!("░                                      ░");
        println!("░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░");
        println!();
        println!("Welcome to HiveTechs Consensus Simple CLI Mode!");
        println!("Your terminal doesn't support advanced TUI features.");
        println!();
        println!("Available commands:");
        println!("  help        - Show this help message");
        println!("  ask [text]  - Ask the consensus engine");
        println!("  status      - Show system status");
        println!("  analyze     - Analyze current directory");
        println!("  history     - Show command history");
        println!("  clear       - Clear screen");
        println!("  cd [dir]    - Change directory");
        println!("  ls          - List directory contents");
        println!("  pwd         - Show current directory");
        println!("  time        - Show current time");
        println!("  exit/quit   - Exit the CLI");
        println!();
        println!("Type 'help' for more information or start with a command.");
        println!();
    }

    /// Show command prompt
    fn show_prompt(&self) {
        let dir_name = self.current_dir
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("/"))
            .to_string_lossy();
        
        print!("🐝 {}> ", dir_name);
        io::stdout().flush().unwrap();
    }

    /// Read user input with fallback to basic stdin
    async fn read_input(&mut self) -> Result<CliResult> {
        let mut input = String::new();
        
        // Try to use crossterm events first
        if Self::can_use_crossterm_events() {
            loop {
                if event::poll(std::time::Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Enter => {
                                    println!(); // New line
                                    break;
                                }
                                KeyCode::Char(c) => {
                                    input.push(c);
                                    print!("{}", c);
                                    io::stdout().flush()?;
                                }
                                KeyCode::Backspace => {
                                    if !input.is_empty() {
                                        input.pop();
                                        print!("\x08 \x08"); // Backspace, space, backspace
                                        io::stdout().flush()?;
                                    }
                                }
                                KeyCode::Esc => {
                                    return Ok(CliResult::Exit);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else {
            // Fall back to basic stdin read
            io::stdin().read_line(&mut input)?;
            input = input.trim().to_string();
        }
        
        self.process_command(input.trim()).await
    }
    
    /// Check if we can use crossterm events
    fn can_use_crossterm_events() -> bool {
        // Try to poll for events to test if it works
        match event::poll(std::time::Duration::from_millis(1)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Process user command
    async fn process_command(&mut self, command: &str) -> Result<CliResult> {
        if command.is_empty() {
            return Ok(CliResult::Continue);
        }
        
        // Add to history
        self.history.push(command.to_string());
        
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(CliResult::Continue);
        }
        
        match parts[0] {
            "help" => {
                self.show_help();
                Ok(CliResult::Continue)
            }
            "ask" => {
                let question = parts[1..].join(" ");
                self.handle_ask_command(&question).await
            }
            "status" => {
                self.show_status();
                Ok(CliResult::Continue)
            }
            "analyze" => {
                self.analyze_directory().await
            }
            "history" => {
                self.show_history();
                Ok(CliResult::Continue)
            }
            "clear" => {
                self.clear_screen();
                Ok(CliResult::Continue)
            }
            "cd" => {
                let target = if parts.len() > 1 { parts[1] } else { "~" };
                self.change_directory(target)
            }
            "ls" => {
                self.list_directory();
                Ok(CliResult::Continue)
            }
            "pwd" => {
                println!("{}", self.current_dir.display());
                Ok(CliResult::Continue)
            }
            "time" => {
                self.show_time();
                Ok(CliResult::Continue)
            }
            "exit" | "quit" => {
                Ok(CliResult::Exit)
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
                println!("Type 'help' for available commands.");
                Ok(CliResult::Continue)
            }
        }
    }

    /// Show help message
    fn show_help(&self) {
        println!();
        println!("🐝 HiveTechs Consensus CLI Help");
        println!("===============================");
        println!();
        println!("Core Commands:");
        println!("  ask [question]  - Ask the AI consensus engine anything");
        println!("  status          - Show system status and health");
        println!("  analyze         - Analyze current directory for insights");
        println!();
        println!("Navigation:");
        println!("  cd [directory]  - Change to directory (cd ~ for home)");
        println!("  ls              - List files and directories");
        println!("  pwd             - Show current working directory");
        println!();
        println!("Utility:");
        println!("  history         - Show command history");
        println!("  clear           - Clear the screen");
        println!("  time            - Show current date and time");
        println!("  help            - Show this help message");
        println!("  exit/quit       - Exit the CLI");
        println!();
        println!("Examples:");
        println!("  ask What is Rust?");
        println!("  analyze");
        println!("  cd src");
        println!("  ls");
        println!();
        println!("Note: This is the simple CLI mode. For full TUI experience,");
        println!("      upgrade your terminal or install a TUI-capable version.");
        println!();
    }

    /// Handle ask command
    async fn handle_ask_command(&self, question: &str) -> Result<CliResult> {
        if question.is_empty() {
            println!("Please provide a question. Example: ask What is Rust?");
            return Ok(CliResult::Continue);
        }
        
        println!();
        println!("🧠 Asking HiveTechs Consensus: {}", question);
        println!();
        
        // Show processing animation
        self.show_processing().await;
        
        // TODO: Integrate with actual consensus engine
        // For now, provide a simulated response
        println!("🐝 Consensus Response:");
        println!();
        self.simulate_consensus_response(question);
        println!();
        
        Ok(CliResult::Continue)
    }

    /// Show system status
    fn show_status(&self) {
        println!();
        println!("🐝 HiveTechs Consensus Status");
        println!("=============================");
        println!();
        println!("System Information:");
        println!("  Mode: Simple CLI (Fallback)");
        println!("  Time: {}", self.temporal.current_time_formatted());
        println!("  Directory: {}", self.current_dir.display());
        println!("  Commands in history: {}", self.history.len());
        println!();
        println!("Core Systems:");
        println!("  ✓ CLI Interface: Active");
        println!("  ✓ Temporal Context: Online");
        println!("  ✓ Command Processing: Ready");
        println!("  ✓ File System Access: Available");
        println!();
        println!("Consensus Engine:");
        println!("  ✓ Generator: Ready");
        println!("  ✓ Refiner: Standby");
        println!("  ✓ Validator: Standby");
        println!("  ✓ Curator: Standby");
        println!();
        println!("Features Available:");
        println!("  ✓ Ask Questions");
        println!("  ✓ Directory Analysis");
        println!("  ✓ File Navigation");
        println!("  ✓ Command History");
        println!();
        println!("Note: Advanced TUI features are not available in this terminal.");
        println!();
    }

    /// Analyze current directory
    async fn analyze_directory(&self) -> Result<CliResult> {
        println!();
        println!("🔍 Analyzing directory: {}", self.current_dir.display());
        println!();
        
        self.show_processing().await;
        
        // Analyze directory contents
        let entries = std::fs::read_dir(&self.current_dir)?;
        let mut files = Vec::new();
        let mut dirs = Vec::new();
        let mut total_size = 0u64;
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(entry.file_name().to_string_lossy().to_string());
                } else {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                    files.push(file_name);
                }
            }
        }
        
        println!("📁 Analysis Results:");
        println!();
        println!("Directory Structure:");
        println!("  Subdirectories: {}", dirs.len());
        println!("  Files: {}", files.len());
        println!("  Total size: {} bytes", total_size);
        println!();
        
        if !dirs.is_empty() {
            println!("Subdirectories:");
            for dir in &dirs {
                println!("  📂 {}", dir);
            }
            println!();
        }
        
        if !files.is_empty() {
            println!("Files (first 10):");
            for (i, file) in files.iter().take(10).enumerate() {
                let icon = self.get_file_icon(file);
                println!("  {} {}", icon, file);
                if i == 9 && files.len() > 10 {
                    println!("  ... and {} more files", files.len() - 10);
                }
            }
            println!();
        }
        
        // Simple project detection
        self.detect_project_type(&files);
        
        Ok(CliResult::Continue)
    }

    /// Detect project type based on files
    fn detect_project_type(&self, files: &[String]) {
        let mut project_types = Vec::new();
        
        if files.iter().any(|f| f == "Cargo.toml") {
            project_types.push("Rust Project");
        }
        if files.iter().any(|f| f == "package.json") {
            project_types.push("Node.js Project");
        }
        if files.iter().any(|f| f == "requirements.txt" || f == "pyproject.toml") {
            project_types.push("Python Project");
        }
        if files.iter().any(|f| f == "Makefile") {
            project_types.push("Make Project");
        }
        if files.iter().any(|f| f == ".git") {
            project_types.push("Git Repository");
        }
        
        if !project_types.is_empty() {
            println!("Detected Project Types:");
            for project_type in project_types {
                println!("  ✓ {}", project_type);
            }
            println!();
        }
    }

    /// Get file icon based on extension
    fn get_file_icon(&self, filename: &str) -> &'static str {
        if let Some(ext) = std::path::Path::new(filename).extension() {
            match ext.to_string_lossy().as_ref() {
                "rs" => "🦀",
                "js" | "ts" => "📜",
                "py" => "🐍",
                "md" => "📝",
                "json" | "toml" | "yaml" | "yml" => "⚙️",
                "png" | "jpg" | "jpeg" | "gif" => "🖼️",
                "txt" => "📄",
                _ => "📄",
            }
        } else {
            "📄"
        }
    }

    /// Show command history
    fn show_history(&self) {
        println!();
        println!("📃 Command History ({} commands)", self.history.len());
        println!("========================");
        println!();
        
        if self.history.is_empty() {
            println!("No commands in history yet.");
        } else {
            for (i, cmd) in self.history.iter().enumerate() {
                println!("  {}: {}", i + 1, cmd);
            }
        }
        println!();
    }

    /// Clear screen
    fn clear_screen(&self) {
        // Clear screen using ANSI escape codes
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    /// Change directory
    fn change_directory(&mut self, target: &str) -> Result<CliResult> {
        let new_dir = if target == "~" {
            dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
        } else {
            std::path::PathBuf::from(target)
        };
        
        match std::env::set_current_dir(&new_dir) {
            Ok(()) => {
                self.current_dir = std::env::current_dir()?;
                println!("Changed to: {}", self.current_dir.display());
            }
            Err(e) => {
                println!("Cannot change directory: {}", e);
            }
        }
        
        Ok(CliResult::Continue)
    }

    /// List directory contents
    fn list_directory(&self) {
        println!();
        println!("📁 Contents of: {}", self.current_dir.display());
        println!();
        
        match std::fs::read_dir(&self.current_dir) {
            Ok(entries) => {
                let mut dirs = Vec::new();
                let mut files = Vec::new();
                
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if entry.path().is_dir() {
                            dirs.push(name);
                        } else {
                            files.push(name);
                        }
                    }
                }
                
                // Sort and display
                dirs.sort();
                files.sort();
                
                if dirs.is_empty() && files.is_empty() {
                    println!("  (empty directory)");
                } else {
                    for dir in dirs {
                        println!("  📂 {}/", dir);
                    }
                    for file in files {
                        let icon = self.get_file_icon(&file);
                        println!("  {} {}", icon, file);
                    }
                }
            }
            Err(e) => {
                println!("Cannot read directory: {}", e);
            }
        }
        println!();
    }

    /// Show current time
    fn show_time(&self) {
        println!();
        println!("🕰️ Current Time: {}", self.temporal.current_time_formatted());
        println!("   Date: {}", self.temporal.current_date_formatted());
        println!();
    }

    /// Show processing animation
    async fn show_processing(&self) {
        let frames = ["|", "/", "-", "\\"];
        print!("Processing ");
        
        for _ in 0..8 {
            for frame in &frames {
                print!("\r{}Processing {}", " ".repeat(20), frame);
                io::stdout().flush().unwrap();
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        
        print!("\rProcessing... Done!\n");
        io::stdout().flush().unwrap();
    }

    /// Simulate consensus response
    fn simulate_consensus_response(&self, question: &str) {
        // This would be replaced with actual consensus engine integration
        let responses = vec![
            "This is a simulated response from the consensus engine.",
            "In the full version, this would provide AI-powered insights.",
            "The question has been processed through our 4-stage pipeline.",
            "For real functionality, please use the full TUI version.",
        ];
        
        println!("Question: {}", question);
        println!();
        
        for response in responses {
            println!("{}", response);
        }
        
        println!();
        println!("Note: This is a simulated response. The full consensus engine");
        println!("      is available in the advanced TUI mode.");
    }

    /// Show goodbye message
    fn show_goodbye(&self) {
        println!();
        println!("░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░");
        println!("░                                      ░");
        println!("░    Thank you for using HiveTechs!     ░");
        println!("░                                      ░");
        println!("░     🐝 Consensus Intelligence       ░");
        println!("░                                      ░");
        println!("░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░");
        println!();
        println!("Session ended at: {}", self.temporal.current_time_formatted());
        println!("Commands executed: {}", self.history.len());
        println!();
        println!("For the full experience, please upgrade to a TUI-capable terminal.");
        println!();
    }
}

/// Run simple CLI fallback mode with terminal error handling
pub async fn run_simple_cli() -> Result<()> {
    // Initialize CLI with error handling
    let mut cli = match SimpleCli::new() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("Failed to initialize simple CLI: {}", e);
            eprintln!("Using basic text-only mode...");
            return run_basic_text_mode().await;
        }
    };
    
    // Run the CLI with error handling
    match cli.run().await {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("CLI error: {}", e);
            eprintln!("Falling back to basic text mode...");
            run_basic_text_mode().await
        }
    }
}

/// Run in basic text mode for environments that don't support any terminal features
async fn run_basic_text_mode() -> Result<()> {
    println!("HiveTechs Consensus - Basic Text Mode");
    println!("=====================================\n");
    
    println!("This is a minimal text-only interface for HiveTechs Consensus.");
    println!("Your terminal doesn't support interactive features.\n");
    
    println!("To use the full interface, please:");
    println!("1. Use a modern terminal (iTerm2, Windows Terminal, etc.)");
    println!("2. Ensure you're running in an interactive shell");
    println!("3. Check that your TERM environment variable is set\n");
    
    println!("Current environment:");
    if let Ok(term) = std::env::var("TERM") {
        println!("  TERM: {}", term);
    } else {
        println!("  TERM: (not set)");
    }
    
    println!("  TTY: {}", if std::io::stdout().is_terminal() { "yes" } else { "no" });
    
    println!("\nFor help, visit: https://docs.hivetechs.com/troubleshooting");
    
    Ok(())
}

impl Default for SimpleCli {
    fn default() -> Self {
        Self::new().unwrap()
    }
}