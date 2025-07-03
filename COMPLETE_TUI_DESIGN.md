# Complete TUI Design: Hive AI Terminal Interface

## Executive Summary

The Hive AI TUI provides a **VS Code-like terminal interface** powered by the **exact same 4-stage consensus engine**, **OpenRouter integration**, and **Cloudflare backend** as the current TypeScript implementation. This ensures 100% feature parity while delivering a revolutionary terminal development experience.

## Core Integration Architecture

### **Consensus Engine Integration**
The TUI is built on top of the **identical consensus pipeline** from the current hive.ai implementation:

```rust
pub struct TuiConsensusIntegration {
    // Exact same consensus engine as TypeScript version
    consensus_engine: Arc<FourStageConsensusEngine>,
    openrouter_client: Arc<OpenRouterClient>,
    cloudflare_sync: Arc<CloudflareD1Sync>,
    
    // TUI-specific enhancements
    real_time_display: Arc<RealTimeDisplay>,
    progress_tracker: Arc<ConsensusProgressTracker>,
}

impl TuiConsensusIntegration {
    pub async fn process_consensus_with_ui(
        &self,
        query: &str,
        context: CodeContext,
    ) -> impl Stream<Item = ConsensusEvent> {
        // Same 4-stage pipeline: Generator → Refiner → Validator → Curator
        let consensus_stream = self.consensus_engine.process_with_context(query, context);
        
        // Enhanced with real-time TUI updates
        consensus_stream.map(|event| {
            // Update TUI panels in real-time
            self.real_time_display.update_consensus_panel(&event);
            self.progress_tracker.update_stage_progress(&event);
            event
        })
    }
}
```

### **OpenRouter Integration (Unchanged)**
```rust
pub struct TuiOpenRouterClient {
    // Identical OpenRouter client from TypeScript version
    base_client: Arc<OpenRouterClient>,
    api_key: String,
    base_url: String, // https://openrouter.ai/api/v1
    
    // TUI enhancements for visual feedback
    progress_display: Arc<ModelProgressDisplay>,
    model_selector_ui: Arc<ModelSelectorUI>,
}

impl TuiOpenRouterClient {
    pub async fn stream_consensus_stage(
        &self,
        stage: ConsensusStage,
        prompt: &str,
        model: &str,
    ) -> impl Stream<Item = StreamToken> {
        // Same OpenRouter API calls as TypeScript version
        let stream = self.base_client
            .create_chat_completion_stream(&CreateChatCompletionRequest {
                model: model.to_string(),
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }],
                stream: true,
                // Same headers and configuration as TypeScript
            })
            .await?;
        
        // Enhanced with TUI progress display
        stream.map(|token| {
            self.progress_display.update_token_progress(&token);
            self.update_consensus_panel_real_time(&token);
            token
        })
    }
}
```

### **Cloudflare D1 Integration (Preserved)**
```rust
pub struct TuiCloudflareSync {
    // Identical Cloudflare integration from TypeScript version
    worker_url: String,      // Same worker endpoint
    d1_database_id: String,  // Same D1 database
    api_token: String,       // Same authentication
    
    // TUI status indicators
    sync_status_display: Arc<SyncStatusDisplay>,
    conversation_counter: Arc<ConversationCounter>,
}

impl TuiCloudflareSync {
    pub async fn sync_conversations_with_ui(&self) -> Result<SyncResult> {
        // Update TUI sync status
        self.sync_status_display.show_syncing();
        
        // Identical sync protocol as TypeScript version
        let local_changes = self.get_local_changes_since_last_sync().await?;
        
        let response = reqwest::Client::new()
            .post(&format!("{}/sync", self.worker_url))
            .header("Authorization", &format!("Bearer {}", self.api_token))
            .json(&SyncRequest {
                conversations: local_changes,
                last_sync: self.last_sync_timestamp(),
                // Same payload structure as TypeScript
            })
            .send()
            .await?;
        
        let sync_result: SyncResponse = response.json().await?;
        
        // Apply remote changes with TUI feedback
        self.apply_remote_changes_with_progress(&sync_result.conversations).await?;
        
        // Update TUI status
        self.sync_status_display.show_synced(sync_result.summary);
        self.conversation_counter.update_count(sync_result.total_conversations);
        
        Ok(sync_result)
    }
}
```

## Complete TUI Layout Design

### **Full Interface Layout**
```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ 🐝 HiveTechs Consensus v2.0.0                           [●●●] Models: 323 │ Sync: ✓ │ 25MB │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ File Explorer        │ Code Editor                    │ Consensus Intelligence         │
│ (25% width)          │ (45% width)                    │ (30% width)                    │
│                      │                                │                                │
│ 📁 hive-project/     │ ┌─ src/main.rs ──────────────┐ │ ┌─ 4-Stage Consensus ─────────┐ │
│ ├─ 📁 src/           │ │  1  use hive_ai::*;        │ │ │ Generator → ████░░░░ 60%   │ │
│ │  ├─ 📄 main.rs ◀   │ │  2                         │ │ │ Using: claude-3-opus       │ │
│ │  ├─ 📄 lib.rs      │ │  3  #[tokio::main]        │ │ │                            │ │
│ │  ├─ 📄 consensus.rs│ │  4  async fn main() {     │ │ │ 💬 "Analyzing your Rust   │ │
│ │  └─ 📄 tui.rs      │ │  5      let engine =      │ │ │     code structure..."     │ │
│ ├─ 📁 tests/         │ │  6          ConsensusEngine│ │ └────────────────────────────┘ │
│ ├─ 📄 Cargo.toml     │ │  7              ::new();  │ │                                │
│ ├─ 📄 README.md      │ │  8      engine.process(   │ │ ┌─ Live Code Analysis ─────────┐ │
│ └─ 📄 .gitignore     │ │  9          "analyze"     │ │ │ Language: Rust             │ │
│                      │ │ 10      ).await?;         │ │ │ Lines: 156 (+12 today)     │ │
│ ┌─ Git Status ──────┐│ │ 11  }                     │ │ │ Functions: 8               │ │
│ │ M  src/main.rs    ││ │ 12                        │ │ │ Quality Score: 8.5/10      │ │
│ │ M  src/consensus.rs││ │ 13  cursor here ▌         │ │ │ Complexity: Low            │ │
│ │ A  src/tui.rs     ││ │ 14                        │ │ │ Test Coverage: 85%         │ │
│ │ ?? temp.log       ││ │                           │ │ │ Dependencies: ✓ Secure     │ │
│ │                   ││ │ ╔═══════════════════════╗ │ │ └────────────────────────────┘ │
│ │ 3 staged, 1 new   ││ │ ║ Ask Hive Anything     ║ │ │                                │
│ └───────────────────┘│ │ ╚═══════════════════════╝ │ │ ┌─ Memory & Context ──────────┐ │
│                      │ │ > What design patterns    │ │ │ 🧠 Related Conversations:  │ │
│ ┌─ OpenRouter ──────┐│ │   should I use here?      │ │ │ • "Rust async patterns"    │ │
│ │ Status: ✓ Online ││ │                           │ │ │ • "Consensus architecture" │ │
│ │ Models: 323       ││ │ 🧠 Thinking... (Stage 2/4)│ │ │ • "TUI implementation"     │ │
│ │ Rate Limit: 95%   ││ │                           │ │ │                            │ │
│ │ Cost Today: $2.34 ││ └───────────────────────────┘ │ │ 📊 Project Insights:       │ │
│ └───────────────────┘│                                │ │ • Architecture: Clean      │ │
│                      │                                │ │ • Patterns: Observer       │ │
│ ┌─ Cloudflare ──────┐│                                │ │ • Performance: Excellent   │ │
│ │ Sync: ✓ Real-time││                                │ └────────────────────────────┘ │
│ │ Conversations: 142││                                │                                │
│ │ Last Sync: 2s ago ││                                │ ┌─ Planning Mode ─────────────┐ │
│ │ Conflicts: 0      ││                                │ │ 📋 Current Plan:           │ │
│ └───────────────────┘│                                │ │ ✓ Setup TUI framework      │ │
├──────────────────────┼────────────────────────────────┼────────────────────────────────┤
│ Integrated Terminal (25% height)                                                        │
│                                                                                         │
│ ┌─ Terminal ─────────────────────────────────────────────────────────────────────────┐ │
│ │ $ cargo build                                                                      │ │
│ │    Compiling hive-ai v2.0.0 (/Users/dev/hive)                                    │ │
│ │    Finished dev [unoptimized + debuginfo] target(s) in 4.23s                     │ │
│ │                                                                                    │ │
│ │ $ hive analyze .                                                                   │ │
│ │ 🔍 Analyzing repository structure...                                               │ │
│ │ 🏗️  Architecture detected: Clean Architecture                                     │ │
│ │ 📊 Quality assessment: 8.5/10                                                     │ │
│ │ ✅ Analysis complete - results shown in Consensus panel                           │ │
│ │                                                                                    │ │
│ │ $ git status                                                                       │ │
│ │ On branch main                                                                     │ │
│ │ Changes to be committed:                                                           │ │
│ │   modified:   src/main.rs                                                         │ │
│ │   modified:   src/consensus.rs                                                    │ │
│ │   new file:   src/tui.rs                                                          │ │
│ │                                                                                    │ │
│ │ $ ▌                                                                               │ │
│ └────────────────────────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Status Bar                                                                              │
│ F1:Explorer │ F2:Editor │ F3:Consensus │ F4:Terminal │ Ctrl+P:QuickOpen │ Ready ✅     │
│ Profile: Balanced │ Consensus: Active │ Files: 156 │ Memory: 142 convs │ Sync: ✓     │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

## Consensus Engine TUI Integration

### **Real-Time 4-Stage Display**
```rust
pub struct ConsensusProgressPanel {
    current_stage: ConsensusStage,
    stage_progress: HashMap<ConsensusStage, f32>,
    model_assignments: HashMap<ConsensusStage, String>,
    stage_outputs: HashMap<ConsensusStage, String>,
    streaming_content: String,
}

impl ConsensusProgressPanel {
    pub fn render_consensus_progress(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Stage progress bars
                Constraint::Min(0),     // Streaming content
                Constraint::Length(3),  // Input area
            ])
            .split(area);
        
        // Render 4-stage progress
        self.render_four_stage_progress(frame, chunks[0]);
        
        // Render streaming consensus output
        self.render_streaming_output(frame, chunks[1]);
        
        // Render input area
        self.render_input_area(frame, chunks[2]);
    }
    
    fn render_four_stage_progress(&mut self, frame: &mut Frame, area: Rect) {
        let stages = vec![
            ("Generator", self.stage_progress.get(&ConsensusStage::Generator).unwrap_or(&0.0)),
            ("Refiner", self.stage_progress.get(&ConsensusStage::Refiner).unwrap_or(&0.0)),
            ("Validator", self.stage_progress.get(&ConsensusStage::Validator).unwrap_or(&0.0)),
            ("Curator", self.stage_progress.get(&ConsensusStage::Curator).unwrap_or(&0.0)),
        ];
        
        let stage_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Generator
                Constraint::Length(2),  // Refiner
                Constraint::Length(2),  // Validator
                Constraint::Length(2),  // Curator
            ])
            .split(area);
        
        for (i, (stage_name, progress)) in stages.iter().enumerate() {
            let model = self.model_assignments.get(&ConsensusStage::from(i)).unwrap_or(&"".to_string());
            
            let gauge = Gauge::default()
                .block(Block::default().title(format!("{} → {}", stage_name, model)))
                .gauge_style(Style::default().fg(self.get_stage_color(i)))
                .percent((**progress * 100.0) as u16);
                
            frame.render_widget(gauge, stage_chunks[i]);
        }
    }
}
```

### **OpenRouter Model Selection UI**
```rust
pub struct ModelSelectorUI {
    available_models: Vec<OpenRouterModel>,
    selected_models: HashMap<ConsensusStage, String>,
    model_performance: HashMap<String, ModelPerformance>,
}

impl ModelSelectorUI {
    pub fn render_model_selector(&mut self, frame: &mut Frame, area: Rect) {
        // Show current model assignments for each stage
        let model_assignments: Vec<ListItem> = vec![
            ListItem::new(format!("🎯 Generator: {}", 
                self.selected_models.get(&ConsensusStage::Generator).unwrap_or(&"auto".to_string()))),
            ListItem::new(format!("🔧 Refiner: {}", 
                self.selected_models.get(&ConsensusStage::Refiner).unwrap_or(&"auto".to_string()))),
            ListItem::new(format!("✅ Validator: {}", 
                self.selected_models.get(&ConsensusStage::Validator).unwrap_or(&"auto".to_string()))),
            ListItem::new(format!("✨ Curator: {}", 
                self.selected_models.get(&ConsensusStage::Curator).unwrap_or(&"auto".to_string()))),
            ListItem::new(""),
            ListItem::new(format!("📊 Total Models Available: {}", self.available_models.len())),
            ListItem::new(format!("💰 Cost per 1K tokens: ${:.4}", self.calculate_average_cost())),
            ListItem::new(format!("⚡ Avg Response Time: {}ms", self.calculate_average_latency())),
        ];
        
        let list = List::new(model_assignments)
            .block(Block::default().title("OpenRouter Models").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
            
        frame.render_widget(list, area);
    }
}
```

### **Cloudflare Sync Status Display**
```rust
pub struct CloudflareSyncDisplay {
    sync_status: SyncStatus,
    last_sync_time: Option<SystemTime>,
    conversation_count: usize,
    pending_changes: usize,
    sync_errors: Vec<String>,
}

impl CloudflareSyncDisplay {
    pub fn render_sync_status(&mut self, frame: &mut Frame, area: Rect) {
        let sync_info = vec![
            Line::from(vec![
                Span::styled("☁️  Cloudflare D1: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    match self.sync_status {
                        SyncStatus::Synced => "✓ Synced",
                        SyncStatus::Syncing => "🔄 Syncing...",
                        SyncStatus::Error => "❌ Error",
                        SyncStatus::Offline => "⚠️ Offline",
                    },
                    Style::default().fg(match self.sync_status {
                        SyncStatus::Synced => Color::Green,
                        SyncStatus::Syncing => Color::Yellow,
                        SyncStatus::Error => Color::Red,
                        SyncStatus::Offline => Color::Gray,
                    })
                ),
            ]),
            Line::from(format!("💬 Conversations: {}", self.conversation_count)),
            Line::from(format!("📊 Pending: {}", self.pending_changes)),
            Line::from(format!("🕐 Last Sync: {}", 
                self.last_sync_time
                    .map(|t| format!("{}s ago", t.elapsed().unwrap_or_default().as_secs()))
                    .unwrap_or_else(|| "Never".to_string())
            )),
        ];
        
        let paragraph = Paragraph::new(sync_info)
            .block(Block::default().title("Cloud Sync").borders(Borders::ALL))
            .wrap(Wrap { trim: false });
            
        frame.render_widget(paragraph, area);
    }
}
```

## File Explorer with Git Integration

### **Enhanced File Tree Display**
```rust
pub struct FileExplorerPanel {
    file_tree: FileTree,
    git_status: GitStatus,
    selected_file: Option<PathBuf>,
    expanded_folders: HashSet<PathBuf>,
    file_analysis: HashMap<PathBuf, FileAnalysis>,
}

impl FileExplorerPanel {
    pub fn render_file_explorer(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // File tree
                Constraint::Length(8),  // Git status
                Constraint::Length(6),  // OpenRouter status
                Constraint::Length(4),  // Cloudflare sync
            ])
            .split(area);
        
        // Render file tree with analysis indicators
        self.render_file_tree_with_analysis(frame, chunks[0]);
        
        // Render Git status
        self.render_git_status(frame, chunks[1]);
        
        // Render OpenRouter connection status
        self.render_openrouter_status(frame, chunks[2]);
        
        // Render Cloudflare sync status
        self.render_cloudflare_status(frame, chunks[3]);
    }
    
    fn render_file_tree_with_analysis(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.file_tree.visible_files().iter().map(|file| {
            let indent = "  ".repeat(file.depth);
            let icon = self.get_file_icon(&file.path);
            let git_status = self.git_status.get_status(&file.path);
            let analysis_indicator = self.get_analysis_indicator(&file.path);
            
            let git_char = match git_status {
                GitFileStatus::Modified => " M",
                GitFileStatus::Added => " A",
                GitFileStatus::Deleted => " D",
                GitFileStatus::Untracked => " ?",
                GitFileStatus::Clean => "",
            };
            
            let display_name = format!("{}{} {}{}{}",
                indent,
                icon,
                file.name,
                git_char,
                analysis_indicator
            );
            
            let style = if Some(&file.path) == self.selected_file.as_ref() {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };
            
            ListItem::new(display_name).style(style)
        }).collect();
        
        let list = List::new(items)
            .block(Block::default().title("📁 Explorer").borders(Borders::ALL));
            
        frame.render_widget(list, area);
    }
    
    fn get_analysis_indicator(&self, path: &Path) -> &str {
        if let Some(analysis) = self.file_analysis.get(path) {
            match analysis.quality_score {
                score if score >= 9.0 => " ✨",
                score if score >= 7.0 => " ✓",
                score if score >= 5.0 => " ⚠️",
                _ => " ❌",
            }
        } else {
            ""
        }
    }
}
```

## Code Editor with Syntax Highlighting

### **Advanced Editor Panel**
```rust
pub struct CodeEditorPanel {
    content_lines: Vec<String>,
    cursor: (usize, usize),
    viewport: (usize, usize),
    current_file: Option<PathBuf>,
    language: Option<Language>,
    syntax_highlighter: SyntaxHighlighter,
    file_analysis: Option<FileAnalysis>,
    consensus_suggestions: Vec<ConsensusSuggestion>,
}

impl CodeEditorPanel {
    pub fn render_code_editor(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // File tab
                Constraint::Min(0),     // Editor content
                Constraint::Length(1),  // Cursor position/stats
            ])
            .split(area);
        
        // Render file tab
        self.render_file_tab(frame, chunks[0]);
        
        // Render editor with syntax highlighting
        self.render_editor_content(frame, chunks[1]);
        
        // Render status line
        self.render_editor_status(frame, chunks[2]);
    }
    
    fn render_editor_content(&mut self, frame: &mut Frame, area: Rect) {
        let visible_lines = self.get_visible_lines(area.height as usize);
        
        let highlighted_lines: Vec<Line> = visible_lines.iter().enumerate()
            .map(|(i, line_content)| {
                let line_num = self.viewport.0 + i + 1;
                let is_cursor_line = line_num == self.cursor.0 + 1;
                
                // Apply syntax highlighting
                let spans = if let Some(lang) = &self.language {
                    self.syntax_highlighter.highlight_line(line_content, lang)
                } else {
                    vec![Span::raw(line_content.clone())]
                };
                
                // Add consensus suggestions as inline hints
                let enhanced_spans = self.add_consensus_hints(spans, line_num);
                
                if is_cursor_line {
                    Line::from(enhanced_spans).style(Style::default().bg(Color::DarkGray))
                } else {
                    Line::from(enhanced_spans)
                }
            })
            .collect();
        
        let paragraph = Paragraph::new(highlighted_lines)
            .block(Block::default()
                .title(self.get_editor_title())
                .borders(Borders::ALL))
            .wrap(Wrap { trim: false });
            
        frame.render_widget(paragraph, area);
        
        // Render cursor
        if area.width > 2 && area.height > 2 {
            let cursor_x = area.x + 1 + (self.cursor.1.saturating_sub(self.viewport.1)) as u16;
            let cursor_y = area.y + 1 + (self.cursor.0.saturating_sub(self.viewport.0)) as u16;
            
            if cursor_x < area.x + area.width - 1 && cursor_y < area.y + area.height - 1 {
                frame.set_cursor(cursor_x, cursor_y);
            }
        }
    }
    
    fn add_consensus_hints(&self, spans: Vec<Span>, line_num: usize) -> Vec<Span> {
        let mut enhanced_spans = spans;
        
        // Add inline suggestions from consensus analysis
        for suggestion in &self.consensus_suggestions {
            if suggestion.line == line_num {
                enhanced_spans.push(Span::styled(
                    format!(" 💡 {}", suggestion.hint),
                    Style::default().fg(Color::Yellow).italic()
                ));
            }
        }
        
        enhanced_spans
    }
}
```

## Terminal Integration

### **Integrated Terminal with Hive Commands**
```rust
pub struct IntegratedTerminal {
    command_history: Vec<String>,
    output_buffer: Vec<String>,
    current_input: String,
    cursor_position: usize,
    working_directory: PathBuf,
    hive_integration: Arc<HiveCommandIntegration>,
    shell_process: Option<Child>,
}

impl IntegratedTerminal {
    pub fn render_terminal(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Terminal output
                Constraint::Length(3),  // Command input
            ])
            .split(area);
        
        // Render terminal output
        self.render_terminal_output(frame, chunks[0]);
        
        // Render command input
        self.render_command_input(frame, chunks[1]);
    }
    
    pub async fn execute_command(&mut self, command: &str) -> Result<()> {
        self.command_history.push(format!("$ {}", command));
        self.output_buffer.push(format!("$ {}", command));
        
        if command.starts_with("hive ") {
            // Execute Hive commands with TUI integration
            let hive_output = self.hive_integration
                .execute_hive_command_with_ui(&command[5..])
                .await?;
            
            self.output_buffer.extend(hive_output.lines().map(String::from));
        } else {
            // Execute regular shell commands
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&self.working_directory)
                .output()
                .await?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stdout.is_empty() {
                self.output_buffer.extend(stdout.lines().map(String::from));
            }
            if !stderr.is_empty() {
                self.output_buffer.extend(
                    stderr.lines().map(|line| format!("Error: {}", line))
                );
            }
        }
        
        // Update TUI panels based on command results
        self.update_tui_panels_from_command_output(command).await?;
        
        Ok(())
    }
}
```

## Event Handling and Key Bindings

### **VS Code-like Key Bindings**
```rust
pub struct TuiEventHandler {
    focused_panel: FocusedPanel,
    key_bindings: KeyBindingConfig,
    command_palette: CommandPalette,
    quick_open: QuickOpenDialog,
}

impl TuiEventHandler {
    pub async fn handle_key_event(&mut self, key: KeyEvent, app: &mut TuiApp) -> Result<bool> {
        match (key.code, key.modifiers) {
            // Global shortcuts (VS Code style)
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.quick_open.toggle();
                Ok(false)
            }
            
            (KeyCode::Char('P'), KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                self.command_palette.toggle();
                Ok(false)
            }
            
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                // Save any pending changes before exit
                app.save_all_modified_files().await?;
                Ok(true) // Exit
            }
            
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                app.save_current_file().await?;
                Ok(false)
            }
            
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                app.create_new_file().await?;
                Ok(false)
            }
            
            // Panel focus (F-keys)
            (KeyCode::F(1), _) => {
                self.focused_panel = FocusedPanel::Explorer;
                Ok(false)
            }
            (KeyCode::F(2), _) => {
                self.focused_panel = FocusedPanel::Editor;
                Ok(false)
            }
            (KeyCode::F(3), _) => {
                self.focused_panel = FocusedPanel::Consensus;
                Ok(false)
            }
            (KeyCode::F(4), _) => {
                self.focused_panel = FocusedPanel::Terminal;
                Ok(false)
            }
            
            // Panel-specific handling
            _ => {
                match self.focused_panel {
                    FocusedPanel::Explorer => {
                        app.file_explorer.handle_key_event(key).await
                    }
                    FocusedPanel::Editor => {
                        app.code_editor.handle_key_event(key).await
                    }
                    FocusedPanel::Consensus => {
                        app.consensus_panel.handle_key_event(key).await
                    }
                    FocusedPanel::Terminal => {
                        app.terminal.handle_key_event(key).await
                    }
                }?;
                Ok(false)
            }
        }
    }
}
```

## Configuration and Customization

### **TUI Configuration Options**
```toml
# ~/.hive/config.toml

[interface]
# Enable TUI mode when running standalone
tui_mode = true
prefer_tui = true

[interface.tui]
# Layout configuration
explorer_width = 25      # Percentage
consensus_width = 30     # Percentage  
terminal_height = 25     # Percentage
editor_line_numbers = true
editor_word_wrap = false

# Visual preferences
theme = "dark"           # "dark", "light", "solarized", "monokai"
mouse_enabled = true
cursor_blink = true
syntax_highlighting = true

# Performance settings
max_terminal_history = 1000
auto_save_interval = 30  # seconds
file_watch_enabled = true

[interface.tui.consensus]
# Consensus panel settings
show_stage_progress = true
show_model_assignments = true
show_token_count = true
show_cost_estimates = true
auto_scroll_output = true

# Real-time analysis
analyze_on_file_change = true
show_quality_indicators = true
show_inline_suggestions = true

[interface.tui.keys]
# Key binding customization
quit = "Ctrl+Q"
save = "Ctrl+S"
new_file = "Ctrl+N"
close_file = "Ctrl+W"
command_palette = "Ctrl+Shift+P"
quick_open = "Ctrl+P"
find_in_file = "Ctrl+F"
find_in_project = "Ctrl+Shift+F"

# Panel focus
focus_explorer = "F1"
focus_editor = "F2"
focus_consensus = "F3"
focus_terminal = "F4"

# Consensus shortcuts
ask_hive = "Ctrl+H"
analyze_file = "Ctrl+A"
plan_mode = "Ctrl+Shift+P"
```

## Integration with Existing Hive.ai Infrastructure

### **Database Compatibility**
```rust
// Uses identical database schema and sync as TypeScript version
pub struct TuiDatabaseIntegration {
    // Same SQLite schema as TypeScript implementation
    local_db: Arc<SqliteDatabase>,
    
    // Same Cloudflare D1 sync protocol
    cloudflare_sync: Arc<CloudflareD1Sync>,
    
    // TUI-specific enhancements
    real_time_updates: Arc<RealTimeUpdates>,
    ui_notifications: Arc<UiNotifications>,
}

impl TuiDatabaseIntegration {
    pub async fn store_conversation_with_ui_update(
        &self,
        conversation: &Conversation,
    ) -> Result<()> {
        // Store using identical schema as TypeScript version
        let conversation_id = self.local_db.store_conversation(conversation).await?;
        
        // Sync to Cloudflare D1 using same protocol
        self.cloudflare_sync.sync_conversation(conversation_id).await?;
        
        // Update TUI panels
        self.real_time_updates.update_conversation_count();
        self.ui_notifications.show_sync_success();
        
        Ok(())
    }
}
```

### **Model Selection and Routing**
```rust
// Uses identical OpenRouter integration as TypeScript version
pub struct TuiModelRouter {
    // Same OpenRouter client and configuration
    openrouter_client: Arc<OpenRouterClient>,
    
    // Same model selection logic
    model_selector: Arc<ModelSelector>,
    
    // TUI enhancements
    model_performance_display: Arc<ModelPerformanceDisplay>,
    cost_tracker_ui: Arc<CostTrackerUI>,
}

impl TuiModelRouter {
    pub async fn select_models_for_consensus_with_ui(
        &self,
        complexity: QueryComplexity,
    ) -> Result<ConsensusModelAssignment> {
        // Use identical model selection logic as TypeScript version
        let assignment = self.model_selector
            .select_models_for_stages(complexity)
            .await?;
        
        // Update TUI display
        self.model_performance_display.update_assignments(&assignment);
        self.cost_tracker_ui.estimate_cost(&assignment);
        
        Ok(assignment)
    }
}
```

## Launch and Detection Logic

### **Automatic TUI Launch Detection**
```rust
impl TuiLauncher {
    pub async fn should_launch_tui() -> bool {
        // Check explicit environment variable
        if std::env::var("HIVE_TUI").is_ok() {
            return true;
        }
        
        // Check if running standalone
        let is_standalone = std::env::args().len() == 1;
        
        // Check terminal capabilities
        let terminal_capable = Self::check_terminal_capabilities();
        
        // Check user preference
        let tui_enabled = Self::check_tui_preference().await;
        
        // Check if we're in a real terminal (not piped)
        let in_terminal = atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stdin);
        
        is_standalone && terminal_capable && tui_enabled && in_terminal
    }
    
    fn check_terminal_capabilities() -> bool {
        // Minimum terminal size for TUI
        if let Ok((width, height)) = crossterm::terminal::size() {
            width >= 120 && height >= 30
        } else {
            false
        }
    }
    
    pub async fn launch_tui_with_full_integration() -> Result<()> {
        // Initialize all components with same configuration as CLI
        let consensus_engine = ConsensusEngine::initialize().await?;
        let openrouter_client = OpenRouterClient::initialize().await?;
        let cloudflare_sync = CloudflareD1Sync::initialize().await?;
        
        // Launch TUI with full Hive.ai integration
        let tui_app = TuiApp::new_with_integrations(
            consensus_engine,
            openrouter_client,
            cloudflare_sync,
        ).await?;
        
        tui_app.run().await?;
        
        Ok(())
    }
}
```

## Summary

This complete TUI design provides:

### **🔄 100% Hive.ai Integration**
- **Identical 4-stage consensus engine** (Generator → Refiner → Validator → Curator)
- **Same OpenRouter API integration** with 323+ models
- **Same Cloudflare D1 sync protocol** for conversation storage
- **Identical database schema** for seamless migration

### **🖥️ VS Code-like Experience**
- **File explorer** with Git status and quality indicators
- **Syntax-highlighted editor** with consensus suggestions
- **Integrated terminal** with Hive command support
- **Real-time consensus panel** showing 4-stage progress

### **⚡ Enhanced Intelligence**
- **Live code analysis** as you type
- **Context-aware suggestions** from consensus engine
- **Real-time quality scoring** and metrics
- **Integrated planning mode** visualization

### **🎯 Seamless Workflow**
- **Automatic detection** of TUI-capable terminals
- **VS Code keybindings** (Ctrl+P, Ctrl+Shift+P, F-keys)
- **Real-time sync status** and conversation count
- **Integrated memory search** and analytics

The TUI mode transforms Hive AI from a CLI tool into a **complete development environment** while maintaining 100% compatibility with the existing TypeScript implementation's core functionality, database, and cloud services.