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

    // Initialize with prompt
    use_effect(move || {
        if output_lines.read().is_empty() {
            output_lines.write().push_back(TerminalLine {
                text: format!("{}> ", current_directory.read()),
                line_type: LineType::Prompt,
                timestamp: Local::now(),
            });
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

    // Handle input submission
    let on_submit = move |_| {
        let command = input_text.read().clone();
        execute_command(command);
        input_text.set(String::new());
        history_index.set(None);
    };

    // Handle key events for history navigation
    let on_keydown = move |evt: Event<KeyboardData>| {
        match evt.data().key() {
            Key::ArrowUp => {
                let history = command_history.read();
                if !history.is_empty() {
                    let current_index = history_index.read().unwrap_or(history.len());
                    if current_index > 0 {
                        let new_index = current_index - 1;
                        history_index.set(Some(new_index));
                        if let Some(cmd) = history.get(new_index) {
                            input_text.set(cmd.clone());
                        }
                    }
                }
            }
            Key::ArrowDown => {
                let history = command_history.read();
                if let Some(current_index) = history_index.read() {
                    if current_index < history.len() - 1 {
                        let new_index = current_index + 1;
                        history_index.set(Some(new_index));
                        if let Some(cmd) = history.get(new_index) {
                            input_text.set(cmd.clone());
                        }
                    } else {
                        history_index.set(None);
                        input_text.set(String::new());
                    }
                }
            }
            _ => {}
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
                    onkeypress: move |evt| {
                        if evt.key() == Key::Enter {
                            on_submit(evt);
                        }
                    }
                }
            }
        }
    }
}

/// Terminal line data
#[derive(Clone, Debug)]
struct TerminalLine {
    text: String,
    line_type: LineType,
    timestamp: chrono::DateTime<chrono::Local>,
}

#[derive(Clone, Debug)]
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