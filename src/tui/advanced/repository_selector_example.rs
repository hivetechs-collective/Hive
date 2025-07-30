//! Repository Selector Usage Example
//!
//! This example demonstrates how to integrate the RepositorySelector component
//! with the AdvancedTuiApp and handle repository switching events.

use crate::desktop::events::{Event, EventPayload, EventType};
use crate::desktop::workspace::WorkspaceState;
use crate::tui::advanced::{AdvancedTuiApp, PanelType};
use crate::tui::advanced::repository_selector::{RepositorySelector, RepositoryEntry, RepositoryGitStatus};
use crate::tui::themes::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::Backend, layout::Rect, Frame};
use std::path::PathBuf;

/// Example integration of RepositorySelector with AdvancedTuiApp
pub struct ExampleTuiWithRepositorySelector {
    /// Base advanced TUI app
    pub advanced_app: AdvancedTuiApp,
    /// Repository selector component
    pub repo_selector: RepositorySelector,
    /// Current workspace state
    pub workspace: WorkspaceState,
}

impl ExampleTuiWithRepositorySelector {
    /// Create new example TUI app with repository selector
    pub async fn new(workspace_path: PathBuf) -> Result<Self> {
        let advanced_app = AdvancedTuiApp::new().await?;
        let mut repo_selector = RepositorySelector::new();
        
        // Initialize workspace state
        let mut workspace = WorkspaceState::new(workspace_path);
        workspace.scan_for_repositories()?;
        
        // Update repository selector with discovered repositories
        repo_selector.update_from_workspace(&workspace);
        
        Ok(Self {
            advanced_app,
            repo_selector,
            workspace,
        })
    }

    /// Render the TUI with repository selector
    pub fn render(&mut self, frame: &mut Frame) {
        let size = frame.size();
        
        // Get theme from advanced app
        let theme = &self.advanced_app.theme;
        
        // Create layout: repository selector at top, rest below
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3), // Repository selector
                ratatui::layout::Constraint::Min(0),    // Advanced TUI content
            ])
            .split(size);

        // Render repository selector
        self.repo_selector.render(frame, chunks[0], theme);
        
        // Render advanced TUI in remaining space
        self.render_advanced_tui_content(frame, chunks[1]);
    }

    /// Render the advanced TUI content in the given area
    fn render_advanced_tui_content(&mut self, frame: &mut Frame, area: Rect) {
        // This would render the standard AdvancedTuiApp content
        // For this example, we'll use a simplified version
        
        let theme = &self.advanced_app.theme;
        
        // Create main layout for advanced TUI panels
        let main_chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(25), // Explorer
                ratatui::layout::Constraint::Percentage(75), // Editor/Terminal
            ])
            .split(area);

        // Render explorer panel
        self.advanced_app.explorer.render(
            frame,
            main_chunks[0],
            theme,
            true, // is_active - for example purposes
        );

        // Split right side for editor and terminal
        let right_chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(70), // Editor
                ratatui::layout::Constraint::Percentage(30), // Terminal
            ])
            .split(main_chunks[1]);

        // Render editor panel
        self.advanced_app.editor.render(
            frame,
            right_chunks[0],
            theme,
            false, // is_active
        );

        // Render terminal panel
        self.advanced_app.terminal.render(
            frame,
            right_chunks[1],
            theme,
            false, // is_active
        );
    }

    /// Handle key events for the entire application
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        // First, let repository selector handle the event
        if let Ok(Some(event)) = self.repo_selector.handle_key_event(key).await {
            // Handle repository switch event
            self.handle_repository_event(event).await?;
            return Ok(false);
        }

        // If repository selector is open and focused, don't pass events to other components
        if self.repo_selector.is_open() && self.repo_selector.is_focused() {
            return Ok(false);
        }

        // Check for repository selector toggle shortcut
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                self.repo_selector.toggle();
                return Ok(false);
            }
            _ => {}
        }

        // Pass event to advanced TUI app
        self.advanced_app.handle_key_event(key).await
    }

    /// Handle repository-related events
    async fn handle_repository_event(&mut self, event: Event) -> Result<()> {
        match event.event_type {
            EventType::RepositoryChanged => {
                if let EventPayload::RepositoryInfo { path, name, remote_url } = event.payload {
                    println!("Switching to repository: {} at {:?}", name, path);
                    
                    // Update workspace active repository
                    self.workspace.switch_repository(path.clone())?;
                    
                    // Update repository selector current repository
                    self.repo_selector.set_current_repository(Some(path.clone()));
                    
                    // Update explorer to show new repository
                    self.advanced_app.explorer.set_root(path).await?;
                    
                    // Optionally save workspace state
                    self.workspace.save_state()?;
                    
                    println!("Repository switched successfully!");
                    if let Some(url) = remote_url {
                        println!("Remote URL: {}", url);
                    }
                }
            }
            _ => {
                // Handle other events as needed
            }
        }
        Ok(())
    }

    /// Refresh repository list (e.g., when workspace changes)
    pub async fn refresh_repositories(&mut self) -> Result<()> {
        // Re-scan workspace for repositories
        self.workspace.scan_for_repositories()?;
        
        // Update repository selector
        self.repo_selector.update_from_workspace(&self.workspace);
        
        println!("Refreshed repository list: {} repositories found", 
                 self.workspace.repositories.len());
        
        Ok(())
    }

    /// Get current repository information
    pub fn current_repository(&self) -> Option<&RepositoryEntry> {
        self.repo_selector.selected_repository()
    }

    /// Check if should quit
    pub fn should_quit(&self) -> bool {
        self.advanced_app.should_quit()
    }
}

/// Example of creating repository entries manually (for testing)
pub fn create_example_repositories() -> Vec<RepositoryEntry> {
    vec![
        RepositoryEntry {
            name: "hive-ai".to_string(),
            path: PathBuf::from("/Users/dev/hive-ai"),
            current_branch: Some("main".to_string()),
            git_status: RepositoryGitStatus::Clean,
            is_current: true,
            remote_url: Some("https://github.com/user/hive-ai.git".to_string()),
        },
        RepositoryEntry {
            name: "project-alpha".to_string(),
            path: PathBuf::from("/Users/dev/project-alpha"),
            current_branch: Some("feature/new-ui".to_string()),
            git_status: RepositoryGitStatus::Modified { count: 3 },
            is_current: false,
            remote_url: Some("https://github.com/user/project-alpha.git".to_string()),
        },
        RepositoryEntry {
            name: "experiment-beta".to_string(),
            path: PathBuf::from("/Users/dev/experiment-beta"),
            current_branch: Some("develop".to_string()),
            git_status: RepositoryGitStatus::Ahead { commits: 2 },
            is_current: false,
            remote_url: None,
        },
    ]
}

/// Example of handling repository selector in a simple event loop
pub async fn example_event_loop() -> Result<()> {
    use crossterm::{
        event::{self, Event as CrosstermEvent},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;
    use std::time::Duration;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create example app
    let workspace_path = std::env::current_dir()?;
    let mut app = ExampleTuiWithRepositorySelector::new(workspace_path).await?;

    // Main event loop
    loop {
        // Render
        terminal.draw(|f| app.render(f))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let CrosstermEvent::Key(key) = event::read()? {
                if app.handle_key_event(key).await? {
                    break; // Quit
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_example_app_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();
        
        let app = ExampleTuiWithRepositorySelector::new(workspace_path).await;
        assert!(app.is_ok());
        
        let app = app.unwrap();
        assert_eq!(app.workspace.repositories.len(), 0); // No git repos in temp dir
    }

    #[test]
    fn test_example_repositories() {
        let repos = create_example_repositories();
        assert_eq!(repos.len(), 3);
        
        // Check that one is marked as current
        let current_count = repos.iter().filter(|r| r.is_current).count();
        assert_eq!(current_count, 1);
        
        // Check various git statuses
        assert!(repos.iter().any(|r| matches!(r.git_status, RepositoryGitStatus::Clean)));
        assert!(repos.iter().any(|r| matches!(r.git_status, RepositoryGitStatus::Modified { .. })));
        assert!(repos.iter().any(|r| matches!(r.git_status, RepositoryGitStatus::Ahead { .. })));
    }

    #[tokio::test]
    async fn test_repository_event_handling() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();
        
        let mut app = ExampleTuiWithRepositorySelector::new(workspace_path).await.unwrap();
        
        // Create a test repository event
        let event = Event::repository_changed(
            PathBuf::from("/test/repo"),
            "test-repo".to_string(),
            Some("https://github.com/test/repo.git".to_string()),
        );
        
        // Handle the event
        let result = app.handle_repository_event(event).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_repository_selector_integration() {
        let mut selector = RepositorySelector::new();
        
        // Test initial state
        assert!(!selector.is_open());
        assert!(!selector.is_focused());
        assert_eq!(selector.visible_count(), 0);
        
        // Test toggle
        selector.toggle();
        assert!(selector.is_open());
        assert!(selector.is_focused());
        
        // Test filter functionality
        selector.set_filter("test".to_string());
        selector.clear_filter();
        
        // Test manual repository setup
        let repos = create_example_repositories();
        // This would require extending RepositorySelector to accept manual repositories
        // For now, we're testing the basic functionality
        
        assert_eq!(selector.current_repository(), None);
    }
}

/// Usage documentation for the Repository Selector component
pub mod usage_docs {
    //! # Repository Selector Component Usage Guide
    //!
    //! The RepositorySelector component provides a VS Code-like repository switcher
    //! for navigating between multiple git repositories in a workspace.
    //!
    //! ## Basic Usage
    //!
    //! ```rust,no_run
    //! use crate::tui::advanced::repository_selector::RepositorySelector;
    //! use crate::desktop::workspace::WorkspaceState;
    //! use std::path::PathBuf;
    //!
    //! # async fn example() -> anyhow::Result<()> {
    //! // Create repository selector
    //! let mut repo_selector = RepositorySelector::new();
    //!
    //! // Set up workspace
    //! let mut workspace = WorkspaceState::new(PathBuf::from("/path/to/workspace"));
    //! workspace.scan_for_repositories()?;
    //!
    //! // Update selector with workspace repositories
    //! repo_selector.update_from_workspace(&workspace);
    //! # Ok(())
    //! # }
    //! ```
    //!
    //! ## Key Bindings
    //!
    //! When the repository selector is **closed**:
    //! - `Ctrl+R` or `F5`: Open repository selector
    //!
    //! When the repository selector is **open**:
    //! - `Escape`: Close repository selector
    //! - `Enter`: Switch to selected repository
    //! - `Up/Down`: Navigate repository list
    //! - `Tab`: Cycle through repositories
    //! - Type characters: Filter repositories by name/path
    //! - `Backspace`: Remove filter characters
    //!
    //! ## Visual Indicators
    //!
    //! - `●` indicates the currently active repository
    //! - Branch name is shown in parentheses when available
    //! - Git status indicators:
    //!   - `M#` - Modified files (# = count)
    //!   - `S#` - Staged files (# = count)
    //!   - `?#` - Untracked files (# = count)
    //!   - `↑#` - Commits ahead of remote (# = count)
    //!   - `↓#` - Commits behind remote (# = count)
    //!   - `↑#↓#` - Diverged from remote
    //!
    //! ## Integration with Event Bus
    //!
    //! The component emits `RepositoryChanged` events when a repository is selected:
    //!
    //! ```rust,no_run
    //! use crate::desktop::events::{Event, EventType};
    //!
    //! # async fn handle_events(mut repo_selector: crate::tui::advanced::repository_selector::RepositorySelector, key: crossterm::event::KeyEvent) -> anyhow::Result<()> {
    //! if let Ok(Some(event)) = repo_selector.handle_key_event(key).await {
    //!     match event.event_type {
    //!         EventType::RepositoryChanged => {
    //!             // Handle repository switch
    //!             println!("Repository changed!");
    //!         }
    //!         _ => {}
    //!     }
    //! }
    //! # Ok(())
    //! # }
    //! ```
    //!
    //! ## Styling and Themes
    //!
    //! The component follows the established theme system:
    //! - Uses `theme.active_border_style()` when focused
    //! - Uses `theme.inactive_border_style()` when not focused
    //! - Uses `theme.selection_bg_color()` for highlighting
    //! - Follows color conventions for git status indicators
    //!
    //! ## Advanced Features
    //!
    //! - **Filtering**: Type to filter repositories by name or path
    //! - **Compact Mode**: Shows current repository when closed
    //! - **Dropdown Mode**: Shows full list with filter when open
    //! - **Keyboard Navigation**: Full keyboard support for accessibility
    //! - **Git Integration**: Real-time git status indicators
    //! - **Responsive Design**: Adapts to different terminal sizes
}