# Terminal Integration Plan - Hive Consensus + Claude Code

## 🎯 Vision
Integrate a VS Code-style terminal into the hive-consensus GUI where Claude Code runs independently, while keeping consensus in the chat window. Bridge them via MCP servers.

## 📋 Implementation Status

### ✅ Completed (2025-07-27)

#### 1. Feature Branch Creation
- Created `feature/claude-direct-integration` to preserve Claude integration attempts
- Successfully pushed to origin

#### 2. Rolled Back Main Branch
- Reset main to commit `cecc364` (before Claude integration)
- Clean state without complex Claude SDK/CLI hybrid code

#### 3. Terminal Integration
- **Created `terminal_tabs.rs`**: VS Code-inspired multi-terminal tab management
- **Enhanced `terminal.rs`**: 
  - Added Claude Code detection on startup
  - Added `install-claude` command
  - Welcome message with instructions
- **Modified `app.rs`**: 
  - Added terminal section at bottom of IDE
  - Layout: File Explorer + Chat (top), Terminal (bottom)
- **Updated `styles/mod.rs`**: Added terminal-specific CSS

#### 4. Key Features Implemented
- Multiple terminal instances with tabs
- Tab management (create, close, switch)
- Command history (arrow up/down)
- Built-in commands: help, clear, cd, install-claude
- Auto-detection of Claude Code installation
- VS Code dark theme styling

### 🚧 In Progress
- Testing full build and GUI functionality
- Creating proper git commits

### 📋 Pending Tasks
1. **Enhance MCP server** to expose consensus and knowledge to Claude
2. **Create bridge** for curator results to Claude file operations
3. **Fix AI Helper** file manipulation issues
4. **Create R2 distribution** package

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│                  Menu Bar                       │
├─────────────────┬───────────────────────────────┤
│                 │                               │
│  File Explorer  │      Chat Interface          │
│                 │   (Consensus Engine)         │
│                 │                               │
├─────────────────┴───────────────────────────────┤
│              Terminal Tabs                      │
│         (Claude Code runs here)                 │
└─────────────────────────────────────────────────┘
```

## 🔄 Integration Strategy

### 1. **Separate Concerns**
- **Chat Window**: 4-stage consensus pipeline for analysis
- **Terminal**: Claude Code for file operations
- **MCP Bridge**: Expose Hive capabilities to Claude

### 2. **MCP Server Enhancement**
The existing MCP server at `src/integration/mcp/` already has:
- `ask_hive` tool for consensus queries
- Repository analysis capabilities
- Can be extended to:
  - Expose thematic memory to Claude
  - Share curator suggestions
  - Enable Claude to execute file operations from curator plans

### 3. **Workflow Example**
1. User asks for code generation in chat
2. Consensus pipeline analyzes and creates plan
3. Curator suggests file operations
4. MCP bridge passes suggestions to Claude
5. User can execute in terminal with Claude Code
6. Results visible in file explorer

## 🛠️ Technical Details

### Terminal Component Structure
```rust
terminal_tabs.rs
├── TerminalTabs component (manages multiple terminals)
├── TerminalTab struct (id, title, icon, working_directory)
├── Tab management functions
└── Terminal instance wrapper

terminal.rs
├── Terminal component (actual terminal emulator)
├── Command execution
├── Claude Code detection
├── Built-in commands
└── History management
```

### CSS Classes Added
- `.main-layout-with-terminal`: Flex column layout
- `.terminal-tabs`: Terminal container
- `.terminal-tab-bar`: Tab bar styling
- `.terminal-tab`: Individual tab styling
- `.terminal-content`: Terminal output area
- `.terminal-input-container`: Input area styling

## 🚀 Next Implementation Steps

### 1. MCP Server Enhancement (Priority: High)
```rust
// Add to src/integration/mcp/tools.rs
register_tool("get_curator_suggestions", ...);
register_tool("get_thematic_memory", ...);
register_tool("execute_file_plan", ...);
```

### 2. Curator → Claude Bridge (Priority: Medium)
- Modify curator to output structured plans
- Create MCP tool to retrieve latest curator suggestion
- Enable Claude to parse and execute plans

### 3. Testing & Polish
- Ensure terminal properly executes all commands
- Test Claude Code installation flow
- Verify MCP bridge functionality
- Performance optimization

## 📝 Original User Requirements

From the user's feedback:
> "I know that if I launched VS Code for example and opened a terminal within VS Code, i could install claude code in that terminal and have its cli to use claude but still have the rest of VS Code to use."

> "So maybe we roll back to where we were, save this work for the future but put it on a feature branch and then roll main back to a commit prior to adding in claude code and then diverge and create a terminal built into our hive-consensus IDE and auto install claude code in that terminal."

> "Then in one chat window users can use our working consensus and in the terminal users can use Claude Code for file manipulation and we can ask claude to read from our growing knowledge base similar to how we build mcp servers."

This implementation directly addresses these requirements by:
- ✅ Creating an embedded terminal like VS Code
- ✅ Allowing Claude Code to run in the terminal
- ✅ Keeping consensus in the chat window
- ✅ Using MCP servers as the bridge

## 🎉 Benefits of This Approach

1. **Clean Separation**: No complex integration code
2. **Full Claude Features**: All Claude Code commands work
3. **Full Consensus Power**: 4-stage pipeline unchanged
4. **Best of Both**: Analysis + Execution
5. **Familiar UX**: Like VS Code with AI