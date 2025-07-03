# Hive AI Terminal User Interface (TUI)

## Overview

A **full-featured terminal interface** that provides a VS Code-like experience entirely in the terminal, built with Rust TUI libraries. This activates when Hive detects it's running in its own terminal session.

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 🐝 HiveTechs Consensus v2.0.0                    [●●●] Config | Memory | CPU │
├─────────────────────────────────────────────────────────────────────────────┤
│ Explorer          │ Editor                       │ Consensus              │
│                   │                              │                        │
│ 📁 project/       │ ┌─ src/main.rs ─────────────┐│ ┌─ Ask Hive ─────────┐ │
│ ├─ 📁 src/        │ │ fn main() {                ││ │ > What does this   │ │
│ │  ├─ 📄 main.rs  │ │     println!("Hello");     ││ │   code do?         │ │
│ │  ├─ 📄 lib.rs   │ │ }                          ││ │                    │ │
│ │  └─ 📄 mod.rs   │ │                            ││ │ 🧠 Analyzing...    │ │
│ ├─ 📁 tests/      │ │ cursor here ▌              ││ └────────────────────┘ │
│ ├─ 📄 Cargo.toml  │ │                            ││                        │
│ └─ 📄 README.md   │ └────────────────────────────┘│ ┌─ File Analysis ────┐ │
│                   │                              │ │ Language: Rust     │ │
│ ┌─ Git Status ───┐│ ┌─ Terminal ──────────────────┐│ │ Lines: 156         │ │
│ │ M src/main.rs  ││ │ $ cargo build              ││ │ Functions: 8       │ │
│ │ ? src/new.rs   ││ │ Compiling project v0.1.0   ││ │ Quality: 8.5/10    │ │
│ │ 2 files staged ││ │ Finished dev [unoptimized] ││ └────────────────────┘ │
│ └────────────────┘│ │ target(s) in 2.34s         ││                        │
│                   │ │ $ hive analyze .           ││ ┌─ Plan Mode ────────┐ │
│                   │ │ 🔍 Analyzing repository... ││ │ □ Parse config     │ │
│                   │ │ ✅ Analysis complete       ││ │ □ Validate input   │ │
│                   │ │ >                          ││ │ ✅ Generate output │ │
│                   │ └────────────────────────────┘│ │ □ Write tests      │ │
│                   │                              │ └────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│ Status: Ready │ Consensus: Balanced │ Models: 323 │ Memory: 142 convs │ 25MB │
└─────────────────────────────────────────────────────────────────────────────┘
```

## TUI Activation Logic

```rust
impl TuiLauncher {
    pub async fn should_launch_tui() -> bool {
        // Check if running in standalone terminal
        let is_standalone = Self::detect_standalone_terminal();
        
        // Check if user prefers TUI mode
        let tui_enabled = Self::check_tui_preference().await;
        
        // Check terminal capabilities
        let terminal_capable = Self::check_terminal_capabilities();
        
        is_standalone && tui_enabled && terminal_capable
    }
    
    fn detect_standalone_terminal() -> bool {
        // Detect if we're the main process in the terminal
        let is_main_process = std::env::var("HIVE_TUI").is_ok() || 
                             Self::is_primary_terminal_process();
        
        // Check terminal size (TUI needs reasonable space)
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        let has_space = width >= 120 && height >= 30;
        
        is_main_process && has_space
    }
    
    async fn check_tui_preference() -> bool {
        // Check config file for TUI preference
        if let Ok(config) = load_config().await {
            config.interface.tui_mode.unwrap_or(true)
        } else {
            true // Default to TUI if available
        }
    }
}
```

## Interface Components

### 1. File Explorer (Left Panel)
```rust
pub struct FileExplorer {
    tree: FileTree,
    selected_file: Option<PathBuf>,
    expanded_dirs: HashSet<PathBuf>,
    git_status: GitStatus,
}

impl FileExplorer {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.tree.files.iter().map(|file| {
            let icon = match file.extension() {
                Some("rs") => "🦀",
                Some("js") | Some("ts") => "📜",
                Some("json") => "📋",
                Some("md") => "📝",
                _ if file.is_dir() => "📁",
                _ => "📄",
            };
            
            let git_indicator = match self.git_status.get_status(file) {
                GitFileStatus::Modified => " M",
                GitFileStatus::Added => " A", 
                GitFileStatus::Untracked => " ?",
                GitFileStatus::Clean => "",
            };
            
            ListItem::new(format!("{} {}{}", icon, file.display(), git_indicator))
        }).collect();
        
        let list = List::new(items)
            .block(Block::default().title("Explorer").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::Blue));
            
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}
```

### 2. Code Editor (Center Panel)
```rust
pub struct CodeEditor {
    content: Vec<String>,
    cursor: (usize, usize), // (line, column)
    viewport: (usize, usize), // (start_line, start_col)
    file_path: Option<PathBuf>,
    language: Option<Language>,
    syntax_highlighter: SyntaxHighlighter,
}

impl CodeEditor {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let visible_lines = self.get_visible_lines(area.height as usize);
        
        let highlighted_lines: Vec<Line> = visible_lines.iter().enumerate()
            .map(|(i, line)| {
                let line_num = self.viewport.0 + i + 1;
                let is_cursor_line = line_num == self.cursor.0 + 1;
                
                let spans = if let Some(lang) = &self.language {
                    self.syntax_highlighter.highlight(line, lang)
                } else {
                    vec![Span::raw(line.clone())]
                };
                
                if is_cursor_line {
                    Line::from(spans).style(Style::default().bg(Color::DarkGray))
                } else {
                    Line::from(spans)
                }
            })
            .collect();
        
        let paragraph = Paragraph::new(highlighted_lines)
            .block(Block::default()
                .title(self.get_title())
                .borders(Borders::ALL))
            .wrap(Wrap { trim: false });
            
        frame.render_widget(paragraph, area);
        
        // Render cursor
        let cursor_x = area.x + 1 + (self.cursor.1 - self.viewport.1) as u16;
        let cursor_y = area.y + 1 + (self.cursor.0 - self.viewport.0) as u16;
        frame.set_cursor(cursor_x, cursor_y);
    }
}
```

### 3. Consensus Panel (Right Panel)
```rust
pub struct ConsensusPanel {
    current_mode: ConsensusMode,
    chat_history: Vec<ChatMessage>,
    input_buffer: String,
    file_analysis: Option<FileAnalysis>,
    plan_tasks: Vec<PlanTask>,
    active_tab: ConsensusPanelTab,
}

#[derive(Clone)]
pub enum ConsensusPanelTab {
    Chat,
    Analysis,
    Planning,
    Memory,
}

impl ConsensusPanel {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tab bar
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Input
            ])
            .split(area);
        
        // Render tab bar
        self.render_tabs(frame, chunks[0]);
        
        // Render active tab content
        match self.active_tab {
            ConsensusPanelTab::Chat => self.render_chat(frame, chunks[1]),
            ConsensusPanelTab::Analysis => self.render_analysis(frame, chunks[1]),
            ConsensusPanelTab::Planning => self.render_planning(frame, chunks[1]),
            ConsensusPanelTab::Memory => self.render_memory(frame, chunks[1]),
        }
        
        // Render input area
        self.render_input(frame, chunks[2]);
    }
    
    fn render_chat(&mut self, frame: &mut Frame, area: Rect) {
        let messages: Vec<ListItem> = self.chat_history.iter().map(|msg| {
            let prefix = match msg.sender {
                MessageSender::User => "👤 You:",
                MessageSender::Hive => "🐝 Hive:",
            };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(Span::raw(&msg.content)),
                Line::raw(""),
            ])
        }).collect();
        
        let list = List::new(messages)
            .block(Block::default().title("Consensus Chat").borders(Borders::ALL));
            
        frame.render_widget(list, area);
    }
}
```

### 4. Integrated Terminal (Bottom Panel)
```rust
pub struct IntegratedTerminal {
    history: Vec<String>,
    current_command: String,
    cursor_pos: usize,
    working_dir: PathBuf,
    process: Option<Child>,
}

impl IntegratedTerminal {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Output
                Constraint::Length(3),  // Input
            ])
            .split(area);
        
        // Render command history
        let history_text: Vec<Line> = self.history.iter()
            .map(|line| {
                if line.starts_with("$ ") {
                    Line::from(vec![
                        Span::styled("$ ", Style::default().fg(Color::Green)),
                        Span::raw(&line[2..]),
                    ])
                } else {
                    Line::raw(line.clone())
                }
            })
            .collect();
        
        let paragraph = Paragraph::new(history_text)
            .block(Block::default().title("Terminal").borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((0, 0));
            
        frame.render_widget(paragraph, chunks[0]);
        
        // Render command input
        let input = Paragraph::new(format!("$ {}", self.current_command))
            .block(Block::default().borders(Borders::ALL));
            
        frame.render_widget(input, chunks[1]);
    }
    
    pub async fn execute_command(&mut self, command: &str) -> Result<()> {
        self.history.push(format!("$ {}", command));
        
        // Special handling for Hive commands
        if command.starts_with("hive ") {
            let output = self.execute_hive_command(&command[5..]).await?;
            self.history.push(output);
        } else {
            // Execute regular shell command
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&self.working_dir)
                .output()
                .await?;
                
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stdout.is_empty() {
                self.history.push(stdout.to_string());
            }
            if !stderr.is_empty() {
                self.history.push(format!("Error: {}", stderr));
            }
        }
        
        Ok(())
    }
}
```

## Event Handling

```rust
pub struct TuiEventHandler {
    current_focus: FocusedPanel,
    key_bindings: KeyBindings,
}

#[derive(Clone, Copy)]
pub enum FocusedPanel {
    Explorer,
    Editor,
    Consensus,
    Terminal,
}

impl TuiEventHandler {
    pub async fn handle_key_event(&mut self, key: KeyEvent, app: &mut TuiApp) -> Result<bool> {
        match key.code {
            // Global hotkeys
            KeyCode::F(1) => {
                self.current_focus = FocusedPanel::Explorer;
            }
            KeyCode::F(2) => {
                self.current_focus = FocusedPanel::Editor;
            }
            KeyCode::F(3) => {
                self.current_focus = FocusedPanel::Consensus;
            }
            KeyCode::F(4) => {
                self.current_focus = FocusedPanel::Terminal;
            }
            
            // Ctrl+Q to quit
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(true); // Exit TUI
            }
            
            // Ctrl+P for quick file open (like VS Code)
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.open_quick_file_dialog();
            }
            
            // Ctrl+Shift+P for command palette
            KeyCode::Char('P') if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                app.open_command_palette();
            }
            
            // Panel-specific handling
            _ => {
                match self.current_focus {
                    FocusedPanel::Explorer => app.explorer.handle_key(key).await?,
                    FocusedPanel::Editor => app.editor.handle_key(key).await?,
                    FocusedPanel::Consensus => app.consensus_panel.handle_key(key).await?,
                    FocusedPanel::Terminal => app.terminal.handle_key(key).await?,
                }
            }
        }
        
        Ok(false)
    }
}
```

## TUI Launch Detection

```rust
impl HiveTui {
    pub async fn main() -> Result<()> {
        // Initialize TUI
        let mut terminal = Self::setup_terminal()?;
        let mut app = TuiApp::new().await?;
        let mut event_handler = TuiEventHandler::new();
        
        // Main event loop
        loop {
            // Render UI
            terminal.draw(|frame| {
                app.render(frame);
            })?;
            
            // Handle events
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    if event_handler.handle_key_event(key, &mut app).await? {
                        break; // Exit requested
                    }
                }
            }
            
            // Update app state
            app.update().await?;
        }
        
        // Cleanup
        Self::restore_terminal(&mut terminal)?;
        Ok(())
    }
    
    fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        Ok(Terminal::new(backend)?)
    }
}
```

## Configuration

```toml
# .hive/config.toml
[interface]
# Enable TUI mode when running standalone
tui_mode = true

# TUI layout preferences
[interface.tui]
# Panel sizes (percentages)
explorer_width = 25
consensus_width = 30
terminal_height = 25

# Theme
theme = "dark"  # "dark", "light", "solarized"

# Key bindings
[interface.tui.keys]
quit = "Ctrl+Q"
command_palette = "Ctrl+Shift+P"
quick_open = "Ctrl+P"
focus_explorer = "F1"
focus_editor = "F2" 
focus_consensus = "F3"
focus_terminal = "F4"
```

## Required Dependencies

```toml
# TUI dependencies
ratatui = "0.25"
crossterm = "0.27"
tui-textarea = "0.4"
syntect = "5.1"  # Syntax highlighting
tokio = { version = "1.35", features = ["full"] }

# File watching
notify = "6.1"

# Git integration
git2 = "0.18"
```

## Launch Command

```bash
# Regular CLI mode
hive ask "What does this do?"

# Force TUI mode
hive --tui
# or
hive tui

# Launch in new terminal with TUI
HIVE_TUI=1 hive
```

## Features

### **VS Code-like Experience:**
- 🗂️ **File Explorer** with Git status indicators
- 📝 **Code Editor** with syntax highlighting and cursor
- 🧠 **Consensus Panel** with chat, analysis, planning tabs
- 💻 **Integrated Terminal** with command execution
- ⌨️ **Familiar Keybindings** (Ctrl+P, Ctrl+Shift+P, etc.)

### **Hive AI Integration:**
- 🔍 **Real-time Analysis** of open files
- 📋 **Planning Mode** with task visualization
- 💬 **Consensus Chat** integrated into the interface
- 📊 **Live Metrics** in status bar
- 🧠 **Memory Access** with conversation history

### **Advanced Features:**
- 🎨 **Syntax Highlighting** for 50+ languages
- 🔄 **File Watching** for auto-refresh
- 📈 **Git Integration** with status and diffs
- 🎯 **Quick Open** and command palette
- 🖱️ **Mouse Support** for clicking and scrolling

This TUI mode would make Hive AI the **ultimate terminal-based development environment** - combining the power of VS Code's interface with Hive AI's consensus intelligence, all running natively in the terminal!