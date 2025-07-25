//! Application State Management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use dioxus::prelude::*;

/// Global profile state - single source of truth
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileState {
    Loading,
    Loaded(ActiveProfileData),
    Error(String),
}

/// Active profile data from database
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveProfileData {
    pub name: String,
    pub generator_model: String,
    pub refiner_model: String,
    pub validator_model: String,
    pub curator_model: String,
}

/// Global signal for profile state
pub static ACTIVE_PROFILE_STATE: GlobalSignal<ProfileState> = Signal::global(|| ProfileState::Loading);

/// Main application state
#[derive(Clone, Debug, PartialEq)]
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

    /// Current model being used
    pub current_model: Option<String>,

    /// User license information
    pub user_id: Option<String>,
    pub license_tier: String,

    /// Usage tracking
    pub daily_conversations_used: u32,
    pub daily_conversations_limit: u32,
    pub total_conversations_remaining: Option<u32>, // From D1 (includes credits)
    pub is_trial_active: bool,
    pub trial_days_remaining: Option<i32>,

    /// Trigger for subscription display refresh
    pub subscription_refresh_trigger: u32,

    /// Trigger for analytics refresh after consensus completion
    pub analytics_refresh_trigger: u32,
    
    /// Trigger for repository context update when directory selection changes
    pub repository_context_update_trigger: u32,
    
    /// Pending file operations requiring user confirmation
    pub pending_operations: Option<Vec<crate::consensus::ai_operation_parser::FileOperationWithMetadata>>,
    
    /// Show operation confirmation dialog
    pub show_operation_confirmation_dialog: bool,
    
    /// Claude execution mode: Direct, ConsensusAssisted, or ConsensusRequired
    pub claude_execution_mode: String,
    
    /// Claude authentication method: ApiKey or OAuth
    pub claude_auth_method: String,
    
    /// Claude Code native mode: normal, auto-accept, plan
    pub claude_mode: String,
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
            current_model: Some("claude-3-5-sonnet".to_string()),
            user_id: None,
            license_tier: "free".to_string(),
            daily_conversations_used: 0,
            daily_conversations_limit: 10,
            total_conversations_remaining: None,
            is_trial_active: false,
            trial_days_remaining: None,
            subscription_refresh_trigger: 0,
            analytics_refresh_trigger: 0,
            repository_context_update_trigger: 0,
            pending_operations: None,
            show_operation_confirmation_dialog: false,
            claude_execution_mode: "ConsensusAssisted".to_string(),
            claude_auth_method: "NotSelected".to_string(),
            claude_mode: "normal".to_string(),
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

    /// Update usage tracking information
    pub fn update_usage_info(
        &mut self,
        user_id: Option<String>,
        tier: &str,
        used: u32,
        limit: u32,
        is_trial: bool,
        trial_days: Option<i32>,
    ) {
        self.user_id = user_id;
        self.license_tier = tier.to_string();
        self.daily_conversations_used = used;
        self.daily_conversations_limit = limit;
        self.is_trial_active = is_trial;
        self.trial_days_remaining = trial_days;
    }
}

/// Project information
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub root_path: PathBuf, // Add root_path for repository context
    pub language: Option<String>,
    pub git_status: GitStatus,
    pub git_branch: Option<String>,
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

        // Extract git branch if available
        let git_branch = match &git_status {
            GitStatus::Repository { branch, .. } => Some(branch.clone()),
            _ => None,
        };

        Ok(Self {
            name,
            path: path.clone(),
            root_path: path, // Use the same path as root_path for now
            language,
            git_status,
            git_branch,
            file_count,
        })
    }
}

/// File explorer state
#[derive(Clone, Debug, PartialEq)]
pub struct FileExplorerState {
    pub root_path: Option<PathBuf>,
    pub expanded_dirs: HashMap<PathBuf, bool>,
    pub selected_file: Option<PathBuf>,
    pub selected_directory: Option<PathBuf>, // Track selected directory for repository context
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
            selected_directory: None,
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
    
    pub fn select_directory(&mut self, path: PathBuf) {
        self.selected_directory = Some(path);
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
    pub depth: usize,  // NEW: Track nesting depth for proper indentation
}

impl PartialEq for FileItem {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.name == other.name
            && self.is_directory == other.is_directory
            && self.is_expanded == other.is_expanded
            && self.children == other.children
            && self.file_type == other.file_type
            && self.git_status == other.git_status
            && self.size == other.size
            && self.depth == other.depth
            && match (&self.modified, &other.modified) {
                (Some(a), Some(b)) => a.timestamp() == b.timestamp(),
                (None, None) => true,
                _ => false,
            }
    }
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
    XML,
    Markdown,
    Text,
    Shell,
    Docker,
    Image,
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
            "xml" => Self::XML,
            "md" | "markdown" => Self::Markdown,
            "txt" => Self::Text,
            "sh" | "bash" => Self::Shell,
            "dockerfile" => Self::Docker,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => Self::Image,
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
            Self::XML => "📄",
            Self::Markdown => "📝",
            Self::Text => "📄",
            Self::Shell => "💻",
            Self::Docker => "🐳",
            Self::Image => "🖼️",
            Self::Binary => "📦",
            Self::Directory => "📁",
            Self::Unknown => "❓",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::TypeScript => "ts",
            Self::JavaScript => "js",
            Self::Python => "py",
            Self::Go => "go",
            Self::Java => "java",
            Self::CPP => "cpp",
            Self::C => "c",
            Self::HTML => "html",
            Self::CSS => "css",
            Self::JSON => "json",
            Self::TOML => "toml",
            Self::YAML => "yaml",
            Self::XML => "xml",
            Self::Markdown => "md",
            Self::Text => "txt",
            Self::Shell => "sh",
            Self::Docker => "dockerfile",
            Self::Image => "img",
            Self::Binary => "bin",
            Self::Directory => "",
            Self::Unknown => "",
        }
    }
}

/// Chat interface state
#[derive(Clone, Debug, PartialEq)]
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

impl PartialEq for ChatMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.content == other.content
            && self.message_type == other.message_type
            && self.timestamp.timestamp() == other.timestamp.timestamp()
            && self.metadata == other.metadata
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    System,
    Error,
    Welcome,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MessageMetadata {
    pub cost: Option<f64>,
    pub model: Option<String>,
    pub processing_time: Option<u64>,
    pub token_count: Option<u32>,
}

/// Consensus engine state
#[derive(Clone, Debug, PartialEq)]
pub struct ConsensusState {
    pub is_active: bool,
    pub is_running: bool,
    pub current_stage: Option<ConsensusStage>,
    pub progress: ConsensusProgress,
    pub stages: Vec<StageInfo>,
    pub total_tokens: usize,
    pub estimated_cost: f64,
    pub streaming_content: String,
    pub raw_streaming_content: String,  // Raw markdown before HTML conversion
    pub active_profile_name: String,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            is_active: false,
            is_running: false,
            current_stage: None,
            progress: ConsensusProgress::default(),
            stages: vec![
                StageInfo::new("Generator", "Not loaded"),
                StageInfo::new("Refiner", "Not loaded"),
                StageInfo::new("Validator", "Not loaded"),
                StageInfo::new("Curator", "Not loaded"),
            ],
            total_tokens: 0,
            estimated_cost: 0.0,
            streaming_content: String::new(),
            raw_streaming_content: String::new(),
            active_profile_name: "No profile".to_string(),
        }
    }

    pub fn start_consensus(&mut self) {
        self.is_active = true;
        self.is_running = true;
        self.current_stage = Some(ConsensusStage::Generator);
        self.progress = ConsensusProgress::default();
        self.total_tokens = 0;
        self.estimated_cost = 0.0;
        self.streaming_content.clear();
        self.raw_streaming_content.clear();
        tracing::info!("🚀 ConsensusState::start_consensus() - is_running set to true");
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
        self.is_running = false;
        self.current_stage = None;
        tracing::info!("🛑 ConsensusState::complete_consensus() - is_running set to false");
    }

    pub fn add_tokens(&mut self, tokens: usize, cost: f64) {
        self.total_tokens += tokens;
        self.estimated_cost += cost;
    }
}

// Use ConsensusStage from consensus module
pub use crate::consensus::ConsensusStage;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConsensusProgress {
    pub generator: u8,
    pub refiner: u8,
    pub validator: u8,
    pub curator: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StageInfo {
    pub name: String,
    pub model: String,
    pub status: StageStatus,
    pub progress: u8,
    pub error_message: Option<String>,
}

impl StageInfo {
    pub fn new(name: &str, model: &str) -> Self {
        Self {
            name: name.to_string(),
            model: model.to_string(),
            status: StageStatus::Waiting,
            progress: 0,
            error_message: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StageStatus {
    Waiting,
    Running,
    Completed,
    Error,
}

/// Application settings
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub enum GitStatus {
    NotRepository,
    Repository { branch: String, has_changes: bool },
}

#[derive(Clone, Debug, PartialEq)]
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
    root: &PathBuf,
    expanded: &HashMap<PathBuf, bool>,
    show_hidden: bool,
) -> anyhow::Result<Vec<FileItem>> {
    crate::desktop::file_system::load_directory_tree(root, expanded, show_hidden).await
}
