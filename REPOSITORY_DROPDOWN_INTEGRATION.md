# Repository Dropdown Component Integration Guide

## Overview

The enhanced repository dropdown menu component (`RepositoryDropdown`) provides a VS Code-style interface for selecting and managing repositories in multi-repository workspaces. This component integrates seamlessly with the status bar and workspace state management.

## Features

### 1. **Rich Repository Display**
- Repository status indicators (clean, modified, conflicts)
- Project type icons (Rust 🦀, JavaScript 📜, TypeScript 💙, Python 🐍, etc.)
- Git branch information
- Upstream sync status (ahead/behind commits)
- Extended metadata display (version, file count, last activity)

### 2. **Advanced Search & Filtering**
- Real-time search by repository name or path
- Filter chips for repository status:
  - ✓ Clean repositories
  - ● Modified repositories
  - ⚠ Repositories with conflicts
- Project type filtering (optional)

### 3. **VS Code-Style Design**
- Consistent with VS Code's dropdown styling
- Smooth animations and transitions
- Keyboard navigation support
- Responsive layout with scrolling for long lists

### 4. **Integration Points**
- Works with `RepositorySelectorState` for state management
- Accepts optional `RepositoryMetadata` for enhanced information
- Integrates with the enhanced status bar
- Event-driven architecture for repository changes

## Usage

### Basic Integration

```rust
use hive::desktop::git::{
    RepositoryDropdown, RepositoryDropdownProps,
    RepositorySelectorState, REPOSITORY_DROPDOWN_STYLES
};

#[component]
fn MyApp() -> Element {
    // Repository selector state
    let mut selector_state = use_signal(RepositorySelectorState::default);
    
    // Dropdown visibility
    let mut dropdown_visible = use_signal(|| false);
    
    rsx! {
        // Add styles
        style { {REPOSITORY_DROPDOWN_STYLES} }
        
        // Repository dropdown
        RepositoryDropdown {
            selector_state: selector_state.clone(),
            repository_metadata: None, // Optional metadata
            visible: dropdown_visible.clone(),
            position: (100, 30), // x, y position
            on_repository_selected: move |path: PathBuf| {
                // Handle repository selection
                selector_state.write().set_current_repository(&path);
                dropdown_visible.set(false);
            },
            on_refresh_requested: move |_| {
                // Handle refresh
            },
            on_manage_repositories: move |_| {
                // Open repository management
            }
        }
    }
}
```

### Integration with Status Bar

```rust
use hive::desktop::status_bar_enhanced::{EnhancedStatusBar, StatusBarState};

// In your component
EnhancedStatusBar {
    state: status_bar_state.clone(),
    on_item_click: move |item_id: String| {
        // Handle status bar clicks
    },
    on_git_branch_click: move |_| {
        // Toggle repository dropdown
        dropdown_visible.set(!dropdown_visible.read());
    }
}
```

### With Repository Metadata

```rust
use hive::desktop::workspace::repository_discovery::RepositoryMetadata;
use std::collections::HashMap;

// Create metadata map
let repository_metadata = HashMap::<PathBuf, RepositoryMetadata>::new();

// Pass to dropdown
RepositoryDropdown {
    selector_state: selector_state.clone(),
    repository_metadata: Some(repository_metadata),
    // ... other props
}
```

## Component Props

### RepositoryDropdownProps

| Prop | Type | Description |
|------|------|-------------|
| `selector_state` | `Signal<RepositorySelectorState>` | Repository selector state |
| `repository_metadata` | `Option<HashMap<PathBuf, RepositoryMetadata>>` | Extended metadata for repositories |
| `visible` | `Signal<bool>` | Controls dropdown visibility |
| `position` | `(i32, i32)` | Position (x, y) for the dropdown |
| `on_repository_selected` | `EventHandler<PathBuf>` | Called when a repository is selected |
| `on_refresh_requested` | `Option<EventHandler<()>>` | Called when refresh is requested |
| `on_manage_repositories` | `Option<EventHandler<()>>` | Called to open repository management |

## Styling

The component uses VS Code's design system variables for consistent theming:

```css
--vscode-dropdown-background
--vscode-dropdown-foreground
--vscode-dropdown-border
--vscode-input-background
--vscode-input-foreground
--vscode-list-activeSelectionBackground
--vscode-list-hoverBackground
```

Include the `REPOSITORY_DROPDOWN_STYLES` constant in your application to get all required styles.

## Example Application

See `examples/repository_dropdown_example.rs` for a complete working example that demonstrates:
- Setting up the repository selector state
- Adding repository metadata
- Integrating with the status bar
- Handling repository selection events

## Testing

Run the integration tests:

```bash
cargo test repository_dropdown_test
```

## Future Enhancements

1. **Keyboard Navigation**
   - Arrow keys for selection
   - Enter to select
   - Escape to close
   - Type-ahead search

2. **Repository Groups**
   - Group by organization
   - Group by project type
   - Custom grouping

3. **Quick Actions**
   - Clone repository
   - Open in terminal
   - Open in file manager
   - Git operations menu

4. **Performance**
   - Virtual scrolling for large lists
   - Lazy loading of metadata
   - Caching of repository information