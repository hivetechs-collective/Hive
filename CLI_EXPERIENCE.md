# HiveTechs Consensus CLI Experience

## Claude Code-Inspired Interface

This document describes the complete CLI experience that replicates and enhances Claude Code's sophisticated interface for HiveTechs Consensus.

## Launch Experience

### 1. Simple Command Launch
```bash
$ hive
```

Shows the Claude Code-style welcome banner:

```
╭───────────────────────────────────────────────────╮
│ ✻ Welcome to HiveTechs Consensus!                 │
│                                                   │
│   /help for help, /status for your current setup  │
│                                                   │
│   cwd: /Users/dev/project                         │
╰───────────────────────────────────────────────────╯

 What's new:
  • Released [Enterprise Hooks](https://docs.hivetechs.com/
  hooks). Deterministic control over AI behavior
  • Temporal context for web search - always knows today's date
  • Repository intelligence with ML-powered analysis
  • 10-40x performance improvements over TypeScript version
  • Planning mode for strategic development workflows
```

### 2. Interactive Mode Launch
```bash
$ hive interactive
```

Launches the persistent TUI interface with:
- Scrolling message area
- Persistent input box at bottom
- Status line with context information

## Interface Architecture

### Advanced TUI Mode (Default)

```
┌─────────────────────────────────────────────────────┐
│ Welcome to HiveTechs Consensus!                     │
│                                                     │
│ What's new:                                         │
│ • Enterprise Hooks - Deterministic control         │
│ • Temporal context for web search                   │
│ • Repository intelligence with ML analysis          │
│ • 10-40x performance improvements                   │
│                                                     │
│ > ask "What does this code do?"                     │
│                                                     │
│ 🤔 Processing your question...                     │
│ 🧠 Running 4-stage consensus pipeline...           │
│                                                     │
│ Generator → ████████ 100% (claude-3-5-sonnet)     │
│ Refiner   → ████████ 100% (gpt-4-turbo)           │
│ Validator → ████████ 100% (claude-3-opus)         │
│ Curator   → ████████ 100% (gpt-4o)                │
│                                                     │
│ ✨ Consensus Response:                             │
│ This code implements a CLI application...           │
├─────────────────────────────────────────────────────┤
│ ╭─────────────────────────────────────────────────╮ │
│ │ > Try "ask <question>" or "analyze ."           │ │
│ ╰─────────────────────────────────────────────────╯ │
├─────────────────────────────────────────────────────┤
│ ⏵⏵ auto-accept edits on    Context left until     │
│ (shift+tab to cycle)       auto-compact: 17%       │
└─────────────────────────────────────────────────────┘
```

### Simple CLI Mode (Fallback)

For terminals that don't support advanced TUI:

```bash
╭──────────────────────────────────────────────────────╮
│ > Try "ask <question>" or "analyze ."                 │
╰──────────────────────────────────────────────────────╯
  ? for shortcuts  
```

User types commands and responses appear above the input box.

## Supported Commands

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `ask <question>` | AI consensus question | `ask "How to optimize this Rust code?"` |
| `analyze <path>` | Repository analysis | `analyze .` or `analyze src/main.rs` |
| `plan <goal>` | Development planning | `plan "Add user authentication"` |
| `improve <file>` | Code improvement | `improve src/main.rs` |
| `hooks list` | Show enterprise hooks | `hooks list` |
| `memory search <query>` | Search history | `memory search "rust performance"` |

### Special Commands

| Command | Description |
|---------|-------------|
| `/help` or `help` | Show help information |
| `/status` or `status` | System status check |
| `/exit` or `exit` or `quit` | Exit interactive mode |

## Real-Time Features

### 1. Consensus Pipeline Visualization

When processing queries, shows real-time progress:

```
🤔 Processing your question...
🧠 Running 4-stage consensus pipeline...

Generator → ██████░░ 75% (claude-3-5-sonnet)
Refiner   → ██░░░░░░ 25% (gpt-4-turbo)
Validator → ░░░░░░░░  0% (pending)
Curator   → ░░░░░░░░  0% (pending)
```

### 2. Status Line Updates

Bottom status line shows:
- Auto-accept edit mode (toggleable with Shift+Tab)
- Context remaining percentage
- Current operation mode

### 3. Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Submit command |
| `Ctrl+C` | Exit application |
| `Shift+Tab` | Toggle auto-accept edits |
| `↑/↓` | Scroll through message history |
| `←/→` | Navigate cursor in input |
| `?` | Show shortcuts |

## Command Examples

### Ask Command
```
> ask "What's the latest in Rust async programming?"

🤔 Processing your question...
🧠 Running 4-stage consensus pipeline...

Generator → ████████ 100% (claude-3-5-sonnet)
Refiner   → ████████ 100% (gpt-4-turbo)
Validator → ████████ 100% (claude-3-opus)
Curator   → ████████ 100% (gpt-4o)

✨ Consensus Response:
IMPORTANT: Today's date is Tuesday, July 2, 2024. Here are the latest developments in Rust async programming:

1. **Async Closures** - Recently stabilized in Rust 1.75+
2. **Async Traits** - Now stable with return position impl trait
3. **Performance Improvements** - Tokio 1.35+ has significant optimizations

[Detailed technical explanation...]
```

### Analyze Command
```
> analyze .

🔍 Analyzing: .
📊 Repository Intelligence:
  • Architecture: Rust CLI Application with TUI
  • Quality Score: 8.5/10
  • Files Analyzed: 47
  • Technical Debt: Low
  • Security Issues: None found
  • Performance Hotspots: 2 identified
```

### Plan Command
```
> plan "Add enterprise SSO integration"

📋 Creating development plan for: Add enterprise SSO integration
🧠 Analyzing requirements and dependencies...

✅ Plan created with 8 tasks
⏱️  Estimated completion: 4-5 days
⚠️  2 risks identified
📊 Dependencies: OAuth2 library, user management system

📝 Use 'execute plan.json' to begin implementation
```

## Configuration

### Environment Variables

```bash
# Force simple CLI mode
export HIVE_SIMPLE_CLI=1

# Enable debug mode
export HIVE_DEBUG=1

# Set timezone for temporal context
export HIVE_TIMEZONE="America/New_York"
```

### Config File

```toml
# ~/.hive/config.toml

[interface]
mode = "interactive"  # "interactive", "simple", "tui"
auto_accept_edits = true
show_progress_bars = true
enable_shortcuts = true

[interface.tui]
theme = "dark"  # "dark", "light", "solarized"
animation_speed = "fast"  # "slow", "normal", "fast"
terminal_title = "HiveTechs Consensus"

[temporal_context]
enabled = true
timezone = "UTC"
business_calendar = "NYSE"
```

## Technical Implementation

### TUI Architecture

1. **Ratatui Framework**: Modern terminal UI framework
2. **Crossterm Backend**: Cross-platform terminal handling
3. **Event Loop**: Async event processing for real-time updates
4. **State Management**: Clean separation of UI and business logic

### Message System

```rust
pub struct Message {
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub enum MessageType {
    Welcome,
    UserInput,
    SystemResponse,
    ConsensusProgress,
    Error,
    Help,
    Status,
}
```

### Progressive Enhancement

1. **Detection**: Automatically detects terminal capabilities
2. **Fallback**: Gracefully falls back to simple CLI if TUI fails
3. **Responsive**: Adapts to different terminal sizes
4. **Accessible**: Works with screen readers and accessibility tools

## Performance

- **Startup Time**: <50ms (40x faster than TypeScript version)
- **Memory Usage**: ~25MB (vs 180MB TypeScript)
- **Response Time**: Real-time consensus progress updates
- **Terminal Rendering**: 60+ FPS smooth scrolling

## Accessibility

- Screen reader compatible
- High contrast mode support
- Keyboard-only navigation
- Customizable themes and fonts
- Reduced motion options

## Comparison with Claude Code

| Feature | Claude Code | HiveTechs Consensus | Advantage |
|---------|-------------|-------------------|-----------|
| **Welcome Banner** | ✅ | ✅ Enhanced | **Customized for consensus** |
| **Persistent Input** | ✅ | ✅ | **Identical experience** |
| **Progress Display** | ✅ | ✅ 4-stage pipeline | **More detailed progress** |
| **Status Line** | ✅ | ✅ Extended | **More context information** |
| **Auto-accept Toggle** | ✅ | ✅ | **Same functionality** |
| **Command History** | ✅ | ✅ | **Enhanced with memory search** |
| **Shortcuts** | ✅ | ✅ Extended | **More shortcuts available** |
| **Theming** | ✅ | ✅ | **Customizable themes** |

This implementation ensures HiveTechs Consensus provides the same polished, professional CLI experience as Claude Code while adding unique features like consensus pipeline visualization, enterprise hooks integration, and temporal context awareness.