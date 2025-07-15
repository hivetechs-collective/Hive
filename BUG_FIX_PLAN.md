# Bug Fix Plan for Hive Analytics Desktop App

## Critical Issues Identified

### 1. 🔑 OpenRouter API Key Lost
**Problem**: Creating a custom profile and setting it as default caused the saved OpenRouter API key to be lost.
**Root Cause**: Likely the profile system is overwriting the API key configuration when switching defaults.
**Impact**: High - Users cannot run consensus without re-entering API key

### 2. 🎯 Default Profile Button Non-Functional
**Problem**: "Set as Default" button in profile editor doesn't work when clicked.
**Root Cause**: Missing event handler or state update logic for the default profile setting.
**Impact**: Medium - Users cannot change their default profile preference

### 3. 📐 Right Pane Resize Issues
**Problem**: Right pane maintains fixed width and covers the app when window is resized smaller.
**Root Cause**: CSS flex/grid layout not properly configured for responsive behavior.
**Impact**: Medium - UI becomes unusable on smaller screens

### 4. 📝 File Editor Functionality Lost (VS Code-like IDE)
**Problem**: Clicking files in the left sidebar (file explorer) no longer opens them in editor tabs in the center pane for viewing/editing.
**Root Cause**: Analytics report integration broke the VS Code-like tab system in the center pane.
**Impact**: High - Core IDE functionality is broken. Users cannot:
- Open multiple files in tabs (like VS Code)
- View and edit local file contents
- Save changes to files
- Switch between open file tabs
**Goal**: Restore VS Code/Claude Code functionality where the left sidebar is a file explorer and the center pane shows tabbed file editors

## Solution Architecture

### Phase 1: API Key Persistence Fix
- Separate API key storage from profile configuration
- Implement independent config storage for sensitive data
- Add validation to prevent overwriting API keys

### Phase 2: Profile Management Fix
- Implement proper event handler for "Set as Default" button
- Add state management for default profile selection
- Persist default profile preference correctly

### Phase 3: Responsive Layout Fix
- Convert right pane to use flexible width units
- Implement proper CSS Grid/Flexbox constraints
- Add min/max width boundaries with overflow handling

### Phase 4: VS Code-like File Editor Restoration
- Restore full IDE functionality with tabbed file editing
- Implement VS Code-style tab management in center pane
- File explorer (left) → opens files in editor tabs (center)
- Keep analytics as one special tab (like VS Code's Welcome tab)
- Implement file operations: open, edit, save (Ctrl+S)
- Support multiple file tabs with switching
- Add syntax highlighting for code files
- Future: Enable consensus AI to modify open files (like Claude Code)

## Implementation Plan

### Database Schema Check
```sql
-- Check if we need separate tables for:
-- 1. API keys (encrypted)
-- 2. User preferences (default profile)
-- 3. UI state (tab positions, sizes)
```

### Code Structure
```
src/bin/hive-consensus.rs
├── API Key Management
│   ├── load_api_key()
│   ├── save_api_key()
│   └── validate_api_key()
├── Profile Management
│   ├── set_default_profile()
│   ├── load_default_profile()
│   └── profile_event_handlers()
├── Layout Management
│   ├── responsive_layout()
│   ├── pane_resize_handlers()
│   └── window_size_observers()
└── Tab Management
    ├── file_tab_system()
    ├── tab_switching()
    └── editor_integration()
```

## Technical Details

### 1. API Key Storage Fix
```rust
// Separate config storage for sensitive data
struct SecureConfig {
    openrouter_api_key: Option<String>,
    // Other sensitive data
}

// Profile config should NOT contain API keys
struct ProfileConfig {
    default_profile_id: String,
    // Profile-specific settings only
}
```

### 2. Profile Default Button Fix
```rust
// Add proper event handler
button {
    onclick: move |_| {
        set_default_profile(profile_id.clone());
        default_profile_id.set(profile_id.clone());
        save_user_preferences();
    },
    "Set as Default"
}
```

### 3. Responsive Layout Fix
```css
.right-panel {
    flex: 1 1 400px; /* grow, shrink, basis */
    min-width: 300px;
    max-width: 600px;
    overflow-x: auto;
}

.main-container {
    display: grid;
    grid-template-columns: 250px 1fr minmax(300px, 400px);
    gap: 0;
}

@media (max-width: 1200px) {
    .main-container {
        grid-template-columns: 200px 1fr 300px;
    }
}
```

### 4. VS Code-like File Editor Tab System
```rust
#[derive(Clone, PartialEq)]
enum TabContent {
    Analytics,  // Special tab for analytics dashboard
    FileEditor { 
        path: PathBuf, 
        content: String,
        modified: bool,  // Track unsaved changes
        language: Language,  // For syntax highlighting
    },
    Welcome,  // Like VS Code's start page
}

struct Tab {
    id: String,
    title: String,
    content: TabContent,
    is_dirty: bool,  // Shows dot for unsaved files
}

struct EditorState {
    tabs: Vec<Tab>,
    active_tab_id: Option<String>,
    file_contents: HashMap<PathBuf, String>,
}

// File operations matching VS Code behavior
impl FileOperations {
    fn open_file(path: &Path) -> Tab { /* Open file in new tab */ }
    fn save_file(tab: &Tab) -> Result<()> { /* Save to disk */ }
    fn close_tab(id: &str) -> Result<()> { /* Handle unsaved changes */ }
}
```

## Testing Plan

1. **API Key Persistence**
   - Create new profile → Verify API key retained
   - Switch default profile → Verify API key retained
   - Restart app → Verify API key loaded

2. **Profile Management**
   - Click "Set as Default" → Verify profile marked as default
   - Restart app → Verify default profile loaded
   - Create multiple profiles → Verify only one default

3. **Responsive Layout**
   - Resize window smaller → Verify panes adjust
   - Minimum window size → Verify scrollbars appear
   - Maximum window size → Verify proper spacing

4. **VS Code-like File Editor**
   - Click file in left explorer → Opens in new tab in center pane
   - Multiple files → Multiple tabs with tab bar at top
   - Click tab → Switches to that file
   - Edit file → Shows unsaved indicator (dot)
   - Ctrl+S → Saves file to disk
   - Close tab → Prompts if unsaved changes
   - Analytics tab → Remains as special tab (can't be closed)
   - Syntax highlighting → Works for all supported languages
   - Future: Consensus can edit open files (like Claude Code)

## Risk Mitigation

1. **Data Loss Prevention**
   - Backup current config before changes
   - Implement config migration logic
   - Add rollback capability

2. **UI State Preservation**
   - Save tab states before fixing
   - Preserve user's workspace layout
   - Maintain file edit history

## Success Criteria

- [ ] API key persists across all profile operations
- [ ] "Set as Default" button functions correctly
- [ ] Right pane resizes properly with window
- [ ] VS Code-like file editing restored in center pane
- [ ] Multiple file tabs with proper switching
- [ ] File save functionality (Ctrl+S) works
- [ ] Analytics remains as one tab option
- [ ] No data loss during migration
- [ ] All existing features remain functional

## Rollback Plan

If issues arise:
1. Revert to backup configuration
2. Restore previous codebase state
3. Document any data format changes
4. Communicate with users about temporary workarounds