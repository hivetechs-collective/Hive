//! UI Events for AI Helper Navigation
//! 
//! This module provides events that allow the AI Helper to control
//! the UI, such as navigating to files or creating new files.

use std::path::PathBuf;
use dioxus::prelude::*;

/// Events that the AI Helper can send to control the UI
#[derive(Debug, Clone, PartialEq)]
pub enum AIHelperUIEvent {
    /// Navigate to a specific file or directory in the File Explorer
    NavigateToPath {
        path: PathBuf,
        reason: String,
    },
    
    /// Open a file in the editor
    OpenFile {
        path: PathBuf,
    },
    
    /// Create a new file and optionally open it
    CreateFile {
        path: PathBuf,
        content: String,
        open_after_create: bool,
    },
    
    /// Refresh the File Explorer after file creation
    RefreshFileExplorer,
    
    /// Show a message to the user
    ShowMessage {
        message: String,
        message_type: MessageType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

/// Global signal for AI Helper UI events
pub static AI_HELPER_EVENTS: GlobalSignal<Option<AIHelperUIEvent>> = Signal::global(|| None);

/// Send an AI Helper UI event
pub fn send_ai_helper_event(event: AIHelperUIEvent) {
    tracing::info!("🤖 AI Helper sending UI event: {:?}", event);
    AI_HELPER_EVENTS.write().replace(event);
}

/// Process AI Helper UI events in the main UI
pub fn process_ai_helper_events(
    mut app_state: Signal<crate::desktop::state::AppState>,
) {
    if let Some(event) = AI_HELPER_EVENTS.read().as_ref() {
        match event {
            AIHelperUIEvent::NavigateToPath { path, reason } => {
                tracing::info!("🧭 AI Helper navigating to: {} ({})", path.display(), reason);
                
                // If it's a directory, update the selected directory and current project
                if path.is_dir() {
                    app_state.write().file_explorer.selected_directory = Some(path.clone());
                    
                    // Update current project asynchronously
                    let path_clone = path.clone();
                    let mut app_state_clone = app_state.clone();
                    spawn(async move {
                        if let Ok(project_info) = crate::desktop::state::ProjectInfo::from_path(path_clone).await {
                            app_state_clone.write().current_project = Some(project_info);
                        }
                    });
                } else if path.is_file() {
                    // It's a file - select it
                    app_state.write().file_explorer.selected_file = Some(path.clone());
                    
                    // Also update the selected directory to the file's parent
                    if let Some(parent) = path.parent() {
                        app_state.write().file_explorer.selected_directory = Some(parent.to_path_buf());
                    }
                }
                
                // TODO: Scroll the File Explorer to show the selected item
            }
            
            AIHelperUIEvent::OpenFile { path } => {
                if path.exists() && path.is_file() {
                    tracing::info!("📂 AI Helper opening file: {}", path.display());
                    app_state.write().file_explorer.selected_file = Some(path.clone());
                    // TODO: Actually open the file in the editor
                }
            }
            
            AIHelperUIEvent::CreateFile { path, content: _, open_after_create } => {
                tracing::info!("📝 AI Helper creating file: {}", path.display());
                
                // The file has already been created by the AI Helper
                // We just need to refresh the UI and optionally open it
                
                if *open_after_create && path.exists() {
                    app_state.write().file_explorer.selected_file = Some(path.clone());
                    // TODO: Open in editor
                }
                
                // Trigger File Explorer refresh
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    if let Err(e) = app_state_clone.write().file_explorer.refresh().await {
                        tracing::error!("Failed to refresh file explorer: {}", e);
                    }
                });
            }
            
            AIHelperUIEvent::RefreshFileExplorer => {
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    if let Err(e) = app_state_clone.write().file_explorer.refresh().await {
                        tracing::error!("Failed to refresh file explorer: {}", e);
                    }
                });
            }
            
            AIHelperUIEvent::ShowMessage { message, message_type } => {
                // TODO: Show message in UI
                match message_type {
                    MessageType::Info => tracing::info!("ℹ️ {}", message),
                    MessageType::Success => tracing::info!("✅ {}", message),
                    MessageType::Warning => tracing::warn!("⚠️ {}", message),
                    MessageType::Error => tracing::error!("❌ {}", message),
                }
            }
        }
        
        // Clear the event after processing
        AI_HELPER_EVENTS.write().take();
    }
}