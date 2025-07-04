//! Application State Management

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Main application state
#[derive(Clone, Debug)]
pub struct AppState {
    /// Current project information
    pub current_project: Option<ProjectInfo>,
    
    /// File explorer state
    pub file_explorer: FileExplorerState,
    
    /// Chat interface state
    pub chat: ChatState,
    
    /// Consensus engine state
    pub consensus: ConsensusState,
    
    /// Application settings
    pub settings: AppSettings,
    
    /// Connection status
    pub connection_status: ConnectionStatus,
    
    /// Cost tracking
    pub total_cost: f64,
    
    /// Context usage percentage
    pub context_usage: u8,
    
    /// Auto-accept edits setting
    pub auto_accept: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_project: None,
            file_explorer: FileExplorerState::new(),
            chat: ChatState::new(),
            consensus: ConsensusState::new(),
            settings: AppSettings::default(),
            connection_status: ConnectionStatus::Disconnected,
            total_cost: 0.0,
            context_usage: 0,
            auto_accept: true,
        }
    }
    
    /// Load a project from a directory
    pub async fn load_project(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let project = ProjectInfo::from_path(path).await?;
        self.current_project = Some(project);
        self.file_explorer.refresh().await?;
        Ok(())
    }
    
    /// Update connection status
    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }
    
    /// Add cost to total
    pub fn add_cost(&mut self, cost: f64) {
        self.total_cost += cost;
    }
    
    /// Update context usage
    pub fn set_context_usage(&mut self, usage: u8) {
        self.context_usage = usage.min(100);
    }
}

/// Project information
#[derive(Clone, Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub language: Option<String>,
    pub git_status: GitStatus,
    pub file_count: usize,
}

impl ProjectInfo {
    pub async fn from_path(path: PathBuf) -> anyhow::Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
            
        // Detect primary language
        let language = detect_primary_language(&path).await;
        
        // Check Git status
        let git_status = check_git_status(&path).await;
        
        // Count files
        let file_count = count_project_files(&path).await;
        
        Ok(Self {
            name,
            path,
            language,
            git_status,
            file_count,
        })
    }
}

/// File explorer state
#[derive(Clone, Debug)]
pub struct FileExplorerState {
    pub root_path: Option<PathBuf>,
    pub expanded_dirs: HashMap<PathBuf, bool>,
    pub selected_file: Option<PathBuf>,
    pub files: Vec<FileItem>,
    pub filter: String,
    pub show_hidden: bool,
}

impl FileExplorerState {
    pub fn new() -> Self {
        Self {
            root_path: None,
            expanded_dirs: HashMap::new(),
            selected_file: None,
            files: Vec::new(),
            filter: String::new(),
            show_hidden: false,
        }
    }
    
    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        if let Some(root) = &self.root_path {
            self.files = load_directory_tree(root, &self.expanded_dirs, self.show_hidden).await?;
        }
        Ok(())
    }
    
    pub fn toggle_directory(&mut self, path: &PathBuf) {
        let expanded = self.expanded_dirs.get(path).copied().unwrap_or(false);
        self.expanded_dirs.insert(path.clone(), !expanded);
    }
    
    pub fn select_file(&mut self, path: PathBuf) {
        self.selected_file = Some(path);
    }
}

/// File item in the explorer
#[derive(Clone, Debug)]
pub struct FileItem {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub children: Vec<FileItem>,
    pub file_type: FileType,
    pub git_status: Option<GitFileStatus>,
    pub size: Option<u64>,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
}

/// File type for syntax highlighting and icons
#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    CPP,
    C,
    HTML,
    CSS,
    JSON,
    TOML,
    YAML,
    Markdown,
    Text,
    Binary,
    Directory,
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "ts" | "tsx" => Self::TypeScript,
            "js" | "jsx" => Self::JavaScript,
            "py" => Self::Python,
            "go" => Self::Go,
            "java" => Self::Java,
            "cpp" | "cc" | "cxx" => Self::CPP,
            "c" | "h" => Self::C,
            "html" | "htm" => Self::HTML,
            "css" => Self::CSS,
            "json" => Self::JSON,
            "toml" => Self::TOML,
            "yaml" | "yml" => Self::YAML,
            "md" => Self::Markdown,
            "txt" => Self::Text,
            _ => Self::Unknown,
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Rust => "🦀",
            Self::TypeScript => "📘",
            Self::JavaScript => "📙",
            Self::Python => "🐍",
            Self::Go => "🐹",
            Self::Java => "☕",
            Self::CPP | Self::C => "⚙️",
            Self::HTML => "🌐",
            Self::CSS => "🎨",
            Self::JSON => "📋",
            Self::TOML => "⚙️",
            Self::YAML => "📄",
            Self::Markdown => "📝",
            Self::Text => "📄",
            Self::Binary => "📦",
            Self::Directory => "📁",
            Self::Unknown => "❓",
        }
    }
}

/// Chat interface state
#[derive(Clone, Debug)]
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub input_text: String,
    pub is_processing: bool,
    pub scroll_position: f32,
    pub selected_message: Option<usize>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_text: String::new(),
            is_processing: false,
            scroll_position: 0.0,
            selected_message: None,
        }
    }
    
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.scroll_to_bottom();
    }
    
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = 1.0; // Bottom
    }
    
    pub fn clear_input(&mut self) {
        self.input_text.clear();
    }
}

/// Chat message
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub id: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: MessageMetadata,
}

#[derive(Clone, Debug)]
pub enum MessageType {
    User,
    Assistant,
    System,
    Error,
    Welcome,
}

#[derive(Clone, Debug, Default)]
pub struct MessageMetadata {
    pub cost: Option<f64>,
    pub model: Option<String>,
    pub processing_time: Option<u64>,
    pub token_count: Option<u32>,
}

/// Consensus engine state
#[derive(Clone, Debug)]
pub struct ConsensusState {
    pub is_active: bool,
    pub current_stage: Option<ConsensusStage>,
    pub progress: ConsensusProgress,
    pub stages: Vec<StageInfo>,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            is_active: false,
            current_stage: None,
            progress: ConsensusProgress::default(),
            stages: vec![
                StageInfo::new("Generator", "claude-3-5-sonnet"),
                StageInfo::new("Refiner", "gpt-4-turbo"),
                StageInfo::new("Validator", "claude-3-opus"),
                StageInfo::new("Curator", "gpt-4o"),
            ],
        }
    }
    
    pub fn start_consensus(&mut self) {
        self.is_active = true;
        self.current_stage = Some(ConsensusStage::Generator);
        self.progress = ConsensusProgress::default();
    }
    
    pub fn update_progress(&mut self, stage: ConsensusStage, progress: u8) {
        match stage {
            ConsensusStage::Generator => self.progress.generator = progress,
            ConsensusStage::Refiner => self.progress.refiner = progress,
            ConsensusStage::Validator => self.progress.validator = progress,
            ConsensusStage::Curator => self.progress.curator = progress,
        }
    }
    
    pub fn complete_consensus(&mut self) {
        self.is_active = false;
        self.current_stage = None;
    }
}

#[derive(Clone, Debug)]
pub enum ConsensusStage {
    Generator,
    Refiner,
    Validator,
    Curator,
}

#[derive(Clone, Debug, Default)]
pub struct ConsensusProgress {
    pub generator: u8,
    pub refiner: u8,
    pub validator: u8,
    pub curator: u8,
}

#[derive(Clone, Debug)]
pub struct StageInfo {
    pub name: String,
    pub model: String,
    pub status: StageStatus,
    pub progress: u8,
}

impl StageInfo {
    pub fn new(name: &str, model: &str) -> Self {
        Self {
            name: name.to_string(),
            model: model.to_string(),
            status: StageStatus::Waiting,
            progress: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum StageStatus {
    Waiting,
    Running,
    Completed,
    Error,
}

/// Application settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub font_size: f32,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub auto_save: bool,
    pub consensus_auto_start: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            font_size: 14.0,
            show_line_numbers: true,
            word_wrap: false,
            auto_save: true,
            consensus_auto_start: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    HighContrast,
}

/// Connection status
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
}

/// Git status
#[derive(Clone, Debug)]
pub enum GitStatus {
    NotRepository,
    Repository { branch: String, has_changes: bool },
}

#[derive(Clone, Debug)]
pub enum GitFileStatus {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Ignored,
}

// Utility functions (to be implemented)
async fn detect_primary_language(_path: &PathBuf) -> Option<String> {
    // TODO: Implement language detection
    Some("Rust".to_string())
}

async fn check_git_status(_path: &PathBuf) -> GitStatus {
    // TODO: Implement Git status checking
    GitStatus::NotRepository
}

async fn count_project_files(_path: &PathBuf) -> usize {
    // TODO: Implement file counting
    0
}

async fn load_directory_tree(
    _root: &PathBuf,
    _expanded: &HashMap<PathBuf, bool>,
    _show_hidden: bool,
) -> anyhow::Result<Vec<FileItem>> {
    // TODO: Implement directory tree loading
    Ok(Vec::new())
}