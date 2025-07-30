//! Repository Selector Component
//!
//! Provides a dropdown/list UI for displaying and switching between repositories.
//! Follows the patterns established in explorer.rs and status_bar.rs.

use crate::desktop::events::{Event, EventPayload, EventType};
use crate::desktop::workspace::WorkspaceState;
use crate::tui::themes::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{debug, error};

/// Repository selector component state
pub struct RepositorySelector {
    /// List of available repositories
    repositories: Vec<RepositoryEntry>,
    /// List state for navigation
    list_state: ListState,
    /// Whether the selector is currently open/active
    is_open: bool,
    /// Selected repository path
    current_repository: Option<PathBuf>,
    /// Whether the component has focus
    is_focused: bool,
    /// Filter text for repository search
    filter_text: String,
    /// Filtered repositories (indices into repositories vec)
    filtered_indices: Vec<usize>,
}

/// Repository entry with metadata
#[derive(Debug, Clone)]
pub struct RepositoryEntry {
    /// Repository name (usually directory name)
    pub name: String,
    /// Full path to repository
    pub path: PathBuf,
    /// Git branch name if available
    pub current_branch: Option<String>,
    /// Git status summary
    pub git_status: RepositoryGitStatus,
    /// Whether this is the current/active repository
    pub is_current: bool,
    /// Remote URL if available
    pub remote_url: Option<String>,
}

/// Git status summary for a repository
#[derive(Debug, Clone, PartialEq)]
pub enum RepositoryGitStatus {
    /// Repository is clean (no changes)
    Clean,
    /// Repository has modified files
    Modified { count: usize },
    /// Repository has staged changes
    Staged { count: usize },
    /// Repository has both staged and modified files
    Mixed { staged: usize, modified: usize },
    /// Repository has untracked files
    Untracked { count: usize },
    /// Repository is ahead of remote
    Ahead { commits: usize },
    /// Repository is behind remote
    Behind { commits: usize },
    /// Repository is diverged from remote
    Diverged { ahead: usize, behind: usize },
    /// Git repository but status unknown
    Unknown,
    /// Not a git repository
    NotGit,
}

impl RepositorySelector {
    /// Create new repository selector
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
            list_state: ListState::default(),
            is_open: false,
            current_repository: None,
            is_focused: false,
            filter_text: String::new(),
            filtered_indices: Vec::new(),
        }
    }

    /// Update repositories from workspace state
    pub fn update_from_workspace(&mut self, workspace: &WorkspaceState) {
        debug!("Updating repository selector from workspace");
        
        self.repositories.clear();
        self.filtered_indices.clear();
        
        // Convert workspace repositories to repository entries
        for (idx, repo_info) in workspace.repositories.iter().enumerate() {
            let is_current = workspace.active_repository.as_ref() == Some(&repo_info.path);
            
            // TODO: In the future, we could enhance RepositoryInfo to include branch and remote info
            // For now, we'll use placeholder values and try to get the info on demand
            let git_status = if repo_info.has_changes {
                RepositoryGitStatus::Modified { count: 1 }
            } else {
                RepositoryGitStatus::Clean
            };
            
            let entry = RepositoryEntry {
                name: repo_info.name.clone(),
                path: repo_info.path.clone(),
                current_branch: None, // Could be populated by querying git directly
                git_status,
                is_current,
                remote_url: None, // Could be populated by querying git directly
            };
            
            self.repositories.push(entry);
            self.filtered_indices.push(idx);
        }

        // Update current repository
        self.current_repository = workspace.active_repository.clone();
        
        // Reset list state
        if !self.repositories.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
        
        debug!("Updated repository selector with {} repositories", self.repositories.len());
    }

    /// Toggle the selector open/closed state
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            self.is_focused = true;
            // Select current repository in list
            if let Some(current_path) = &self.current_repository {
                if let Some(index) = self.repositories.iter().position(|r| &r.path == current_path) {
                    self.list_state.select(Some(index));
                }
            }
        } else {
            self.is_focused = false;
            self.clear_filter();
        }
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Check if selector is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Check if selector has focus
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Get current repository path
    pub fn current_repository(&self) -> Option<&Path> {
        self.current_repository.as_deref()
    }

    /// Set current repository
    pub fn set_current_repository(&mut self, path: Option<PathBuf>) {
        self.current_repository = path;
        // Update is_current flags
        for repo in &mut self.repositories {
            repo.is_current = self.current_repository.as_ref() == Some(&repo.path);
        }
    }

    /// Apply filter to repositories
    pub fn set_filter(&mut self, filter: String) {
        self.filter_text = filter;
        self.apply_filter();
    }

    /// Clear filter
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.apply_filter();
    }

    /// Apply current filter to repository list
    fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            // Show all repositories
            self.filtered_indices = (0..self.repositories.len()).collect();
        } else {
            // Filter repositories by name or path
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_indices = self.repositories
                .iter()
                .enumerate()
                .filter(|(_, repo)| {
                    repo.name.to_lowercase().contains(&filter_lower) ||
                    repo.path.to_string_lossy().to_lowercase().contains(&filter_lower)
                })
                .map(|(idx, _)| idx)
                .collect();
        }

        // Reset selection to first filtered item
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    /// Get currently selected repository
    pub fn selected_repository(&self) -> Option<&RepositoryEntry> {
        self.list_state.selected()
            .and_then(|idx| self.filtered_indices.get(idx))
            .and_then(|&repo_idx| self.repositories.get(repo_idx))
    }

    /// Get the number of visible repositories
    pub fn visible_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Render the repository selector
    pub fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.is_open {
            // Render compact view (just current repository name)
            self.render_compact(frame, area, theme);
        } else {
            // Render expanded dropdown
            self.render_dropdown(frame, area, theme);
        }
    }

    /// Render compact view showing just the current repository
    fn render_compact(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let current_name = self.current_repository
            .as_ref()
            .and_then(|path| self.repositories.iter().find(|r| &r.path == path))
            .map(|r| r.name.as_str())
            .unwrap_or("No Repository");

        let current_branch = self.current_repository
            .as_ref()
            .and_then(|path| self.repositories.iter().find(|r| &r.path == path))
            .and_then(|r| r.current_branch.as_ref())
            .map(|b| format!(" ({})", b))
            .unwrap_or_default();

        let content = format!("📁 {} {}", current_name, current_branch);
        
        let paragraph = Paragraph::new(content)
            .style(if self.is_focused {
                theme.active_border_style()
            } else {
                theme.text_style()
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Repository")
                    .border_style(if self.is_focused {
                        theme.active_border_style()
                    } else {
                        theme.inactive_border_style()
                    }),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Render expanded dropdown view
    fn render_dropdown(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Calculate dropdown size and position
        let dropdown_height = (self.filtered_indices.len() + 3).min(15) as u16; // +3 for border and filter
        let dropdown_width = area.width.min(60);
        
        let dropdown_area = Rect {
            x: area.x,
            y: area.y,
            width: dropdown_width,
            height: dropdown_height,
        };

        // Clear the area first
        frame.render_widget(Clear, dropdown_area);

        // Split area for filter and list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Filter input
                Constraint::Min(0),    // Repository list
            ])
            .split(dropdown_area);

        // Render filter input
        self.render_filter_input(frame, chunks[0], theme);

        // Render repository list
        self.render_repository_list(frame, chunks[1], theme);
    }

    /// Render filter input field
    fn render_filter_input(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let filter_text = if self.filter_text.is_empty() {
            "Type to filter repositories...".to_string()
        } else {
            self.filter_text.clone()
        };

        let paragraph = Paragraph::new(filter_text)
            .style(if self.filter_text.is_empty() {
                theme.muted_color().into()
            } else {
                theme.text_style()
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Filter")
                    .border_style(theme.active_border_style()),
            );

        frame.render_widget(paragraph, area);
    }

    /// Render repository list
    fn render_repository_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = self.filtered_indices
            .iter()
            .map(|&idx| self.repositories.get(idx))
            .filter_map(|repo| repo.map(|r| create_repository_list_item(r, theme)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Repositories ({})", self.filtered_indices.len()))
                    .border_style(theme.active_border_style()),
            )
            .style(theme.panel_style())
            .highlight_style(
                Style::default()
                    .bg(theme.selection_bg_color())
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    /// Handle key events
    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Event>> {
        if !self.is_open {
            // When closed, only handle toggle key
            match key.code {
                KeyCode::F(5) | KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.toggle();
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        match key.code {
            KeyCode::Esc => {
                self.toggle(); // Close dropdown
                Ok(None)
            }
            KeyCode::Enter => {
                if let Some(repo) = self.selected_repository() {
                    let event = Event::repository_changed(
                        repo.path.clone(),
                        repo.name.clone(),
                        repo.remote_url.clone(),
                    );
                    self.toggle(); // Close dropdown
                    Ok(Some(event))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                } else if !self.filtered_indices.is_empty() {
                    self.list_state.select(Some(0));
                }
                Ok(None)
            }
            KeyCode::Down => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.filtered_indices.len().saturating_sub(1) {
                        self.list_state.select(Some(selected + 1));
                    }
                } else if !self.filtered_indices.is_empty() {
                    self.list_state.select(Some(0));
                }
                Ok(None)
            }
            KeyCode::Char(c) => {
                // Add to filter
                self.filter_text.push(c);
                self.apply_filter();
                Ok(None)
            }
            KeyCode::Backspace => {
                // Remove from filter
                if !self.filter_text.is_empty() {
                    self.filter_text.pop();
                    self.apply_filter();
                }
                Ok(None)
            }
            KeyCode::Tab => {
                // Cycle through repositories
                if !self.filtered_indices.is_empty() {
                    let current = self.list_state.selected().unwrap_or(0);
                    let next = (current + 1) % self.filtered_indices.len();
                    self.list_state.select(Some(next));
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

/// Create a list item for a repository entry
fn create_repository_list_item<'a>(repo: &'a RepositoryEntry, theme: &'a Theme) -> ListItem<'a> {
    let mut spans = Vec::new();

    // Current repository indicator
    if repo.is_current {
        spans.push(Span::styled("● ", Style::default().fg(theme.success_color())));
    } else {
        spans.push(Span::raw("  "));
    }

    // Repository name
    spans.push(Span::styled(
        &repo.name,
        Style::default()
            .fg(theme.primary_color())
            .add_modifier(Modifier::BOLD),
    ));

    // Branch name
    if let Some(branch) = &repo.current_branch {
        spans.push(Span::styled(
            format!(" ({})", branch),
            Style::default().fg(theme.muted_color()),
        ));
    }

    // Git status indicator
    let status_span = get_git_status_span(&repo.git_status, theme);
    if let Some(span) = status_span {
        spans.push(Span::raw(" "));
        spans.push(span);
    }

    ListItem::new(Line::from(spans))
}

/// Get git status span for display
fn get_git_status_span<'a>(status: &'a RepositoryGitStatus, theme: &'a Theme) -> Option<Span<'a>> {
    match status {
        RepositoryGitStatus::Clean => None,
        RepositoryGitStatus::Modified { count } => Some(Span::styled(
            format!("M{}", count),
            Style::default().fg(theme.warning_color()),
        )),
        RepositoryGitStatus::Staged { count } => Some(Span::styled(
            format!("S{}", count),
            Style::default().fg(theme.success_color()),
        )),
        RepositoryGitStatus::Mixed { staged, modified } => Some(Span::styled(
            format!("S{}M{}", staged, modified),
            Style::default().fg(theme.accent_color()),
        )),
        RepositoryGitStatus::Untracked { count } => Some(Span::styled(
            format!("?{}", count),
            Style::default().fg(theme.info_color()),
        )),
        RepositoryGitStatus::Ahead { commits } => Some(Span::styled(
            format!("↑{}", commits),
            Style::default().fg(theme.success_color()),
        )),
        RepositoryGitStatus::Behind { commits } => Some(Span::styled(
            format!("↓{}", commits),
            Style::default().fg(theme.warning_color()),
        )),
        RepositoryGitStatus::Diverged { ahead, behind } => Some(Span::styled(
            format!("↑{}↓{}", ahead, behind),
            Style::default().fg(theme.error_color()),
        )),
        RepositoryGitStatus::Unknown => Some(Span::styled(
            "?",
            Style::default().fg(theme.muted_color()),
        )),
        RepositoryGitStatus::NotGit => None,
    }
}


impl Default for RepositorySelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_repository_selector_creation() {
        let selector = RepositorySelector::new();
        assert!(!selector.is_open());
        assert!(!selector.is_focused());
        assert!(selector.repositories.is_empty());
    }

    #[test]
    fn test_repository_selector_toggle() {
        let mut selector = RepositorySelector::new();
        
        assert!(!selector.is_open());
        selector.toggle();
        assert!(selector.is_open());
        assert!(selector.is_focused());
        
        selector.toggle();
        assert!(!selector.is_open());
        assert!(!selector.is_focused());
    }

    #[test]
    fn test_filter_functionality() {
        let mut selector = RepositorySelector::new();
        
        // Add test repositories
        selector.repositories = vec![
            RepositoryEntry {
                name: "project-a".to_string(),
                path: PathBuf::from("/path/to/project-a"),
                current_branch: Some("main".to_string()),
                git_status: RepositoryGitStatus::Clean,
                is_current: false,
                remote_url: None,
            },
            RepositoryEntry {
                name: "project-b".to_string(),
                path: PathBuf::from("/path/to/project-b"),
                current_branch: Some("develop".to_string()),
                git_status: RepositoryGitStatus::Modified { count: 2 },
                is_current: true,
                remote_url: None,
            },
        ];
        
        // Initial state - all repositories visible
        selector.apply_filter();
        assert_eq!(selector.filtered_indices.len(), 2);
        
        // Filter by name
        selector.set_filter("project-a".to_string());
        assert_eq!(selector.filtered_indices.len(), 1);
        assert_eq!(selector.filtered_indices[0], 0);
        
        // Clear filter
        selector.clear_filter();
        assert_eq!(selector.filtered_indices.len(), 2);
    }

    #[test]
    fn test_current_repository_tracking() {
        let mut selector = RepositorySelector::new();
        
        let repo_path = PathBuf::from("/test/repo");
        selector.set_current_repository(Some(repo_path.clone()));
        
        assert_eq!(selector.current_repository(), Some(repo_path.as_path()));
    }
}