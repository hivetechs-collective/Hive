# VS Code UI Patterns Analysis for Hive Consensus IDE

## Executive Summary

This document analyzes VS Code UI patterns and provides actionable implementation strategies for the Hive Consensus IDE using Rust/Dioxus. The analysis focuses on maintaining HiveTechs branding while adopting VS Code's proven UX patterns.

## Key VS Code UI Patterns to Migrate

### 1. Activity Bar Behavior and Interactions

**VS Code Pattern:**
- Fixed width (48px) vertical bar on the far left
- Icon-based navigation with tooltips
- Active item indicator (colored left border)
- Badge notifications on icons
- Right-click context menus for customization

**Dioxus Implementation:**
```rust
#[component]
pub fn ActivityBar() -> Element {
    let mut active_view = use_signal(|| ActivityView::Explorer);
    
    rsx! {
        div {
            class: "activity-bar",
            style: "width: 48px; background: var(--vscode-activityBar-background);",
            
            ActivityBarItem {
                icon: "fa-solid fa-files",
                view: ActivityView::Explorer,
                tooltip: "Explorer (Ctrl+Shift+E)",
                badge: None,
                active: active_view.read() == &ActivityView::Explorer,
                onclick: move |_| active_view.set(ActivityView::Explorer),
            }
            
            ActivityBarItem {
                icon: "fa-solid fa-magnifying-glass",
                view: ActivityView::Search,
                tooltip: "Search (Ctrl+Shift+F)",
                badge: Some("3"), // Number of search results
                active: active_view.read() == &ActivityView::Search,
                onclick: move |_| active_view.set(ActivityView::Search),
            }
            
            ActivityBarItem {
                icon: "fa-solid fa-code-branch",
                view: ActivityView::SourceControl,
                tooltip: "Source Control (Ctrl+Shift+G)",
                badge: Some("5"), // Number of changes
                active: active_view.read() == &ActivityView::SourceControl,
                onclick: move |_| active_view.set(ActivityView::SourceControl),
            }
            
            // HiveTechs-specific: Consensus View
            ActivityBarItem {
                icon: "fa-solid fa-brain",
                view: ActivityView::Consensus,
                tooltip: "Consensus Intelligence",
                badge: None,
                active: active_view.read() == &ActivityView::Consensus,
                onclick: move |_| active_view.set(ActivityView::Consensus),
            }
        }
    }
}

#[component]
fn ActivityBarItem(
    icon: &'static str,
    view: ActivityView,
    tooltip: &'static str,
    badge: Option<&'static str>,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            class: if active { "activity-item active" } else { "activity-item" },
            onclick: move |evt| onclick.call(evt),
            oncontextmenu: move |evt| show_activity_context_menu(evt, view),
            
            i { class: icon }
            
            if let Some(count) = badge {
                span { class: "activity-badge", "{count}" }
            }
            
            // Tooltip on hover
            div {
                class: "activity-tooltip",
                "{tooltip}"
            }
        }
    }
}
```

**CSS Styling:**
```css
.activity-bar {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 4px 0;
}

.activity-item {
    position: relative;
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--vscode-activityBar-inactiveForeground);
    transition: all 0.1s ease;
}

.activity-item:hover {
    color: var(--vscode-activityBar-foreground);
}

.activity-item.active {
    color: var(--vscode-activityBar-foreground);
    position: relative;
}

.activity-item.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 8px;
    bottom: 8px;
    width: 2px;
    background: var(--hive-yellow); /* HiveTechs brand color */
}

.activity-badge {
    position: absolute;
    top: 4px;
    right: 4px;
    background: var(--hive-yellow);
    color: #000;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 4px;
    border-radius: 10px;
    min-width: 16px;
    text-align: center;
}

.activity-tooltip {
    display: none;
    position: absolute;
    left: 52px;
    top: 50%;
    transform: translateY(-50%);
    background: #1e1e1e;
    border: 1px solid #454545;
    padding: 4px 8px;
    font-size: 12px;
    white-space: nowrap;
    z-index: 1000;
}

.activity-item:hover .activity-tooltip {
    display: block;
}
```

### 2. Explorer Panel Features

**VS Code Pattern:**
- Collapsible folders with chevron icons
- File decorations (Git status, problems)
- Multi-select with Ctrl/Cmd
- Drag and drop support
- Context menus (New File, Rename, Delete)
- Search/filter box at top

**Dioxus Implementation:**
```rust
#[component]
pub fn EnhancedFileExplorer() -> Element {
    let mut expanded_folders = use_signal(|| HashSet::<PathBuf>::new());
    let mut selected_files = use_signal(|| HashSet::<PathBuf>::new());
    let mut search_query = use_signal(|| String::new());
    
    rsx! {
        div {
            class: "explorer-panel",
            
            // Search/Filter Box
            div {
                class: "explorer-search",
                input {
                    r#type: "text",
                    placeholder: "Filter files...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                }
            }
            
            // Toolbar
            div {
                class: "explorer-toolbar",
                button {
                    class: "explorer-action",
                    onclick: |_| create_new_file(),
                    title: "New File",
                    i { class: "fa-solid fa-file-plus" }
                }
                button {
                    class: "explorer-action",
                    onclick: |_| create_new_folder(),
                    title: "New Folder",
                    i { class: "fa-solid fa-folder-plus" }
                }
                button {
                    class: "explorer-action",
                    onclick: |_| refresh_explorer(),
                    title: "Refresh",
                    i { class: "fa-solid fa-arrows-rotate" }
                }
                button {
                    class: "explorer-action",
                    onclick: |_| collapse_all_folders(),
                    title: "Collapse All",
                    i { class: "fa-solid fa-compress" }
                }
            }
            
            // File Tree
            div {
                class: "file-tree",
                ondrop: move |evt| handle_file_drop(evt),
                ondragover: |evt| evt.prevent_default(),
                
                FileTreeNode {
                    path: root_path.clone(),
                    expanded_folders: expanded_folders.clone(),
                    selected_files: selected_files.clone(),
                    search_query: search_query.read().clone(),
                }
            }
        }
    }
}

#[component]
fn FileTreeNode(
    path: PathBuf,
    expanded_folders: Signal<HashSet<PathBuf>>,
    selected_files: Signal<HashSet<PathBuf>>,
    search_query: String,
) -> Element {
    let entries = std::fs::read_dir(&path).ok()?;
    let mut items = Vec::new();
    
    for entry in entries {
        if let Ok(entry) = entry {
            let entry_path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            // Filter by search query
            if !search_query.is_empty() && !file_name.contains(&search_query) {
                continue;
            }
            
            items.push((entry_path, entry.file_type().ok()?.is_dir()));
        }
    }
    
    // Sort: directories first, then alphabetically
    items.sort_by(|a, b| {
        match (a.1, b.1) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.file_name().cmp(&b.0.file_name()),
        }
    });
    
    rsx! {
        for (item_path, is_dir) in items {
            div {
                class: "tree-node",
                
                FileTreeItem {
                    path: item_path.clone(),
                    is_directory: is_dir,
                    expanded_folders: expanded_folders.clone(),
                    selected_files: selected_files.clone(),
                    git_status: get_git_status(&item_path),
                }
                
                if is_dir && expanded_folders.read().contains(&item_path) {
                    div {
                        class: "tree-indent",
                        FileTreeNode {
                            path: item_path,
                            expanded_folders: expanded_folders.clone(),
                            selected_files: selected_files.clone(),
                            search_query: search_query.clone(),
                        }
                    }
                }
            }
        }
    }
}
```

**File Decorations:**
```rust
fn get_file_decoration(path: &Path) -> FileDecoration {
    FileDecoration {
        git_status: get_git_status(path),
        problems: get_file_problems(path),
        is_modified: is_file_modified(path),
    }
}

#[component]
fn FileDecorations(decoration: FileDecoration) -> Element {
    rsx! {
        span {
            class: "file-decorations",
            
            // Git status indicator
            if let Some(status) = decoration.git_status {
                span {
                    class: format!("git-status git-{}", status.to_string()),
                    title: format!("Git: {}", status.to_string()),
                    match status {
                        GitStatus::Modified => "M",
                        GitStatus::Added => "A",
                        GitStatus::Deleted => "D",
                        GitStatus::Renamed => "R",
                        GitStatus::Untracked => "U",
                    }
                }
            }
            
            // Problem count
            if decoration.problems > 0 {
                span {
                    class: "problem-badge",
                    title: format!("{} problems", decoration.problems),
                    "{decoration.problems}"
                }
            }
            
            // Modified indicator
            if decoration.is_modified {
                span {
                    class: "modified-indicator",
                    "•"
                }
            }
        }
    }
}
```

### 3. Layout System and Panel Management

**VS Code Pattern:**
- Flexible panel system with drag-to-resize
- Panels can be minimized, maximized, or closed
- Layout persistence across sessions
- Split views (horizontal/vertical)

**Dioxus Implementation:**
```rust
#[derive(Clone, Copy)]
struct PanelLayout {
    sidebar_width: f32,      // Percentage
    editor_width: f32,       // Percentage
    sidebar_visible: bool,
    panel_visible: bool,
    panel_height: f32,       // Percentage
}

#[component]
pub fn FlexibleLayout() -> Element {
    let mut layout = use_signal(|| PanelLayout {
        sidebar_width: 20.0,
        editor_width: 50.0,
        sidebar_visible: true,
        panel_visible: true,
        panel_height: 30.0,
    });
    
    // Persist layout to localStorage
    use_effect(move || {
        if let Ok(json) = serde_json::to_string(&*layout.read()) {
            web_sys::window()
                .unwrap()
                .local_storage()
                .unwrap()
                .unwrap()
                .set_item("hive-layout", &json)
                .ok();
        }
    });
    
    rsx! {
        div {
            class: "ide-layout",
            
            // Activity Bar (fixed width)
            ActivityBar {}
            
            // Main content area
            div {
                class: "content-area",
                style: "display: flex; flex: 1;",
                
                // Sidebar (Explorer/Search/etc)
                if layout.read().sidebar_visible {
                    ResizablePanel {
                        width: layout.read().sidebar_width,
                        min_width: 170.0,
                        max_width: 50.0,
                        onresize: move |new_width| {
                            layout.write().sidebar_width = new_width;
                        },
                        
                        SidebarContent {}
                    }
                }
                
                // Editor area
                div {
                    class: "editor-area",
                    style: format!("flex: {};", layout.read().editor_width / 100.0),
                    
                    EditorTabs {}
                    EditorContent {}
                }
                
                // Right panel (Consensus/Chat)
                ResizablePanel {
                    width: 100.0 - layout.read().sidebar_width - layout.read().editor_width,
                    min_width: 200.0,
                    max_width: 50.0,
                    onresize: move |new_width| {
                        // Adjust editor width when right panel resizes
                    },
                    
                    ConsensusPanel {}
                }
            }
            
            // Bottom panel (Terminal/Problems/Output)
            if layout.read().panel_visible {
                ResizablePanel {
                    height: layout.read().panel_height,
                    min_height: 100.0,
                    max_height: 80.0,
                    orientation: PanelOrientation::Horizontal,
                    onresize: move |new_height| {
                        layout.write().panel_height = new_height;
                    },
                    
                    BottomPanel {}
                }
            }
            
            // Status bar (fixed height)
            StatusBar {}
        }
    }
}

#[component]
fn ResizablePanel(
    #[props(default = 300.0)] width: f32,
    #[props(default = 200.0)] min_width: f32,
    #[props(default = 600.0)] max_width: f32,
    #[props(default = PanelOrientation::Vertical)] orientation: PanelOrientation,
    onresize: EventHandler<f32>,
    children: Element,
) -> Element {
    let mut is_resizing = use_signal(|| false);
    let mut start_pos = use_signal(|| 0.0);
    let mut start_size = use_signal(|| width);
    
    rsx! {
        div {
            class: "resizable-panel",
            style: match orientation {
                PanelOrientation::Vertical => format!("width: {}%;", width),
                PanelOrientation::Horizontal => format!("height: {}px;", width),
            },
            
            {children}
            
            // Resize handle
            div {
                class: match orientation {
                    PanelOrientation::Vertical => "resize-handle-vertical",
                    PanelOrientation::Horizontal => "resize-handle-horizontal",
                },
                onmousedown: move |evt| {
                    is_resizing.set(true);
                    start_pos.set(match orientation {
                        PanelOrientation::Vertical => evt.client_x() as f32,
                        PanelOrientation::Horizontal => evt.client_y() as f32,
                    });
                    start_size.set(width);
                },
                onmousemove: move |evt| {
                    if *is_resizing.read() {
                        let current_pos = match orientation {
                            PanelOrientation::Vertical => evt.client_x() as f32,
                            PanelOrientation::Horizontal => evt.client_y() as f32,
                        };
                        let delta = current_pos - *start_pos.read();
                        let new_size = (*start_size.read() + delta).clamp(min_width, max_width);
                        onresize.call(new_size);
                    }
                },
                onmouseup: move |_| {
                    is_resizing.set(false);
                },
            }
        }
    }
}
```

### 4. Status Bar Components and Updates

**VS Code Pattern:**
- Fixed height bottom bar
- Left side: current branch, sync status, problems/warnings
- Right side: line/column, spaces/tabs, encoding, language mode
- Click actions on each item
- Background color changes based on state

**Dioxus Implementation:**
```rust
#[component]
pub fn EnhancedStatusBar() -> Element {
    let git_branch = use_signal(|| get_current_branch());
    let cursor_position = use_signal(|| CursorPosition { line: 1, column: 1 });
    let file_info = use_signal(|| FileInfo::default());
    let consensus_status = use_context::<Signal<ConsensusStatus>>();
    
    rsx! {
        div {
            class: "status-bar",
            style: get_status_bar_style(&consensus_status.read()),
            
            // Left side items
            div {
                class: "status-bar-left",
                
                // Git branch
                StatusBarItem {
                    icon: "fa-solid fa-code-branch",
                    text: git_branch.read().clone(),
                    tooltip: "Current branch",
                    onclick: |_| show_branch_picker(),
                }
                
                // Sync status
                StatusBarItem {
                    icon: "fa-solid fa-arrows-rotate",
                    text: "↓0 ↑0",
                    tooltip: "Synchronize Changes",
                    onclick: |_| sync_changes(),
                }
                
                // Problems/Warnings
                StatusBarItem {
                    icon: "fa-solid fa-circle-exclamation",
                    text: format!("{} {}", problems_count(), warnings_count()),
                    tooltip: "Problems",
                    onclick: |_| show_problems_panel(),
                    class: if problems_count() > 0 { "has-problems" } else { "" },
                }
                
                // HiveTechs: Consensus status
                StatusBarItem {
                    icon: "fa-solid fa-brain",
                    text: consensus_status.read().to_string(),
                    tooltip: "Consensus Engine Status",
                    onclick: |_| toggle_consensus_panel(),
                    class: "hive-consensus-status",
                }
            }
            
            // Right side items
            div {
                class: "status-bar-right",
                
                // Cursor position
                StatusBarItem {
                    text: format!("Ln {}, Col {}", 
                        cursor_position.read().line, 
                        cursor_position.read().column
                    ),
                    tooltip: "Go to Line/Column",
                    onclick: |_| show_goto_line_dialog(),
                }
                
                // Indentation
                StatusBarItem {
                    text: if file_info.read().uses_tabs { "Tab Size: 4" } else { "Spaces: 4" },
                    tooltip: "Select Indentation",
                    onclick: |_| show_indentation_picker(),
                }
                
                // Encoding
                StatusBarItem {
                    text: file_info.read().encoding.clone(),
                    tooltip: "Select Encoding",
                    onclick: |_| show_encoding_picker(),
                }
                
                // Language mode
                StatusBarItem {
                    text: file_info.read().language.clone(),
                    tooltip: "Select Language Mode",
                    onclick: |_| show_language_picker(),
                }
                
                // HiveTechs: API usage
                StatusBarItem {
                    icon: "fa-solid fa-coins",
                    text: format!("${:.2}", get_api_usage_today()),
                    tooltip: "OpenRouter API Usage Today",
                    class: "hive-api-usage",
                }
            }
        }
    }
}

#[component]
fn StatusBarItem(
    #[props(default = "")] icon: &'static str,
    text: String,
    tooltip: &'static str,
    onclick: EventHandler<MouseEvent>,
    #[props(default = "")] class: &'static str,
) -> Element {
    rsx! {
        button {
            class: format!("status-bar-item {}", class),
            title: tooltip,
            onclick: move |evt| onclick.call(evt),
            
            if !icon.is_empty() {
                i { class: icon, style: "margin-right: 4px;" }
            }
            span { "{text}" }
        }
    }
}

fn get_status_bar_style(consensus_status: &ConsensusStatus) -> String {
    match consensus_status {
        ConsensusStatus::Processing => "background: var(--hive-yellow);",
        ConsensusStatus::Error => "background: var(--vscode-statusBar-debuggingBackground);",
        _ => "",
    }
}
```

### 5. Welcome/Getting Started Experience

**VS Code Pattern:**
- Clean welcome tab with quick actions
- Recent files/folders list
- Quick command shortcuts
- Learning resources
- Walkthroughs for new features

**Dioxus Implementation:**
```rust
#[component]
pub fn WelcomeTab() -> Element {
    let recent_projects = use_signal(|| get_recent_projects());
    let show_walkthrough = use_signal(|| false);
    
    rsx! {
        div {
            class: "welcome-tab",
            
            // Header with branding
            div {
                class: "welcome-header",
                img {
                    src: "assets/hive-logo.svg",
                    alt: "HiveTechs Logo",
                    class: "welcome-logo",
                }
                h1 { "Welcome to Hive Consensus IDE" }
                p { 
                    class: "welcome-subtitle",
                    "AI-Powered Development with 4-Stage Consensus Intelligence" 
                }
            }
            
            // Quick actions grid
            div {
                class: "welcome-actions",
                
                WelcomeAction {
                    icon: "fa-solid fa-folder-open",
                    title: "Open Folder",
                    description: "Open a folder to start working",
                    onclick: |_| open_folder_dialog(),
                }
                
                WelcomeAction {
                    icon: "fa-solid fa-plus",
                    title: "New File",
                    description: "Create a new file",
                    shortcut: "Ctrl+N",
                    onclick: |_| create_new_file(),
                }
                
                WelcomeAction {
                    icon: "fa-solid fa-brain",
                    title: "Start Consensus Chat",
                    description: "Ask Hive AI anything",
                    shortcut: "Ctrl+Shift+C",
                    onclick: |_| focus_consensus_chat(),
                }
                
                WelcomeAction {
                    icon: "fa-solid fa-terminal",
                    title: "Open Terminal",
                    description: "Toggle integrated terminal",
                    shortcut: "Ctrl+`",
                    onclick: |_| toggle_terminal(),
                }
            }
            
            // Recent projects
            div {
                class: "welcome-section",
                h2 { "Recent" }
                
                if recent_projects.read().is_empty() {
                    p { class: "muted", "No recent folders" }
                } else {
                    ul {
                        class: "recent-list",
                        for project in recent_projects.read().iter().take(10) {
                            RecentProjectItem { project: project.clone() }
                        }
                    }
                }
            }
            
            // Learning resources
            div {
                class: "welcome-section",
                h2 { "Get Started" }
                
                div {
                    class: "resource-links",
                    
                    ResourceLink {
                        icon: "fa-solid fa-graduation-cap",
                        text: "Interactive Tutorial",
                        onclick: move |_| show_walkthrough.set(true),
                    }
                    
                    ResourceLink {
                        icon: "fa-solid fa-book",
                        text: "Read Documentation",
                        onclick: |_| open_documentation(),
                    }
                    
                    ResourceLink {
                        icon: "fa-solid fa-keyboard",
                        text: "Keyboard Shortcuts",
                        onclick: |_| show_keyboard_shortcuts(),
                    }
                    
                    ResourceLink {
                        icon: "fa-solid fa-video",
                        text: "Video Tutorials",
                        onclick: |_| open_video_tutorials(),
                    }
                }
            }
            
            // Walkthrough overlay
            if *show_walkthrough.read() {
                InteractiveWalkthrough {
                    onclose: move |_| show_walkthrough.set(false),
                }
            }
        }
    }
}

#[component]
fn InteractiveWalkthrough(onclose: EventHandler<()>) -> Element {
    let current_step = use_signal(|| 0);
    let steps = vec![
        WalkthroughStep {
            title: "Welcome to Hive Consensus IDE",
            content: "Let's take a quick tour of the AI-powered features",
            target: None,
        },
        WalkthroughStep {
            title: "4-Stage Consensus Engine",
            content: "Our AI uses Generator → Refiner → Validator → Curator stages",
            target: Some(".consensus-panel"),
        },
        WalkthroughStep {
            title: "Real-time Code Analysis",
            content: "Get instant feedback on code quality and suggestions",
            target: Some(".editor-area"),
        },
        // ... more steps
    ];
    
    rsx! {
        div {
            class: "walkthrough-overlay",
            onclick: move |_| onclose.call(()),
            
            div {
                class: "walkthrough-content",
                onclick: |evt| evt.stop_propagation(),
                
                h3 { "{steps[*current_step.read()].title}" }
                p { "{steps[*current_step.read()].content}" }
                
                div {
                    class: "walkthrough-actions",
                    
                    button {
                        class: "btn-secondary",
                        onclick: move |_| {
                            if *current_step.read() > 0 {
                                current_step.set(*current_step.read() - 1);
                            }
                        },
                        disabled: *current_step.read() == 0,
                        "Previous"
                    }
                    
                    span { 
                        class: "step-indicator",
                        "{} of {}", *current_step.read() + 1, steps.len() 
                    }
                    
                    if *current_step.read() < steps.len() - 1 {
                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                current_step.set(*current_step.read() + 1);
                            },
                            "Next"
                        }
                    } else {
                        button {
                            class: "btn-primary",
                            onclick: move |_| onclose.call(()),
                            "Get Started"
                        }
                    }
                }
            }
        }
    }
}
```

### 6. CSS Architecture and Theming

**VS Code Pattern:**
- CSS variables for all colors
- Semantic color naming
- Theme-aware components
- Smooth transitions
- Platform-specific adjustments

**Enhanced CSS Architecture:**
```css
/* Theme System */
:root {
    /* VS Code Base Colors */
    --vscode-foreground: #cccccc;
    --vscode-background: #1e1e1e;
    
    /* Semantic Colors */
    --color-success: #89d185;
    --color-warning: #e9c46a;
    --color-error: #f48771;
    --color-info: #75beff;
    
    /* HiveTechs Brand Integration */
    --hive-accent: #FFC107;
    --hive-accent-hover: #FFD54F;
    --hive-background: #0E1414;
    --hive-surface: #181E21;
    
    /* Component-specific */
    --panel-border-width: 1px;
    --panel-resize-handle: 4px;
    --activity-bar-width: 48px;
    --status-bar-height: 22px;
    --tab-height: 35px;
    
    /* Animations */
    --transition-fast: 0.1s ease;
    --transition-normal: 0.2s ease;
    --transition-slow: 0.3s ease;
}

/* Dark Theme with HiveTechs Branding */
.theme-dark {
    --vscode-editor-background: var(--hive-background);
    --vscode-sideBar-background: var(--hive-surface);
    --vscode-activityBar-activeBorder: var(--hive-accent);
    --vscode-statusBar-background: var(--hive-background);
    --vscode-button-background: var(--hive-accent);
    --vscode-button-foreground: #000000;
}

/* Smooth theme transitions */
* {
    transition: background-color var(--transition-normal),
                color var(--transition-normal),
                border-color var(--transition-normal);
}

/* Platform-specific adjustments */
@media (prefers-color-scheme: dark) {
    :root {
        color-scheme: dark;
    }
}

/* macOS-specific */
.platform-darwin {
    --font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', sans-serif;
    --font-family-mono: 'SF Mono', Monaco, monospace;
}

.platform-darwin .titlebar {
    -webkit-app-region: drag;
    padding-left: 70px; /* Space for traffic lights */
}

/* Windows-specific */
.platform-win32 {
    --font-family: 'Segoe UI', sans-serif;
    --font-family-mono: 'Cascadia Code', Consolas, monospace;
}

/* Linux-specific */
.platform-linux {
    --font-family: Ubuntu, 'DejaVu Sans', sans-serif;
    --font-family-mono: 'Ubuntu Mono', monospace;
}

/* Responsive breakpoints */
@media (max-width: 768px) {
    .activity-bar { display: none; }
    .sidebar { position: absolute; z-index: 100; }
}
```

## Implementation Priority

1. **Phase 1: Core Layout**
   - Activity bar with HiveTechs branding
   - Resizable panel system
   - Status bar with real-time updates

2. **Phase 2: Explorer Enhancement**
   - File decorations and Git status
   - Context menus
   - Multi-select support

3. **Phase 3: Editor Integration**
   - Tab management system
   - Split view support
   - File dirty indicators

4. **Phase 4: Consensus Features**
   - Real-time consensus progress
   - Integrated chat panel
   - Code analysis overlays

5. **Phase 5: Polish**
   - Welcome experience
   - Interactive walkthrough
   - Keyboard shortcut system

## Key Differentiators

1. **HiveTechs Yellow Accent** - Replace VS Code blue with brand yellow
2. **Consensus Panel** - Unique 4-stage AI visualization
3. **Real-time Collaboration** - Show other users' cursors and selections
4. **AI-Powered Context Menus** - Add "Ask Hive about this" to all menus
5. **Smart Status Indicators** - Show API usage, consensus status, sync state

## Performance Considerations

1. **Virtual Scrolling** - For large file trees
2. **Lazy Loading** - Load file contents on demand
3. **Debounced Resizing** - Smooth panel resize performance
4. **CSS Containment** - Isolate reflow/repaint to panels
5. **Web Workers** - Offload syntax highlighting and file parsing

This implementation guide provides a complete roadmap for creating a VS Code-like experience while maintaining the unique HiveTechs identity and AI-powered features.