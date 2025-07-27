//! Terminal Panel Component for Dioxus Desktop
//!
//! VS Code-like integrated terminal experience

use dioxus::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use chrono::Local;

const MAX_OUTPUT_LINES: usize = 1000;
const MAX_HISTORY: usize = 100;

/// Terminal panel component that provides command execution
#[component]
pub fn Terminal() -> Element {
    // Terminal state
    let mut output_lines = use_signal(|| VecDeque::<TerminalLine>::new());
    let mut input_text = use_signal(|| String::new());
    let mut command_history = use_signal(|| VecDeque::<String>::new());
    let mut history_index = use_signal(|| Option::<usize>::None);
    let mut is_running = use_signal(|| false);
    let mut current_directory = use_signal(|| {
        std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    // Auto-scroll to bottom flag
    let mut should_scroll = use_signal(|| true);
    
    // Claude Code installation state
    let mut claude_installed = use_signal(|| false);
    let mut checked_claude = use_signal(|| false);

    // Initialize with prompt and check for Claude Code
    use_effect(move || {
        if output_lines.read().is_empty() {
            // Welcome message
            output_lines.write().push_back(TerminalLine {
                text: "🐝 HiveTechs Terminal - VS Code Style".to_string(),
                line_type: LineType::Success,
                timestamp: Local::now(),
            });
            output_lines.write().push_back(TerminalLine {
                text: "Type 'help' for available commands".to_string(),
                line_type: LineType::Output,
                timestamp: Local::now(),
            });
            output_lines.write().push_back(TerminalLine {
                text: String::new(),
                line_type: LineType::Output,
                timestamp: Local::now(),
            });
            
            // Check for Claude Code installation
            if !*checked_claude.read() {
                checked_claude.set(true);
                let mut output_lines = output_lines.clone();
                let mut claude_installed = claude_installed.clone();
                let mut current_directory = current_directory.clone();
                
                spawn(async move {
                    output_lines.write().push_back(TerminalLine {
                        text: "Checking for Claude Code installation...".to_string(),
                        line_type: LineType::Output,
                        timestamp: Local::now(),
                    });
                    
                    // Check if claude command exists
                    match Command::new("claude")
                        .arg("--version")
                        .output()
                        .await
                    {
                        Ok(output) if output.status.success() => {
                            let version = String::from_utf8_lossy(&output.stdout);
                            output_lines.write().push_back(TerminalLine {
                                text: format!("✅ Claude Code is installed: {}", version.trim()),
                                line_type: LineType::Success,
                                timestamp: Local::now(),
                            });
                            claude_installed.set(true);
                        }
                        _ => {
                            output_lines.write().push_back(TerminalLine {
                                text: "⚠️ Claude Code not found. Would you like to install it?".to_string(),
                                line_type: LineType::Error,
                                timestamp: Local::now(),
                            });
                            output_lines.write().push_back(TerminalLine {
                                text: "Run 'install-claude' to set up Claude Code automatically.".to_string(),
                                line_type: LineType::Output,
                                timestamp: Local::now(),
                            });
                        }
                    }
                    
                    output_lines.write().push_back(TerminalLine {
                        text: String::new(),
                        line_type: LineType::Output,
                        timestamp: Local::now(),
                    });
                    
                    // Add initial prompt
                    output_lines.write().push_back(TerminalLine {
                        text: format!("{}> ", current_directory.read()),
                        line_type: LineType::Prompt,
                        timestamp: Local::now(),
                    });
                });
            }
        }
    });

    // Execute command function
    let execute_command = move |command: String| {
        if command.trim().is_empty() {
            return;
        }

        let mut output_lines = output_lines.clone();
        let mut is_running = is_running.clone();
        let mut current_directory = current_directory.clone();
        let mut command_history = command_history.clone();
        let mut should_scroll = should_scroll.clone();

        // Add command to output
        output_lines.write().push_back(TerminalLine {
            text: format!("{}> {}", current_directory.read(), command),
            line_type: LineType::Command,
            timestamp: Local::now(),
        });

        // Add to history
        command_history.write().push_back(command.clone());
        if command_history.read().len() > MAX_HISTORY {
            command_history.write().pop_front();
        }

        // Parse command
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        // Handle built-in commands
        match parts[0] {
            "help" => {
                output_lines.write().push_back(TerminalLine {
                    text: "Available commands:".to_string(),
                    line_type: LineType::Success,
                    timestamp: Local::now(),
                });
                output_lines.write().push_back(TerminalLine {
                    text: "  help              - Show this help message".to_string(),
                    line_type: LineType::Output,
                    timestamp: Local::now(),
                });
                output_lines.write().push_back(TerminalLine {
                    text: "  clear             - Clear the terminal".to_string(),
                    line_type: LineType::Output,
                    timestamp: Local::now(),
                });
                output_lines.write().push_back(TerminalLine {
                    text: "  cd <path>         - Change directory".to_string(),
                    line_type: LineType::Output,
                    timestamp: Local::now(),
                });
                output_lines.write().push_back(TerminalLine {
                    text: "  install-claude    - Install Claude Code CLI".to_string(),
                    line_type: LineType::Output,
                    timestamp: Local::now(),
                });
                output_lines.write().push_back(TerminalLine {
                    text: "  claude <command>  - Run Claude Code commands".to_string(),
                    line_type: LineType::Output,
                    timestamp: Local::now(),
                });
                output_lines.write().push_back(TerminalLine {
                    text: String::new(),
                    line_type: LineType::Output,
                    timestamp: Local::now(),
                });
                output_lines.write().push_back(TerminalLine {
                    text: format!("{}> ", current_directory.read()),
                    line_type: LineType::Prompt,
                    timestamp: Local::now(),
                });
                should_scroll.set(true);
                return;
            }
            "install-claude" => {
                output_lines.write().push_back(TerminalLine {
                    text: "🚀 Installing Claude Code...".to_string(),
                    line_type: LineType::Success,
                    timestamp: Local::now(),
                });
                
                let mut output_lines_clone = output_lines.clone();
                let mut claude_installed_clone = claude_installed.clone();
                let current_dir = current_directory.read().clone();
                
                is_running.set(true);
                spawn(async move {
                    // Try npm install first
                    output_lines_clone.write().push_back(TerminalLine {
                        text: "Attempting to install via npm...".to_string(),
                        line_type: LineType::Output,
                        timestamp: Local::now(),
                    });
                    
                    match Command::new("npm")
                        .args(&["install", "-g", "@anthropic-ai/claude-code"])
                        .output()
                        .await
                    {
                        Ok(output) if output.status.success() => {
                            output_lines_clone.write().push_back(TerminalLine {
                                text: "✅ Claude Code installed successfully!".to_string(),
                                line_type: LineType::Success,
                                timestamp: Local::now(),
                            });
                            claude_installed_clone.set(true);
                        }
                        _ => {
                            // If npm fails, provide manual instructions
                            output_lines_clone.write().push_back(TerminalLine {
                                text: "⚠️ Automatic installation failed. Please install manually:".to_string(),
                                line_type: LineType::Error,
                                timestamp: Local::now(),
                            });
                            output_lines_clone.write().push_back(TerminalLine {
                                text: "1. Visit https://claude.ai/download".to_string(),
                                line_type: LineType::Output,
                                timestamp: Local::now(),
                            });
                            output_lines_clone.write().push_back(TerminalLine {
                                text: "2. Download and install Claude Code for your platform".to_string(),
                                line_type: LineType::Output,
                                timestamp: Local::now(),
                            });
                            output_lines_clone.write().push_back(TerminalLine {
                                text: "3. Restart this terminal once installed".to_string(),
                                line_type: LineType::Output,
                                timestamp: Local::now(),
                            });
                        }
                    }
                    
                    output_lines_clone.write().push_back(TerminalLine {
                        text: String::new(),
                        line_type: LineType::Output,
                        timestamp: Local::now(),
                    });
                    output_lines_clone.write().push_back(TerminalLine {
                        text: format!("{}> ", current_dir),
                        line_type: LineType::Prompt,
                        timestamp: Local::now(),
                    });
                });
                
                is_running.set(false);
                should_scroll.set(true);
                return;
            }
            "clear" => {
                output_lines.write().clear();
                output_lines.write().push_back(TerminalLine {
                    text: format!("{}> ", current_directory.read()),
                    line_type: LineType::Prompt,
                    timestamp: Local::now(),
                });
                should_scroll.set(true);
                return;
            }
            "cd" => {
                if parts.len() > 1 {
                    let new_path = if parts[1].starts_with('~') {
                        if let Some(home) = dirs::home_dir() {
                            home.join(&parts[1][2..])
                        } else {
                            std::path::PathBuf::from(parts[1])
                        }
                    } else {
                        std::path::PathBuf::from(parts[1])
                    };

                    match std::env::set_current_dir(&new_path) {
                        Ok(_) => {
                            let new_dir = std::env::current_dir()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            current_directory.set(new_dir.clone());
                            output_lines.write().push_back(TerminalLine {
                                text: format!("Changed to: {}", new_dir),
                                line_type: LineType::Success,
                                timestamp: Local::now(),
                            });
                        }
                        Err(e) => {
                            output_lines.write().push_back(TerminalLine {
                                text: format!("cd: {}", e),
                                line_type: LineType::Error,
                                timestamp: Local::now(),
                            });
                        }
                    }
                } else {
                    // Change to home directory
                    if let Some(home) = dirs::home_dir() {
                        std::env::set_current_dir(&home).ok();
                        let new_dir = home.to_string_lossy().to_string();
                        current_directory.set(new_dir.clone());
                        output_lines.write().push_back(TerminalLine {
                            text: format!("Changed to: {}", new_dir),
                            line_type: LineType::Success,
                            timestamp: Local::now(),
                        });
                    }
                }

                // Add new prompt
                output_lines.write().push_back(TerminalLine {
                    text: format!("{}> ", current_directory.read()),
                    line_type: LineType::Prompt,
                    timestamp: Local::now(),
                });
                should_scroll.set(true);
                return;
            }
            _ => {}
        }

        // Execute external command
        is_running.set(true);
        spawn(async move {
            let shell = if cfg!(windows) { "cmd" } else { "sh" };
            let shell_arg = if cfg!(windows) { "/C" } else { "-c" };

            match Command::new(shell)
                .arg(shell_arg)
                .arg(&command)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    // Read stdout
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        let mut lines = reader.lines();

                        while let Ok(Some(line)) = lines.next_line().await {
                            output_lines.write().push_back(TerminalLine {
                                text: line,
                                line_type: LineType::Output,
                                timestamp: Local::now(),
                            });

                            // Limit output size
                            while output_lines.read().len() > MAX_OUTPUT_LINES {
                                output_lines.write().pop_front();
                            }

                            should_scroll.set(true);
                        }
                    }

                    // Read stderr
                    if let Some(stderr) = child.stderr.take() {
                        let reader = BufReader::new(stderr);
                        let mut lines = reader.lines();

                        while let Ok(Some(line)) = lines.next_line().await {
                            output_lines.write().push_back(TerminalLine {
                                text: line,
                                line_type: LineType::Error,
                                timestamp: Local::now(),
                            });

                            // Limit output size
                            while output_lines.read().len() > MAX_OUTPUT_LINES {
                                output_lines.write().pop_front();
                            }

                            should_scroll.set(true);
                        }
                    }

                    // Wait for completion
                    match child.wait().await {
                        Ok(status) => {
                            if !status.success() {
                                output_lines.write().push_back(TerminalLine {
                                    text: format!("Process exited with code: {:?}", status.code()),
                                    line_type: LineType::Error,
                                    timestamp: Local::now(),
                                });
                            }
                        }
                        Err(e) => {
                            output_lines.write().push_back(TerminalLine {
                                text: format!("Error: {}", e),
                                line_type: LineType::Error,
                                timestamp: Local::now(),
                            });
                        }
                    }
                }
                Err(e) => {
                    output_lines.write().push_back(TerminalLine {
                        text: format!("Failed to execute command: {}", e),
                        line_type: LineType::Error,
                        timestamp: Local::now(),
                    });
                }
            }

            // Add new prompt
            output_lines.write().push_back(TerminalLine {
                text: format!("{}> ", current_directory.read()),
                line_type: LineType::Prompt,
                timestamp: Local::now(),
            });

            is_running.set(false);
            should_scroll.set(true);
        });
    };

    // Clone for the submit handler
    let execute_command_for_submit = execute_command.clone();

    // Handle key events for history navigation
    let on_keydown = {
        let mut history_index = history_index.clone();
        let mut input_text = input_text.clone();
        let command_history = command_history.clone();
        move |evt: Event<KeyboardData>| {
            match evt.data().key() {
                Key::ArrowUp => {
                    let history = command_history.read();
                    if !history.is_empty() {
                        let current_index = (*history_index.read()).unwrap_or(history.len());
                        if current_index > 0 {
                            let new_index = current_index - 1;
                            if let Some(cmd) = history.get(new_index) {
                                input_text.set(cmd.clone());
                                history_index.set(Some(new_index));
                            }
                        }
                    }
                }
                Key::ArrowDown => {
                    let history = command_history.read();
                    if let Some(current_index) = *history_index.read() {
                        if current_index < history.len() - 1 {
                            let new_index = current_index + 1;
                            if let Some(cmd) = history.get(new_index) {
                                input_text.set(cmd.clone());
                                history_index.set(Some(new_index));
                            }
                        } else {
                            input_text.set(String::new());
                            history_index.set(None);
                        }
                    }
                }
                _ => {}
            }
        }
    };

    let terminal_style = "
        display: flex;
        flex-direction: column;
        height: 100%;
        background: #000000;
        color: #cccccc;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: 13px;
    ";

    let output_style = "
        flex: 1;
        overflow-y: auto;
        padding: 10px;
        background: #000000;
        line-height: 1.4;
    ";

    let input_container_style = "
        display: flex;
        align-items: center;
        padding: 8px 10px;
        background: #1a1a1a;
        border-top: 1px solid #333333;
    ";

    let input_style = "
        flex: 1;
        background: transparent;
        border: none;
        color: #cccccc;
        font-family: inherit;
        font-size: inherit;
        outline: none;
        padding: 4px;
    ";

    let prompt_style = "
        color: #4ec9b0;
        margin-right: 8px;
        font-weight: 600;
    ";

    rsx! {
        div {
            style: "{terminal_style}",

            // Output area
            div {
                style: "{output_style}",
                onmounted: move |_| {
                    if should_scroll() {
                        // TODO: Implement scroll to bottom
                        should_scroll.set(false);
                    }
                },

                // Render output lines
                for (idx, line) in output_lines.read().iter().enumerate() {
                    TerminalLineComponent {
                        key: "{idx}",
                        line: line.clone()
                    }
                }
            }

            // Input area
            div {
                style: "{input_container_style}",

                span {
                    style: "{prompt_style}",
                    "$"
                }

                input {
                    style: "{input_style}",
                    r#type: "text",
                    value: "{input_text}",
                    placeholder: "Type a command...",
                    disabled: is_running(),
                    autofocus: true,
                    oninput: move |evt| input_text.set(evt.value()),
                    onkeydown: on_keydown,
                    onkeypress: {
                        let mut input_text = input_text.clone();
                        let mut history_index = history_index.clone();
                        move |evt| {
                            if evt.key() == Key::Enter {
                                let command = input_text.read().clone();
                                execute_command_for_submit(command);
                                input_text.set(String::new());
                                history_index.set(None);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Terminal line data
#[derive(Clone, Debug, PartialEq)]
struct TerminalLine {
    text: String,
    line_type: LineType,
    timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Clone, Debug, PartialEq)]
enum LineType {
    Command,
    Output,
    Error,
    Success,
    Prompt,
}

/// Component for rendering a single terminal line
#[component]
fn TerminalLineComponent(line: TerminalLine) -> Element {
    let line_style = match line.line_type {
        LineType::Command => "color: #569cd6; font-weight: 600;",
        LineType::Output => "color: #cccccc;",
        LineType::Error => "color: #f44747;",
        LineType::Success => "color: #4ec9b0;",
        LineType::Prompt => "color: #dcdcaa; font-weight: 600;",
    };

    let timestamp_style = "color: #666666; font-size: 11px; margin-right: 8px;";

    rsx! {
        div {
            style: "margin: 2px 0; display: flex; align-items: baseline;",

            // Optional timestamp (could be toggled)
            if matches!(line.line_type, LineType::Command | LineType::Error) {
                span {
                    style: "{timestamp_style}",
                    {format!("[{}]", line.timestamp.format("%H:%M:%S"))}
                }
            }

            // Line content
            pre {
                style: "{line_style} margin: 0; font-family: inherit; white-space: pre-wrap; word-break: break-all;",
                "{line.text}"
            }
        }
    }
}