# Tauri Migration Plan - COMPLETE FEATURE PARITY

## Core Principle
**Replace ONLY the Dioxus UI layer with Tauri + Web UI. Keep ALL existing business logic, database, and features intact.**

## Current Implementation Status
### ✅ Completed (30%)
- Basic 3-panel layout (sidebar, editor, chat)
- HiveTechs branding and logo
- File explorer in sidebar
- Single terminal at bottom
- Consensus chat panel with 4-stage display
- Basic analytics view (placeholder)
- Terminal bridge to existing PTY
- Consensus engine connection
- Database connection

### ❌ Missing Features (70%)
The frontend currently lacks critical functionality that exists in the Dioxus version.

## Complete Feature Parity Requirements

### Phase 1: Critical Core Features (Must Have)
These are blocking features - the app is unusable without them:

#### 1.1 Profile Selector
- **Frontend**: Dropdown in chat panel header
- **Bridge**: Connect to `src/core/profiles.rs`
- **Backend**: Use existing `ConsensusProfile`, `BudgetProfile`, `PerformanceProfile`
- **Commands**: 
  - `get_available_profiles()` → Returns ["speed", "balanced", "quality", "consensus"]
  - `set_active_profile(profile: String)`
  - `get_profile_config(profile: String)` → Returns model configs

#### 1.2 Settings Dialog
- **Frontend**: Modal dialog with tabs
- **Bridge**: Connect to `src/core/api_keys.rs`
- **Backend**: Use existing `ApiKeyManager`
- **Commands**:
  - `get_api_key_status()` → Returns configured providers
  - `save_api_key(provider: String, key: String)`
  - `validate_api_key(provider: String, key: String)`
  - `get_settings()` → Returns all config
  - `save_settings(config: SettingsConfig)`

#### 1.3 Git/LazyGit Integration
- **Frontend**: Tab in sidebar (Files | Git toggle)
- **Bridge**: Connect to `src/desktop/git/*`
- **Backend**: Use existing `LazyGitWrapper`, `GitCommitBox`
- **Commands**:
  - `create_lazygit_terminal(path: String)` → Returns terminal ID
  - `get_git_status(path: String)` → Returns file statuses
  - `get_current_branch()` → Returns branch name
  - `commit_changes(message: String, files: Vec<String>)`

### Phase 2: Essential Features

#### 2.1 Conversation History
- **Frontend**: Panel below file tree
- **Bridge**: Connect to `src/core/database.rs`
- **Backend**: Use existing conversation storage
- **Commands**:
  - `get_conversation_history(limit: u32)` → Returns past conversations
  - `load_conversation(id: String)` → Returns full conversation
  - `delete_conversation(id: String)`
  - `search_conversations(query: String)`

#### 2.2 Multiple File Tabs
- **Frontend**: Tab bar with close buttons
- **Bridge**: Connect to file system operations
- **Backend**: Use existing file handling
- **Commands**:
  - `open_file(path: String)` → Returns content
  - `save_file(path: String, content: String)`
  - `close_file(path: String)`
  - `get_file_metadata(path: String)`

#### 2.3 File Operations Context Menu
- **Frontend**: Right-click menu in explorer
- **Bridge**: Connect to file system
- **Backend**: Standard fs operations
- **Commands**:
  - `create_file(path: String, name: String)`
  - `create_folder(path: String, name: String)`
  - `rename_item(old_path: String, new_path: String)`
  - `delete_item(path: String)`
  - `copy_item(src: String, dest: String)`

### Phase 3: Enhanced Features

#### 3.1 Markdown Rendering
- **Frontend**: Preview mode for .md files
- **Bridge**: Connect to `src/desktop/markdown_renderer.rs`
- **Backend**: Use existing markdown parser
- **Commands**:
  - `render_markdown(content: String)` → Returns HTML
  - `export_markdown_to_pdf(content: String, path: String)`

#### 3.2 Syntax Highlighting
- **Frontend**: Monaco editor or CodeMirror
- **Bridge**: Connect to `src/analysis/syntax_highlighter.rs`
- **Backend**: Use existing Tree-sitter integration
- **Commands**:
  - `get_syntax_tokens(file_path: String)` → Returns token ranges
  - `get_language_for_file(path: String)` → Returns language ID

#### 3.3 Terminal Tabs
- **Frontend**: Tab bar above terminal
- **Bridge**: Extend existing PTY bridge
- **Backend**: Multiple PTY instances
- **Commands**:
  - `create_terminal_tab(name: String)` → Returns terminal ID
  - `switch_terminal(id: String)`
  - `close_terminal_tab(id: String)`
  - `rename_terminal_tab(id: String, name: String)`

### Phase 4: Analytics & Intelligence

#### 4.1 Real Analytics Dashboard
- **Frontend**: Rich charts and graphs
- **Bridge**: Connect to `src/core/analytics.rs`
- **Backend**: Use existing AnalyticsEngine
- **Commands**:
  - `get_analytics_summary()` → Returns metrics
  - `get_cost_breakdown()` → Returns cost by model/stage
  - `get_performance_trends()` → Returns time series
  - `export_analytics(format: String)`

#### 4.2 Repository Intelligence
- **Frontend**: Code analysis panel
- **Bridge**: Connect to `src/analysis/*`
- **Backend**: Use existing analyzers
- **Commands**:
  - `analyze_repository(path: String)` → Returns insights
  - `get_dependency_graph()`
  - `find_symbols(query: String)`
  - `get_code_quality_metrics()`

### Phase 5: UI Polish & UX

#### 5.1 Welcome Dialog
- **Frontend**: First-run configuration
- **Bridge**: Connect to settings
- **Backend**: Persist preferences
- **Commands**:
  - `get_first_run_status()` → Returns bool
  - `complete_onboarding(config: OnboardingConfig)`

#### 5.2 AI Operations Confirmation
- **Frontend**: Modal for file operations
- **Bridge**: Connect to AI operation parser
- **Backend**: Use existing safety checks
- **Commands**:
  - `preview_operations(query: String)` → Returns planned ops
  - `confirm_operations(ops: Vec<Operation>)`
  - `cancel_operations()`

#### 5.3 Progress Indicators
- **Frontend**: Animated stage progress
- **Bridge**: Stream progress events
- **Backend**: Use existing callbacks
- **Commands**:
  - Already implemented via streaming callbacks
  - Add token counting display
  - Add ETA calculation

#### 5.4 Cost Tracking Display
- **Frontend**: Real-time cost counter
- **Bridge**: Connect to cost tracking
- **Backend**: Use existing cost calculation
- **Commands**:
  - `get_current_session_cost()` → Returns running total
  - `get_cost_by_model()` → Returns breakdown
  - `set_cost_limit(limit: f64)`

### Phase 6: Advanced Features

#### 6.1 Branch Selector
- **Frontend**: Dropdown in status bar
- **Bridge**: Connect to git
- **Backend**: Use git2 library
- **Commands**:
  - `get_branches()` → Returns list
  - `switch_branch(name: String)`
  - `create_branch(name: String)`

#### 6.2 Resizable Panels
- **Frontend**: Draggable dividers
- **Backend**: Persist layout preferences
- **Commands**:
  - `save_layout(config: LayoutConfig)`
  - `get_saved_layout()` → Returns config

#### 6.3 Theme System
- **Frontend**: Light/dark/custom themes
- **Backend**: Theme definitions
- **Commands**:
  - `get_available_themes()` → Returns list
  - `set_theme(theme_id: String)`
  - `get_theme_config(theme_id: String)`

#### 6.4 Keyboard Shortcuts
- **Frontend**: Keybinding system
- **Backend**: Configurable shortcuts
- **Commands**:
  - `get_keybindings()` → Returns map
  - `set_keybinding(action: String, keys: String)`
  - `reset_keybindings()`

## Implementation Order & Dependencies

### Dependency Graph
```
Foundation (Complete):
├── Database Connection ✅
├── Consensus Engine ✅
└── Terminal PTY ✅

Critical Path (Priority 1):
├── Profile Selector
│   └── Requires: profiles.rs bridge
├── Settings Dialog
│   └── Requires: api_keys.rs bridge
└── Git Integration
    └── Requires: LazyGit terminal bridge

Essential Path (Priority 2):
├── Conversation History
│   └── Requires: Database queries
├── Multiple Tabs
│   └── Requires: File system bridge
└── File Operations
    └── Requires: fs operations

Enhancement Path (Priority 3):
├── Markdown Rendering
│   └── Requires: markdown_renderer.rs
├── Syntax Highlighting
│   └── Requires: Tree-sitter bridge
└── Terminal Tabs
    └── Requires: PTY manager

Analytics Path (Priority 4):
├── Real Analytics
│   └── Requires: AnalyticsEngine bridge
└── Repository Intelligence
    └── Requires: Analysis modules

Polish Path (Priority 5):
├── Welcome Dialog
├── AI Operations Confirmation
├── Progress Indicators
└── Cost Tracking

Advanced Path (Priority 6):
├── Branch Selector
├── Resizable Panels
├── Theme System
└── Keyboard Shortcuts
```

## Bridge Module Organization

```rust
src-tauri/src/
├── bridge/
│   ├── mod.rs           // Main bridge module
│   ├── profiles.rs      // Profile management bridge
│   ├── settings.rs      // Settings & API keys bridge
│   ├── git.rs          // Git & LazyGit bridge
│   ├── conversation.rs  // History bridge
│   ├── filesystem.rs    // File operations bridge
│   ├── markdown.rs      // Markdown rendering bridge
│   ├── syntax.rs        // Syntax highlighting bridge
│   ├── terminal.rs      // Terminal management bridge
│   ├── analytics.rs     // Analytics bridge
│   ├── repository.rs    // Repository intelligence bridge
│   └── ui.rs           // UI state management bridge
└── main.rs             // Register all command handlers
```

## Frontend Component Structure

```
frontend/
├── index.html          // Main layout
├── css/
│   ├── main.css       // Core styles
│   ├── themes.css     // Theme definitions
│   └── components.css  // Component styles
├── js/
│   ├── main.js        // App initialization
│   ├── consensus.js   // Consensus handling
│   ├── editor.js      // Editor management
│   ├── terminal.js    // Terminal handling
│   ├── git.js         // Git integration
│   ├── settings.js    // Settings dialog
│   ├── analytics.js   // Analytics display
│   └── shortcuts.js   // Keyboard handling
└── components/
    ├── ProfileSelector.js
    ├── SettingsDialog.js
    ├── ConversationHistory.js
    ├── FileExplorer.js
    ├── GitPanel.js
    ├── MarkdownPreview.js
    └── AnalyticsDashboard.js
```

## Testing Requirements

### Integration Tests
1. **Profile Switching**: Verify all 4 profiles work
2. **API Key Management**: Test save/load/validate
3. **Git Operations**: Test LazyGit terminal creation
4. **Conversation History**: Test load/save/search
5. **File Operations**: Test CRUD operations
6. **Consensus Pipeline**: Test all 4 stages
7. **Analytics Accuracy**: Verify real data display
8. **Cost Tracking**: Verify accurate calculation

### Performance Requirements
- App startup: < 500ms
- File open: < 100ms
- Consensus start: < 200ms
- Tab switch: < 50ms
- Terminal creation: < 300ms
- Analytics load: < 500ms

### Compatibility Requirements
- All existing keyboard shortcuts must work
- All existing file formats must be supported
- Database migration must preserve all data
- Settings must migrate from Dioxus config

## Success Criteria

The migration is complete when:
1. ✅ ALL features from Dioxus version are working
2. ✅ Performance meets or exceeds Dioxus version
3. ✅ Zero data loss during migration
4. ✅ All existing backend code is reused
5. ✅ User cannot tell it's not the Dioxus version
6. ✅ All tests pass
7. ✅ Memory usage is equal or better
8. ✅ Startup time is equal or better

## Risk Mitigation

### Identified Risks
1. **Monaco Editor Integration**: May need fallback to CodeMirror
2. **LazyGit Terminal**: May need custom PTY handling
3. **Markdown Rendering**: May need WASM module
4. **Performance**: Web rendering may be slower than native

### Mitigation Strategies
1. **Progressive Enhancement**: Ship working features incrementally
2. **Feature Flags**: Allow toggling between implementations
3. **Fallback Options**: Have simpler alternatives ready
4. **Performance Monitoring**: Track metrics from day 1

## Timeline Estimate

- **Week 1**: Critical features (Profiles, Settings, Git)
- **Week 2**: Essential features (History, Tabs, File Ops)
- **Week 3**: Enhanced features (Markdown, Syntax, Terminals)
- **Week 4**: Analytics & Intelligence
- **Week 5**: UI Polish & UX improvements
- **Week 6**: Testing, optimization, and bug fixes

Total: **6 weeks for 100% feature parity**

## Next Immediate Actions

1. Create bridge modules for profiles and settings
2. Implement ProfileSelector component in frontend
3. Create SettingsDialog with API key management
4. Bridge LazyGit terminal creation
5. Test each feature against Dioxus version

This plan ensures we achieve 100% feature parity while maximizing code reuse and maintaining the exact same user experience as the Dioxus version.