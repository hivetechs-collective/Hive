# Immediate Bug Fixes Implementation

## Fix 1: API Key / Consensus Engine Reload

The issue is that when creating a custom profile, the consensus engine doesn't reload with the API key. The API key is stored correctly in the database, but the consensus manager needs to be recreated.

### Solution:
```rust
// In hive-consensus.rs, increment api_keys_version when profile changes
// This will trigger consensus manager recreation

// After creating custom profile:
*api_keys_version.write() += 1;

// Or better, create a specific trigger:
let profile_change_trigger = use_signal(|| 0u32);

// When profile is created or set as default:
*profile_change_trigger.write() += 1;

// Update consensus manager hook:
let consensus_manager = use_consensus_with_version(
    *api_keys_version.read() + *profile_change_trigger.read()
);
```

## Fix 2: "Set as Default" Button

The button renders but the onclick handler doesn't work properly.

### Current Issue in dialogs.rs:
```rust
// The function exists but may not be connected properly
onclick: move |_| {
    let profile_id = profile.id;
    let reload_callback = on_reload_profiles.clone();
    spawn(async move {
        if let Err(e) = set_default_profile(profile_id).await {
            tracing::error!("Failed to set default profile: {}", e);
        } else {
            tracing::info!("Set profile {} as default", profile_id);
            reload_callback.call(());
        }
    });
},
```

### Fix:
Ensure the callback properly updates the UI and triggers consensus reload.

## Fix 3: Right Pane Responsive Layout

### Current CSS:
```css
.chat-panel {
    flex: 1;
    background: #1e1e1e;
    display: flex;
    flex-direction: column;
    min-width: 400px; /* This is the problem */
}
```

### Fixed CSS:
```css
.chat-panel {
    flex: 1 1 400px; /* grow, shrink, basis */
    min-width: 300px; /* Smaller minimum */
    max-width: 600px; /* Prevent too wide */
    background: #1e1e1e;
    display: flex;
    flex-direction: column;
}

/* Main container needs update too */
.main-container {
    display: flex;
    width: 100%;
    height: 100vh;
    overflow: hidden;
}

/* Add media query for small screens */
@media (max-width: 1200px) {
    .sidebar { 
        min-width: 180px;
        width: 180px;
    }
    .chat-panel {
        min-width: 250px;
    }
}

/* For very small screens, hide sidebar */
@media (max-width: 768px) {
    .sidebar {
        position: absolute;
        z-index: 100;
        transform: translateX(-100%);
        transition: transform 0.3s;
    }
    .sidebar.open {
        transform: translateX(0);
    }
    .chat-panel {
        min-width: 200px;
    }
}
```

## Fix 4: VS Code-like Multi-Tab Editor

### Step 1: Update State Structure
```rust
// Replace single file state
let mut selected_file = use_signal(|| Some("__welcome__".to_string()));
let mut file_content = use_signal(String::new);

// With multi-tab state
#[derive(Clone, PartialEq)]
struct EditorTab {
    id: String,
    path: Option<PathBuf>,
    title: String,
    content: String,
    is_dirty: bool,
    tab_type: TabType,
}

#[derive(Clone, PartialEq)]
enum TabType {
    Welcome,
    Analytics,
    File,
}

let mut editor_tabs = use_signal(|| vec![
    EditorTab {
        id: "welcome".to_string(),
        path: None,
        title: "Welcome".to_string(),
        content: String::new(),
        is_dirty: false,
        tab_type: TabType::Welcome,
    }
]);
let mut active_tab_id = use_signal(|| "welcome".to_string());
```

### Step 2: Update File Click Handler
```rust
// In the file tree onclick
onclick: move |_| {
    if is_dir {
        // Toggle directory
    } else {
        // Open file in new tab
        let path = file_path.clone();
        let mut editor_tabs = editor_tabs.clone();
        let mut active_tab_id = active_tab_id.clone();
        
        spawn(async move {
            match file_system::read_file_content(&path).await {
                Ok(content) => {
                    // Check if already open
                    let existing = editor_tabs.read()
                        .iter()
                        .find(|t| t.path.as_ref() == Some(&path))
                        .map(|t| t.id.clone());
                    
                    if let Some(id) = existing {
                        // Switch to existing tab
                        *active_tab_id.write() = id;
                    } else {
                        // Create new tab
                        let id = format!("file-{}", uuid::Uuid::new_v4());
                        let title = path.file_name()
                            .unwrap_or("Untitled")
                            .to_string_lossy()
                            .to_string();
                        
                        editor_tabs.write().push(EditorTab {
                            id: id.clone(),
                            path: Some(path),
                            title,
                            content,
                            is_dirty: false,
                            tab_type: TabType::File,
                        });
                        
                        *active_tab_id.write() = id;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                }
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
    // Render all tabs
    for tab in editor_tabs.read().iter() {
        div {
            class: if *active_tab_id.read() == tab.id { 
                "editor-tab active" 
            } else { 
                "editor-tab" 
            },
            onclick: {
                let id = tab.id.clone();
                move |_| *active_tab_id.write() = id.clone()
            },
            
            // Tab content
            span { "{tab.title}" }
            if tab.is_dirty {
                span { style: "color: #fff; margin-left: 4px;", "•" }
            }
            
            // Close button (except for Welcome)
            if tab.tab_type != TabType::Welcome {
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
    
    // Analytics tab button
    button {
        class: "editor-tab",
        onclick: move |_| open_analytics_tab(),
        "📊 Analytics"
    }
}
```

## Implementation Priority

1. **Fix API Key Issue First** - Users can't use consensus without this
2. **Fix "Set as Default" Button** - Related to API key issue
3. **Fix Right Pane Sizing** - UI is unusable on smaller screens
4. **Implement Multi-Tab Editor** - Core IDE functionality

## Testing Plan

1. Create custom profile → Verify consensus still works
2. Set profile as default → Verify it's marked and consensus uses it
3. Resize window → Verify panes adjust properly
4. Click multiple files → Verify tabs open and switch correctly