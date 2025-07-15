# VS Code-like IDE Implementation Plan

## Current State Analysis

### What's Working
- Left sidebar file explorer shows files and folders
- Files can be clicked and content is loaded
- Basic tab UI exists but only shows one file

### What's Broken
1. **Single File Only**: Only one `selected_file` and `file_content` state
2. **No Tab Management**: Clicking a file replaces current file instead of opening new tab
3. **Analytics Not a Tab**: Analytics view is separate, not integrated as a tab
4. **No Editor**: Files display as read-only text, no editing capability
5. **No Save**: Can't save changes back to disk

## Target Architecture (VS Code-like)

### Core Components
```
┌─────────────────┬────────────────────────────┬─────────────────┐
│  File Explorer  │      Editor Tabs           │   Chat Panel    │
│  (Left Sidebar) │    (Center Pane)           │ (Right Sidebar) │
├─────────────────┼────────────────────────────┼─────────────────┤
│ 📁 Project      │ [file1.rs][file2.js][📊]   │ Consensus Chat  │
│  └ src/         │ ─────────────────────────  │                 │
│    └ main.rs    │ // Editor content here     │ Q: Explain...   │
│  └ tests/       │ fn main() {                │ A: This code... │
│                 │   println!("Hello");       │                 │
│                 │ }                          │                 │
└─────────────────┴────────────────────────────┴─────────────────┘
```

### State Management
```rust
// Replace single file state with multi-tab state
struct EditorState {
    tabs: Vec<EditorTab>,
    active_tab_id: Option<String>,
}

struct EditorTab {
    id: String,               // Unique ID
    path: Option<PathBuf>,    // None for special tabs like Analytics
    title: String,            // Display name
    content: String,          // File content
    tab_type: TabType,        // File, Analytics, Welcome
    is_dirty: bool,           // Has unsaved changes
    language: Option<String>, // For syntax highlighting
}

enum TabType {
    File,
    Analytics,
    Welcome,
}
```

## Implementation Steps

### Step 1: Create Tab Management System
```rust
// New signals to replace single file state
let mut editor_state = use_signal(|| EditorState::default());
let mut active_tab = use_signal(|| Option::<String>::None);

// Tab operations
fn open_file_in_tab(path: PathBuf, content: String) -> String {
    let tab_id = generate_tab_id();
    let tab = EditorTab {
        id: tab_id.clone(),
        path: Some(path.clone()),
        title: path.file_name().unwrap_or("Untitled").to_string(),
        content,
        tab_type: TabType::File,
        is_dirty: false,
        language: detect_language(&path),
    };
    
    editor_state.write().tabs.push(tab);
    active_tab.set(Some(tab_id.clone()));
    tab_id
}

fn switch_tab(tab_id: &str) {
    active_tab.set(Some(tab_id.to_string()));
}

fn close_tab(tab_id: &str) {
    // Handle unsaved changes
    if let Some(tab) = find_tab(tab_id) {
        if tab.is_dirty {
            // Show save dialog
        }
    }
    // Remove tab and activate next
}
```

### Step 2: Update File Click Handler
```rust
// In the file tree onclick handler
onclick: move |_| {
    if is_dir {
        // Toggle directory
    } else {
        // Open file in new tab (VS Code behavior)
        let path = file_path.clone();
        spawn(async move {
            match read_file_content(&path).await {
                Ok(content) => {
                    // Check if already open
                    if let Some(existing_tab) = find_tab_by_path(&path) {
                        switch_tab(&existing_tab.id);
                    } else {
                        open_file_in_tab(path, content);
                    }
                }
                Err(e) => show_error_notification(e),
            }
        });
    }
}
```

### Step 3: Render Tab Bar
```rust
// In the editor container
div {
    class: "editor-tabs",
    // Tab bar with all open tabs
    for tab in editor_state.read().tabs.iter() {
        div {
            class: if Some(&tab.id) == active_tab.read().as_ref() { 
                "editor-tab active" 
            } else { 
                "editor-tab" 
            },
            onclick: move |_| switch_tab(&tab.id),
            
            // Tab icon based on type
            span { class: "tab-icon",
                match tab.tab_type {
                    TabType::Analytics => "📊",
                    TabType::Welcome => "👋",
                    TabType::File => get_file_icon(&tab.path),
                }
            }
            
            // Tab title
            span { class: "tab-title", "{tab.title}" }
            
            // Dirty indicator
            if tab.is_dirty {
                span { class: "dirty-indicator", "●" }
            }
            
            // Close button (except for special tabs)
            if matches!(tab.tab_type, TabType::File) {
                button {
                    class: "tab-close",
                    onclick: move |e| {
                        e.stop_propagation();
                        close_tab(&tab.id);
                    },
                    "×"
                }
            }
        }
    }
    
    // New tab button
    button {
        class: "new-tab-button",
        onclick: |_| create_new_file(),
        "+"
    }
}
```

### Step 4: Editor Content Area
```rust
// Show content based on active tab
div {
    class: "editor-content",
    if let Some(tab_id) = active_tab.read().as_ref() {
        if let Some(tab) = find_tab(tab_id) {
            match tab.tab_type {
                TabType::Analytics => {
                    // Analytics dashboard component
                    AnalyticsView { analytics_data }
                }
                TabType::Welcome => {
                    // Welcome screen
                    WelcomeScreen { }
                }
                TabType::File => {
                    // Code editor with syntax highlighting
                    CodeEditor {
                        content: tab.content.clone(),
                        language: tab.language.clone(),
                        on_change: move |new_content| {
                            update_tab_content(tab_id, new_content);
                            mark_tab_dirty(tab_id);
                        }
                    }
                }
            }
        }
    } else {
        // No tabs open
        div { class: "empty-state", "Open a file to start editing" }
    }
}
```

### Step 5: File Operations
```rust
// Save file (Ctrl+S)
fn save_active_file() {
    if let Some(tab_id) = active_tab.read().as_ref() {
        if let Some(tab) = find_tab_mut(tab_id) {
            if let Some(path) = &tab.path {
                match std::fs::write(path, &tab.content) {
                    Ok(_) => {
                        tab.is_dirty = false;
                        show_notification("File saved");
                    }
                    Err(e) => show_error_notification(e),
                }
            }
        }
    }
}

// Keyboard shortcuts
use_effect(move || {
    let eval = eval(r#"
        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 's') {
                e.preventDefault();
                window.save_active_file();
            }
        });
    "#);
});
```

### Step 6: Analytics as a Tab
```rust
// Add analytics tab to the tab bar
fn open_analytics_tab() {
    // Check if already open
    if let Some(tab) = find_analytics_tab() {
        switch_tab(&tab.id);
    } else {
        let tab = EditorTab {
            id: generate_tab_id(),
            path: None,
            title: "Analytics Dashboard".to_string(),
            content: String::new(), // Not used for analytics
            tab_type: TabType::Analytics,
            is_dirty: false,
            language: None,
        };
        editor_state.write().tabs.push(tab);
        active_tab.set(Some(tab.id));
    }
}
```

## CSS Updates

```css
/* Tab bar styling */
.editor-tabs {
    display: flex;
    height: 35px;
    background: #2d2d30;
    border-bottom: 1px solid #3e3e42;
    overflow-x: auto;
}

.editor-tab {
    display: flex;
    align-items: center;
    padding: 0 12px;
    height: 100%;
    background: #2d2d30;
    border-right: 1px solid #3e3e42;
    cursor: pointer;
    min-width: 120px;
    max-width: 200px;
}

.editor-tab.active {
    background: #1e1e1e;
}

.tab-icon {
    margin-right: 6px;
}

.dirty-indicator {
    color: #fff;
    margin-left: 4px;
    font-size: 16px;
}

.tab-close {
    margin-left: auto;
    background: none;
    border: none;
    color: #858585;
    cursor: pointer;
    padding: 2px 6px;
}

.tab-close:hover {
    color: #fff;
    background: rgba(255,255,255,0.1);
}
```

## Future Enhancements

1. **Syntax Highlighting**: Use tree-sitter or similar for proper highlighting
2. **Code Completion**: LSP integration for IntelliSense
3. **Find/Replace**: Ctrl+F functionality
4. **Split View**: Multiple editors side by side
5. **File Tree Context Menu**: Right-click for rename, delete, etc.
6. **Consensus Integration**: Allow AI to modify open files with user approval

## Migration Path

1. Keep existing single-file state temporarily for backward compatibility
2. Implement new tab system alongside
3. Gradually migrate features to use tab system
4. Remove old single-file state once stable

This implementation will give us a proper VS Code-like IDE experience where:
- Files open in tabs
- Multiple files can be open
- Analytics is just another tab
- Files can be edited and saved
- Everything works like developers expect from VS Code