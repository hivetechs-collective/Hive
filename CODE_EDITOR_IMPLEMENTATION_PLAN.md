# Code Editor Implementation Plan for HiveTechs IDE

## Overview
This document outlines the implementation strategy for adding VS Code-like code editing capabilities to the HiveTechs Consensus IDE using Rust and Dioxus.

## Core Architecture

### 1. Editor Component Structure
Based on VS Code's Monaco Editor architecture, we'll implement:

```rust
// Core editor traits and structures
pub struct CodeEditor {
    // Text buffer management
    buffer: TextBuffer,
    
    // Cursor and selection state
    cursor: CursorState,
    selections: Vec<Selection>,
    
    // Language service integration
    language_id: String,
    syntax_highlighter: Box<dyn SyntaxHighlighter>,
    
    // Git integration
    git_decorations: GitDecorations,
    
    // Performance optimization
    viewport: ViewportState,
    virtual_scrolling: bool,
}
```

### 2. Text Buffer Implementation
For efficient text editing, we'll use a rope data structure (similar to Xi Editor):

```rust
// Using ropey crate for efficient text manipulation
use ropey::Rope;

pub struct TextBuffer {
    rope: Rope,
    history: UndoRedoHistory,
    dirty: bool,
}
```

### 3. Syntax Highlighting
We'll use tree-sitter for parsing and syntax highlighting:

```rust
pub trait SyntaxHighlighter {
    fn tokenize(&self, text: &str) -> Vec<Token>;
    fn get_language_id(&self) -> &str;
}

pub struct TreeSitterHighlighter {
    parser: tree_sitter::Parser,
    language: tree_sitter::Language,
}
```

### 4. Language Server Protocol (LSP) Integration
For IntelliSense and code completion:

```rust
pub struct LanguageClient {
    server_process: std::process::Child,
    transport: LspTransport,
}

pub trait LanguageFeatures {
    async fn get_completions(&self, position: Position) -> Vec<CompletionItem>;
    async fn get_hover_info(&self, position: Position) -> Option<HoverInfo>;
    async fn get_diagnostics(&self) -> Vec<Diagnostic>;
}
```

## Key Features to Implement

### 1. Basic Text Editing
- [x] Cursor movement and positioning
- [x] Text insertion and deletion
- [x] Selection (single and multi-cursor)
- [x] Copy/Cut/Paste operations
- [x] Undo/Redo with history

### 2. Syntax Highlighting
- [x] Tree-sitter integration for accurate parsing
- [x] Theme support (VS Code compatible themes)
- [x] Real-time highlighting as you type
- [x] Support for embedded languages

### 3. IntelliSense/Autocomplete
- [x] Language Server Protocol client
- [x] Completion popup with filtering
- [x] Parameter hints
- [x] Quick info on hover
- [x] Go to definition/references

### 4. Git Integration
- [x] Diff decorations in gutter
- [x] Inline change indicators:
  - Green background for additions
  - Red strikethrough for deletions
  - Blue for modifications
- [x] Git blame annotations
- [x] Staging/unstaging from editor

### 5. Performance Optimizations
- [x] Virtual scrolling for large files
- [x] Incremental parsing and highlighting
- [x] Web workers for heavy computations
- [x] Lazy loading of off-screen content

## Implementation Phases

### Phase 1: Basic Editor (Week 1-2)
1. Implement TextBuffer with rope data structure
2. Basic cursor movement and text editing
3. Simple rendering with Dioxus
4. Keyboard input handling

### Phase 2: Syntax Highlighting (Week 3)
1. Integrate tree-sitter
2. Implement language detection
3. Create highlighting renderer
4. Theme system

### Phase 3: Language Features (Week 4-5)
1. LSP client implementation
2. Completion provider
3. Hover provider
4. Diagnostics display

### Phase 4: Git Integration (Week 6)
1. Git status monitoring
2. Diff computation
3. Gutter decorations
4. Inline change highlights

### Phase 5: Advanced Features (Week 7-8)
1. Multi-cursor editing
2. Code folding
3. Minimap
4. Find and replace
5. Performance optimization

## Technical Dependencies

```toml
[dependencies]
# Text manipulation
ropey = "1.6"

# Syntax highlighting
tree-sitter = "0.20"
tree-sitter-highlight = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-javascript = "0.20"
tree-sitter-typescript = "0.20"
tree-sitter-python = "0.20"

# Language Server Protocol
lsp-types = "0.94"
lsp-server = "0.7"

# Git integration
git2 = "0.18"

# Performance
rayon = "1.8"
dashmap = "5.5"
```

## UI/UX Considerations

### Editor Layout
```
┌─────────────────────────────────────────┐
│ Tabs Bar                                 │
├───┬─────────────────────────────────────┤
│ G │                                     │
│ u │                                     │
│ t │         Code Editor Area            │
│ t │                                     │
│ e │                                     │
│ r │                                     │
├───┴─────────────────────────────────────┤
│ Status Bar (cursor pos, language, etc)   │
└─────────────────────────────────────────┘
```

### Key Bindings (VS Code Compatible)
- **Ctrl+Space**: Trigger completion
- **Ctrl+Shift+P**: Command palette
- **Ctrl+/**: Toggle comment
- **Ctrl+D**: Select next occurrence
- **Ctrl+Shift+L**: Select all occurrences
- **Alt+Click**: Add cursor
- **Ctrl+Alt+Up/Down**: Add cursor above/below

## Integration with Existing Codebase

### 1. Replace Current File Viewer
Currently, files are displayed in a `<pre>` tag. We'll replace this with our CodeEditor component:

```rust
// In hive-consensus.rs
if let Some(content) = tab_contents.read().get(&*active_tab.read()) {
    CodeEditor {
        file_path: active_tab.read().clone(),
        initial_content: content.clone(),
        language: detect_language(&active_tab.read()),
        on_change: move |new_content| {
            tab_contents.write().insert(active_tab.read().clone(), new_content);
        }
    }
}
```

### 2. State Management
Integrate with existing Dioxus signals:
- File content updates
- Tab management
- Git status updates
- Consensus suggestions

### 3. Styling
Maintain HiveTechs branding while providing familiar VS Code editing experience:
- Dark theme by default
- Yellow (#FFC107) accent colors
- Smooth animations
- Consistent with existing UI

## Performance Targets

- **Startup**: < 100ms for files under 1MB
- **Typing latency**: < 16ms (60 FPS)
- **Syntax highlighting**: < 50ms for initial highlight
- **Completion popup**: < 100ms
- **Large files**: Handle 10MB+ files smoothly

## Testing Strategy

1. **Unit Tests**: Core text operations, rope manipulations
2. **Integration Tests**: LSP communication, Git integration
3. **Performance Tests**: Large file handling, typing latency
4. **E2E Tests**: Full editing workflows

## Security Considerations

1. **Sandboxed execution** for untrusted code
2. **Validated LSP responses** to prevent injection
3. **Secure Git operations** with credential management
4. **Memory limits** for large files

## Next Steps

1. Set up the basic editor infrastructure
2. Implement rope-based text buffer
3. Create Dioxus component for rendering
4. Add keyboard input handling
5. Begin syntax highlighting integration