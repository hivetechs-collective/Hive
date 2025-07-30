# VS Code Git Integration Complete Implementation Plan for Hive Consensus

## Executive Summary

This plan covers the complete implementation of VS Code-style git integration for our Rust/Dioxus IDE, including all status bar features, source control panel functionality, file decorations, and interactive elements. The implementation will address the issues identified (clickable status bar not working, push button freezing) and add all missing features.

## 1. Status Bar Git Features

### 1.1 Branch Indicator Section
- **Current Implementation**: Shows branch name with ahead/behind counts
- **Missing Features**:
  - Click handler to open branch menu
  - Hover tooltip with full repository info
  - Branch operations menu:
    - Create new branch
    - Checkout branch (with search)
    - Checkout detached
    - Checkout tag
    - Create tag
  - Sync status spinner during operations
  - Problems indicator integration

### 1.2 Sync Status Section
- **Current**: Shows ahead/behind arrows (↓2 ↑0)
- **Missing**:
  - Click to sync (pull then push)
  - Publish branch button for unpublished branches
  - Auto-fetch configuration
  - Sync in progress indicator
  - Conflict status indicator

### 1.3 Repository Selector
- **Missing Completely**:
  - Multi-repository support
  - Repository switcher in status bar
  - Repository status indicators

## 2. Source Control Panel Enhancements

### 2.1 Toolbar Actions
- **Current**: Basic push, pull, fetch, sync buttons
- **Missing**:
  - Refresh button (with spinner during refresh)
  - Clone repository button
  - More actions menu (...):
    - Pull (Rebase)
    - Fetch (Prune)
    - Commit (Amend)
    - Commit (Signed)
    - Push (Force)
    - Stash/Stash Pop
    - Apply Latest Stash

### 2.2 Source Control View Features
- **Missing**:
  - Commit message box improvements:
    - Character counter
    - Previous commit messages dropdown
    - Commit message templates
  - Changes section enhancements:
    - Group by folder option
    - Collapse/expand all
    - Stage/unstage all in folder
    - Context menu on files:
      - Open File
      - Open Changes
      - Stage/Unstage Changes
      - Discard Changes
      - Add to .gitignore

### 2.3 Diff Viewer Improvements
- **Current**: Basic side-by-side diff
- **Missing**:
  - Inline diff actions:
    - Stage/unstage hunks
    - Revert hunks
    - Navigate between changes
  - 3-way merge editor
  - Accessibility features
  - Better syntax highlighting in diffs

## 3. File Explorer Git Integration

### 3.1 File Decorations
- **Missing Completely**:
  - Status badges (M, A, D, U, C, !)
  - Color coding for file status
  - Ignored file dimming
  - Incoming changes indicators (↓M, ↓A, etc.)

### 3.2 Context Menu Git Actions
- **Missing**:
  - Stage/Unstage file
  - Discard changes
  - View history
  - Compare with HEAD
  - Add to .gitignore

## 4. Advanced Git Features

### 4.1 Git Graph/History View
- **Missing Completely**:
  - Commit graph visualization
  - Branch visualization
  - Search in history
  - Cherry-pick from history
  - Revert commits

### 4.2 Stash Management
- **Missing Completely**:
  - Stash list view
  - Apply/pop stash
  - Drop stash
  - View stash contents

### 4.3 Remote Management
- **Missing**:
  - Add/remove remotes
  - Fetch from specific remote
  - Push to specific remote
  - Set upstream branch

## 5. UI/UX Improvements

### 5.1 Icons (VS Code uses Codicon font)
- **Need to implement**:
  - git-branch
  - git-commit
  - git-compare
  - git-merge
  - git-pull-request
  - cloud-upload
  - sync
  - sync-spin
  - refresh
  - check
  - x (close)
  - warning
  - info

### 5.2 Keyboard Shortcuts
- **Missing**:
  - Ctrl/Cmd+Shift+G: Focus source control
  - Ctrl/Cmd+Enter: Commit
  - Ctrl/Cmd+Shift+Enter: Commit all
  - Alt+S: Stage/Unstage file

### 5.3 Settings/Configuration
- **Missing**:
  - git.enableSmartCommit
  - git.confirmSync
  - git.autofetch
  - git.autofetchPeriod
  - git.branchProtection
  - git.defaultCloneDirectory
  - git.enableCommitSigning

## 6. Bug Fixes Required

### 6.1 Status Bar Click Handler
- Fix the current non-functional click handler
- Implement proper menu system
- Add keyboard navigation support

### 6.2 Push Button Freeze
- Fix async operation handling
- Add proper error handling
- Implement progress indicators
- Add operation cancellation

### 6.3 Real-time Updates
- Ensure all UI components update when git state changes
- Fix race conditions in state updates
- Implement proper debouncing

## 7. Implementation Architecture

### 7.1 Core Components Needed

```
// New modules to create
src/desktop/git/
├── branch_menu.rs         // Branch selection menu
├── commit_input.rs        // Enhanced commit message input
├── decorations.rs         // File decoration provider
├── graph.rs              // Commit graph visualization
├── history.rs            // Commit history view
├── icons.rs              // Git icon definitions
├── merge_editor.rs       // 3-way merge conflict editor
├── remote_manager.rs     // Remote repository management
├── settings.rs           // Git configuration
├── shortcuts.rs          // Keyboard shortcut handlers
├── stash_manager.rs      // Stash operations
└── status_menu.rs        // Status bar menu implementation
```

### 7.2 State Management Enhancements
- Implement proper event bus for git state changes
- Add operation queue for sequential git operations
- Implement progress tracking for long operations
- Add cancellation tokens for operations

### 7.3 UI Component Structure
- Create reusable menu component
- Implement tooltip system
- Add progress indicator component
- Create icon component system

## 8. Implementation Phases

### Phase 1: Core Infrastructure (Week 1)
- Fix existing bugs (click handler, push freeze)
- Implement menu system
- Add icon support
- Create tooltip component

### Phase 2: Status Bar Complete (Week 2)
- Branch selection menu
- Sync operations
- Repository selector
- Problems integration

### Phase 3: Source Control Panel (Week 3)
- Enhanced toolbar
- Improved commit input
- Context menus
- File grouping

### Phase 4: File Decorations (Week 4)
- Status badges
- Color coding
- Incoming changes
- Context menu actions

### Phase 5: Advanced Features (Week 5-6)
- Commit history
- Stash management
- Remote management
- Merge editor

### Phase 6: Polish & Settings (Week 7)
- Keyboard shortcuts
- Configuration options
- Performance optimization
- Testing & bug fixes

## 9. Technical Considerations

### 9.1 Performance
- Implement virtual scrolling for large file lists
- Cache git status for performance
- Debounce file watcher events
- Use web workers for heavy operations

### 9.2 Error Handling
- Comprehensive error messages
- Retry mechanisms
- Graceful degradation
- User-friendly error notifications

### 9.3 Cross-platform Support
- Test on Windows, macOS, Linux
- Handle platform-specific git behaviors
- Ensure consistent UI across platforms

## 10. Testing Strategy
- Unit tests for all git operations
- Integration tests for UI components
- End-to-end tests for workflows
- Performance benchmarks
- User acceptance testing

## Implementation Tracking

### Status Legend
- ⬜ Not Started
- 🟨 In Progress
- ✅ Completed
- ❌ Blocked
- 🔄 In Review

### Current Status by Feature
- Status Bar Git Features: 🟨
- Source Control Panel: 🟨
- File Explorer Integration: ⬜
- Advanced Git Features: ⬜
- UI/UX Improvements: ⬜
- Bug Fixes: 🟨

This comprehensive plan will bring our git integration to full parity with VS Code, providing users with a familiar and powerful git experience in the Hive Consensus IDE.