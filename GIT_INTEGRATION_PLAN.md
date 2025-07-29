# VS Code-Style Git Integration Implementation Plan

## Overview
This document outlines the complete implementation plan for VS Code-style git integration in our Rust/Dioxus IDE, including diff viewer, change highlighting, and all git operations.

## Current State
- ✅ Basic git branch detection
- ✅ Source Control panel with file list
- ✅ File status indicators (M/A/D/R/U)
- ❌ Diff viewer for changes
- ❌ Code highlighting (red/green)
- ❌ Git operations (commit, push, pull, etc.)
- ❌ Inline/side-by-side diff views
- ❌ Stage/unstage functionality

## VS Code Git Architecture Analysis

### Key Components in VS Code:
1. **Git Extension** (`extensions/git/`)
   - Git provider implementation
   - Git operations (commit, push, pull, etc.)
   - File status tracking
   
2. **SCM Workbench** (`src/vs/workbench/contrib/scm/`)
   - UI components for source control
   - SCM view pane
   - Input box for commit messages
   
3. **Diff Editor** (`src/vs/editor/browser/widget/diffEditorWidget.ts`)
   - Side-by-side and inline diff views
   - Change highlighting with red/green
   - Gutter indicators for add/remove
   - Stage/revert buttons in gutter

### VS Code Features to Implement:

#### 1. Diff Viewer
- **Side-by-side view**: Original file on left, modified on right
- **Inline view**: Changes shown in single column with +/- indicators
- **Syntax highlighting**: Preserved in both views
- **Change highlighting**: 
  - Green background for additions
  - Red background for deletions
  - Light blue for modifications
- **Line numbers**: Show original and new line numbers
- **Fold unchanged regions**: Collapse large unchanged sections

#### 2. Git Operations Toolbar
- **Refresh** - Update git status
- **Commit** - Stage all and commit
- **Pull** - Pull from remote
- **Push** - Push to remote
- **More Actions** (...) menu:
  - Push To...
  - Pull From...
  - Fetch
  - Checkout to...
  - Create Branch...
  - Merge Branch...
  - Rebase Branch...
  - Create Tag...
  - Stash / Pop Stash
  - Discard All Changes

#### 3. File Actions
- **Open File** - Open in editor
- **Stage Changes** - Stage individual file
- **Unstage Changes** - Unstage file
- **Discard Changes** - Revert file
- **Open Changes** - Open diff view

#### 4. Inline Actions in Diff View
- **Stage/Unstage** buttons in gutter
- **Revert** buttons for individual changes
- **Navigation**: Next/Previous change buttons

## Implementation Plan

### Phase 1: Diff Algorithm & Data Structures
```rust
// Core diff types
pub struct DiffResult {
    pub original_lines: Vec<String>,
    pub modified_lines: Vec<String>,
    pub hunks: Vec<DiffHunk>,
}

pub struct DiffHunk {
    pub original_start: usize,
    pub original_count: usize,
    pub modified_start: usize,
    pub modified_count: usize,
    pub lines: Vec<DiffLine>,
}

pub enum DiffLineType {
    Added,
    Deleted,
    Modified,
    Unchanged,
}

pub struct DiffLine {
    pub line_type: DiffLineType,
    pub original_line_number: Option<usize>,
    pub modified_line_number: Option<usize>,
    pub content: String,
}
```

### Phase 2: Diff Viewer Component
```rust
// Diff viewer component
#[component]
pub fn DiffViewer(
    file_path: String,
    diff_mode: DiffViewMode, // SideBySide or Inline
) -> Element {
    // Load original and modified content
    // Apply diff algorithm
    // Render with syntax highlighting
}

pub enum DiffViewMode {
    SideBySide,
    Inline,
}
```

### Phase 3: Git Operations Service
```rust
pub struct GitOperations {
    repo: git2::Repository,
}

impl GitOperations {
    pub async fn stage_file(&self, path: &Path) -> Result<()>;
    pub async fn unstage_file(&self, path: &Path) -> Result<()>;
    pub async fn discard_changes(&self, path: &Path) -> Result<()>;
    pub async fn commit(&self, message: &str) -> Result<()>;
    pub async fn push(&self, remote: &str, branch: &str) -> Result<()>;
    pub async fn pull(&self, remote: &str, branch: &str) -> Result<()>;
    pub async fn fetch(&self, remote: &str) -> Result<()>;
    pub async fn create_branch(&self, name: &str) -> Result<()>;
    pub async fn checkout(&self, branch: &str) -> Result<()>;
    pub async fn merge(&self, branch: &str) -> Result<()>;
    pub async fn stash(&self) -> Result<()>;
    pub async fn stash_pop(&self) -> Result<()>;
}
```

### Phase 4: Enhanced Source Control Panel
- Replace simple file list with actionable items
- Add toolbar with git operations
- Implement staging area view
- Add commit message input with history

### Phase 5: Integration & Polish
- Connect diff viewer to file clicks
- Add keyboard shortcuts
- Implement progress indicators for operations
- Add error handling and notifications
- Persist view preferences

## Technical Implementation Details

### 1. Diff Algorithm
Use Myers' diff algorithm (same as VS Code):
- Implement O(ND) difference algorithm
- Support for moved code detection
- Handle large files with streaming

### 2. Syntax Highlighting in Diff
- Preserve original syntax highlighting
- Overlay diff highlighting (red/green)
- Use CSS layers for proper rendering

### 3. Git Operations
- Use git2 crate for all operations
- Implement async wrappers for long operations
- Add progress callbacks for push/pull

### 4. UI Components
```rust
// Main components structure
- GitPanel
  ├── GitToolbar
  ├── StagedChanges
  │   └── FileItem (with actions)
  ├── Changes
  │   └── FileItem (with actions)
  └── CommitSection
      ├── CommitInput
      └── CommitButton

- DiffEditor
  ├── DiffToolbar (view mode toggle)
  ├── DiffContent
  │   ├── OriginalPane
  │   ├── GutterActions
  │   └── ModifiedPane
  └── DiffStatusBar
```

### 5. Styling
```css
/* Diff highlighting colors (VS Code defaults) */
.diff-added-line {
    background-color: rgba(155, 185, 85, 0.2);
}

.diff-removed-line {
    background-color: rgba(255, 97, 136, 0.2);
}

.diff-added-char {
    background-color: rgba(155, 185, 85, 0.5);
}

.diff-removed-char {
    background-color: rgba(255, 97, 136, 0.5);
}

.diff-modified-line {
    background-color: rgba(97, 175, 239, 0.2);
}
```

## Migration Strategy

1. **Keep existing code** - Don't break current functionality
2. **Incremental updates** - Add features one by one
3. **Feature flags** - Allow toggling between old/new UI
4. **Testing** - Comprehensive tests for each phase

## Testing Requirements

1. **Unit tests** for diff algorithm
2. **Integration tests** for git operations
3. **UI tests** for component interactions
4. **Performance tests** for large files
5. **Cross-platform tests** (Windows, macOS, Linux)

## Performance Considerations

1. **Lazy loading** - Only load diffs when needed
2. **Virtual scrolling** - For large diffs
3. **Debouncing** - For file watchers
4. **Caching** - Cache diff results
5. **Background processing** - Run diffs in separate thread

## Security Considerations

1. **Credential handling** - Use system credential manager
2. **SSH key support** - For push/pull operations
3. **HTTPS authentication** - Token-based auth
4. **Input validation** - Sanitize commit messages
5. **Path validation** - Prevent directory traversal

## Timeline Estimate

- Phase 1 (Diff Algorithm): 2-3 days
- Phase 2 (Diff Viewer): 3-4 days  
- Phase 3 (Git Operations): 2-3 days
- Phase 4 (Enhanced Panel): 2-3 days
- Phase 5 (Integration): 2-3 days

**Total: 11-16 days**

## Next Steps

1. Start with Phase 1 - Implement diff algorithm
2. Create basic diff viewer with syntax highlighting
3. Test with various file types and sizes
4. Iterate based on user feedback

This plan provides a complete roadmap for implementing VS Code-style git integration with all the features users expect from a modern IDE.