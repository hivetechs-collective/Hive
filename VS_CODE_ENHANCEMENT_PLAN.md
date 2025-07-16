# VS Code Enhancement Plan for Hive Consensus IDE

## Current Status

### ✅ Components Created (Not Yet Integrated)

1. **Activity Bar** (`src/desktop/activity_bar.rs`)
   - VS Code-style 48px vertical bar
   - Items: Explorer, Search, Consensus, Models, Extensions
   - Badge support (notifications)
   - HiveTechs yellow accent for active items
   - Bottom section for accounts/settings
   - Complete with CSS styles

2. **Enhanced Explorer** (`src/desktop/explorer_enhanced.rs`)
   - Collapsible folder tree with chevrons
   - File decorations (Git status, problems)
   - Multi-select support
   - Inline rename functionality
   - Search/filter box
   - Context-aware state management
   - Complete with CSS styles

3. **Enhanced Status Bar** (`src/desktop/status_bar_enhanced.rs`)
   - Left: Git branch, sync, problems, warnings
   - Right: Consensus status, API usage, cursor position
   - Dynamic background colors
   - Real-time update methods
   - HiveTechs branding integration
   - Complete with CSS styles

4. **VS Code Context Menu** (`src/desktop/context_menu_vscode.rs`)
   - Complete menu structure matching VS Code
   - Menu item types: Action, Separator, Submenu, Checkbox
   - Context-aware menu building
   - Keyboard navigation support
   - Smooth animations

5. **Welcome Experience** (`src/desktop/welcome_enhanced.rs`)
   - HiveTechs branded welcome screen
   - Start section with quick actions
   - Recent projects list
   - Interactive walkthroughs
   - Help & resources
   - Complete with CSS styles

6. **Layout Management** (`src/desktop/layout_enhanced.rs`)
   - Flexible panel system
   - Resizable splitters
   - Zen mode support
   - Layout persistence preparation
   - Responsive design
   - Complete with CSS styles

### ❌ Integration Status

**CRITICAL**: None of these components are integrated into the main application (`src/bin/hive-consensus.rs`)!

## Critical Implementation Principles

### ❌ NO STUBBED/FAKE DATA
- **NO** hardcoded file lists - must read actual file system
- **NO** fake Git status - must use real `git status` commands
- **NO** dummy API usage numbers - must track real consensus costs
- **NO** placeholder recent projects - must read from actual database
- **NO** mock walkthroughs - must have real interactive tutorials
- **NO** static status messages - must reflect actual system state

### ✅ REAL IMPLEMENTATIONS ONLY
- File operations must actually create/delete/rename files
- Git decorations must run actual git commands
- API usage must query real database tracking
- Recent projects must load from user's history
- Context menus must perform real actions
- Status bar must show live data

## Integration Plan

### Step 1: Update the Main Application Structure

The main file `src/bin/hive-consensus.rs` needs to be restructured to use the new components:

```rust
// Current structure uses:
- Basic sidebar with file tree
- Simple consensus panel
- Basic context menu

// Needs to change to:
- VSCodeLayout wrapper
- ActivityBar component
- EnhancedExplorer component  
- EnhancedStatusBar component
- VSCodeContextMenu component
```

### Step 2: State Management Updates

Create unified state management for:
- `ActivityBarState` - which panel is active
- `ExplorerState` - enhanced file tree state
- `StatusBarState` - status items and updates
- `LayoutState` - panel visibility and sizes
- `VSCodeContextMenuState` - context menu state

### Step 3: Event Handling Integration

Connect all the event handlers:
- Activity bar item clicks → change active panel
- Explorer file operations → file system actions
- Status bar item clicks → respective actions
- Layout splitter drags → resize panels
- Context menu actions → file operations

### Step 4: Replace Existing Components

1. Replace the current sidebar file tree with `EnhancedExplorer`
2. Add `ActivityBar` to the left side
3. Add `EnhancedStatusBar` to the bottom
4. Replace current context menu with `VSCodeContextMenu`
5. Wrap everything in `VSCodeLayout`
6. Add `EnhancedWelcome` as a tab option

## Detailed TODO List

### 🔴 High Priority (Core Integration with REAL Features)

1. **Refactor Main Application (`src/bin/hive-consensus.rs`)**
   - [ ] Import all new components
   - [ ] Create unified state structure
   - [ ] Replace hardcoded UI with component-based structure
   - [ ] Connect event handlers to REAL operations
   - [ ] Test with ACTUAL file system operations

2. **State Management**
   - [ ] Create `AppState` struct combining all component states
   - [ ] Add state persistence for layout
   - [ ] Implement state synchronization between components
   - [ ] Add state change notifications

3. **Event System**
   - [ ] Connect activity bar to panel switching
   - [ ] Wire up file operations from explorer
   - [ ] Implement status bar updates from consensus
   - [ ] Handle layout resize events

### 🟡 Medium Priority (Feature Completion)

4. **Explorer Enhancements (REAL Implementation)**
   - [ ] Use `std::fs` for actual file operations (create, delete, rename)
   - [ ] Execute `git status --porcelain` for real Git decorations
   - [ ] Use `notify` crate for real file system watching
   - [ ] Implement actual drag-and-drop with file moves
   - [ ] Use `ripgrep` for real file content search

5. **Context Menu Improvements**
   - [ ] Add remaining menu items
   - [ ] Implement keyboard navigation
   - [ ] Add submenu animations
   - [ ] Implement all file operations

6. **Status Bar Features (REAL Data)**
   - [ ] Query actual consensus engine state
   - [ ] Read API costs from database activity_logs table
   - [ ] Execute real actions on click (open terminal, switch branch)
   - [ ] Show actual progress from running operations

### 🟢 Low Priority (Polish)

7. **Welcome Screen (REAL Content)**
   - [ ] Query conversations table for actual recent projects
   - [ ] Create real interactive walkthrough system
   - [ ] Execute actual configuration commands
   - [ ] Open real documentation URLs

8. **Layout Persistence**
   - [ ] Implement localStorage saving
   - [ ] Add layout presets
   - [ ] Implement workspace saving
   - [ ] Add responsive breakpoints

9. **Theme Integration**
   - [ ] Ensure all components use theme variables
   - [ ] Add theme switcher
   - [ ] Implement high contrast mode
   - [ ] Add custom theme support

## Integration Example

Here's how the main application should be restructured:

```rust
#[component]
fn App() -> Element {
    // Unified state
    let layout_state = use_signal(|| LayoutState::default());
    let activity_state = use_signal(|| ActivityBarState::default());
    let explorer_state = use_signal(|| ExplorerState::default());
    let status_bar_state = use_signal(|| StatusBarState::default());
    let context_menu_state = use_signal(|| VSCodeContextMenuState::default());
    
    rsx! {
        VSCodeLayout {
            state: layout_state,
            
            // Activity Bar
            activity_bar: rsx! {
                ActivityBar {
                    state: activity_state,
                    on_item_click: move |id| {
                        // Switch active panel
                    },
                }
            },
            
            // Sidebar
            sidebar: rsx! {
                match activity_state.read().active_item.as_deref() {
                    Some("explorer") => rsx! {
                        EnhancedExplorer {
                            state: explorer_state,
                            on_file_select: move |path| {
                                // Handle file selection
                            },
                        }
                    },
                    Some("consensus") => rsx! {
                        ConsensusPanel { /* ... */ }
                    },
                    _ => rsx! { div {} }
                }
            },
            
            // Editor area
            editor: rsx! {
                // Current editor/welcome content
            },
            
            // Status Bar
            status_bar: rsx! {
                EnhancedStatusBar {
                    state: status_bar_state,
                    on_item_click: move |id| {
                        // Handle status bar clicks
                    },
                }
            },
        }
        
        // Context Menu
        VSCodeContextMenu {
            state: context_menu_state,
            on_action: move |action| {
                // Handle context menu actions
            },
        }
    }
}
```

## Key Challenges to Address

1. **State Synchronization**: Multiple components need to share state
2. **Event Propagation**: Complex event flows between components
3. **Performance**: Virtual scrolling for large file trees
4. **Compatibility**: Ensuring existing features still work
5. **Testing**: Need comprehensive testing of all interactions

## Success Metrics

- [ ] All VS Code-style components visible and functional
- [ ] File operations work through new UI
- [ ] Consensus integration maintained
- [ ] Performance remains smooth
- [ ] HiveTechs branding preserved
- [ ] User can perform all previous actions

## Next Immediate Steps

1. Start with minimal integration - just add ActivityBar to existing UI
2. Test that it displays correctly
3. Gradually replace components one by one
4. Test each integration thoroughly
5. Polish and optimize

## File Locations Reference

- Main app: `/src/bin/hive-consensus.rs`
- New components: `/src/desktop/[component_name].rs`
- Styles: Already integrated in `/src/desktop/styles/mod.rs`
- Module exports: Already added to `/src/desktop/mod.rs`

## Critical Notes

1. **Components Not Integrated**: The current application is NOT using any of the new components yet. They exist but are not imported or instantiated in the main application. This is why you see no visual changes when running the app.

2. **No Fake Data Policy**: Every feature must be backed by real functionality:
   - File tree must use actual `std::fs::read_dir()`
   - Git status must execute actual `git` commands
   - API usage must query real database
   - Recent projects must come from actual user history
   - All operations must have real effects on the system

## Real Implementation Examples

### ❌ WRONG (Stubbed):
```rust
let recent_projects = vec![
    RecentProject {
        name: "hive-consensus".to_string(),
        path: "~/Developer/Private/hive".to_string(),
        last_opened: "2 hours ago".to_string(),
    }
];
```

### ✅ CORRECT (Real):
```rust
let recent_projects = {
    let db = get_database().await?;
    let projects = db.query_row(
        "SELECT DISTINCT project_path, MAX(created_at) as last_opened 
         FROM conversations 
         WHERE project_path IS NOT NULL 
         GROUP BY project_path 
         ORDER BY last_opened DESC 
         LIMIT 10"
    )?;
    projects.into_iter().map(|row| {
        RecentProject {
            name: Path::new(&row.project_path).file_name().unwrap().to_string(),
            path: row.project_path,
            last_opened: format_relative_time(row.last_opened),
        }
    }).collect()
};
```