# 🚀 Terminal & AI CLI Integration - Implementation Plan

## Executive Summary
With our isolated terminal panel foundation complete, we now have the perfect architecture to build a sophisticated terminal system that integrates with AI CLI tools. This plan outlines a systematic approach to achieve our goals while maintaining complete isolation and preventing any impact on the existing application.

## Current Foundation Status ✅
- **Isolated Terminal Panel**: Fully functional, resizable, collapsible
- **Tab System**: Working with System Log as first tab
- **Console Capture**: Successfully intercepting console output
- **Complete Isolation**: Zero impact on other app components
- **Resize Behavior**: Mirrors consensus panel perfectly

## 🎯 Strategic Goals

### 1. Terminal Infrastructure Goals
- **Full xterm.js Integration**: Professional terminal emulator
- **Multiple Terminal Instances**: Each in its own tab
- **Process Management**: Spawn and manage shell processes
- **AI Tool Terminals**: Dedicated terminals for each AI CLI tool
- **Command Execution**: Full shell capabilities

### 2. AI CLI Tool Integration Goals
- **Automatic Detection**: Find installed AI tools on startup
- **Dynamic Terminal Creation**: Launch tools in dedicated terminals
- **Tool-Specific Contexts**: Each tool gets its own environment
- **Memory Service Integration**: Connect tools to our Memory Service
- **Real-time Status Monitoring**: Track tool health and status

### 3. User Experience Goals
- **Seamless Integration**: Tools launch directly in our terminal
- **Visual Feedback**: Clear indication of tool status
- **Easy Management**: Start/stop/restart tools from UI
- **Context Preservation**: Maintain terminal history
- **Professional Feel**: VS Code-like terminal experience

## 📋 Implementation Phases

### Phase 1: xterm.js Integration (Week 1)
**Goal**: Replace placeholder terminals with real xterm.js instances

#### Tasks:
1. **Install Dependencies**
   ```bash
   npm install xterm xterm-addon-fit xterm-addon-web-links
   npm install node-pty @types/node-pty
   ```

2. **Create Terminal Manager**
   ```typescript
   // src/components/terminal/TerminalManager.ts
   class TerminalManager {
     private terminals: Map<string, XTerminal>
     private ptyProcesses: Map<string, IPty>
     
     createTerminal(id: string, options?: ITerminalOptions)
     destroyTerminal(id: string)
     writeToTerminal(id: string, data: string)
     resizeTerminal(id: string, cols: number, rows: number)
   }
   ```

3. **Integrate with IsolatedTerminalPanel**
   - Replace placeholder content with xterm instances
   - Handle resize events properly
   - Implement proper cleanup on tab close

4. **System Log Enhancement**
   - Keep System Log as read-only xterm
   - Style it differently (gray background, no cursor)
   - Maintain console capture functionality

### Phase 2: Process Management (Week 1-2)
**Goal**: Spawn and manage shell processes for terminals

#### Tasks:
1. **Create Process Bridge**
   - IPC handlers for terminal operations
   - Main process spawns node-pty instances
   - Bidirectional data flow (input/output)

2. **Shell Environment Setup**
   ```typescript
   // Detect and use appropriate shell
   const shell = process.platform === 'win32' ? 'powershell.exe' : 
                 process.env.SHELL || '/bin/bash'
   ```

3. **Working Directory Management**
   - Set initial directory to current project
   - Allow directory changes via commands
   - Track current directory per terminal

### Phase 3: AI CLI Tool Detection (Week 2)
**Goal**: Automatically detect and manage AI CLI tools

#### Tasks:
1. **Enhanced Detection System**
   ```typescript
   // src/utils/enhanced-cli-detector.ts
   interface CliToolInfo {
     id: string
     name: string
     installed: boolean
     version?: string
     path?: string
     command: string
     args?: string[]
     env?: Record<string, string>
   }
   
   class EnhancedCliDetector {
     async detectAllTools(): Promise<CliToolInfo[]>
     async getToolCommand(toolId: string): Promise<string>
     async checkToolHealth(toolId: string): Promise<boolean>
   }
   ```

2. **Tool Registry**
   ```typescript
   const AI_CLI_TOOLS = [
     { id: 'claude-code', command: 'claude', name: 'Claude Code' },
     { id: 'gemini-cli', command: 'gemini', name: 'Gemini CLI' },
     { id: 'qwen-code', command: 'qwen', name: 'Qwen Code' },
     { id: 'aider', command: 'aider', name: 'Aider' },
     { id: 'continue', command: 'continue', name: 'Continue' }
   ]
   ```

3. **Dynamic Tab Creation**
   - Add tool tabs automatically when detected
   - Use tool-specific icons and colors
   - Show version in tab tooltip

### Phase 4: AI Tool Integration (Week 2-3)
**Goal**: Launch and manage AI tools in dedicated terminals

#### Tasks:
1. **Tool Launch System**
   ```typescript
   class AiToolLauncher {
     async launchTool(toolId: string, workDir: string): Promise<string> {
       // Create dedicated terminal tab
       const terminalId = `ai-tool-${toolId}`
       
       // Set up environment
       const env = {
         ...process.env,
         MEMORY_SERVICE_URL: 'http://localhost:3457',
         HIVE_INTEGRATION: 'true'
       }
       
       // Launch tool with proper arguments
       const command = await this.buildCommand(toolId)
       return this.terminalManager.createTerminal(terminalId, {
         command,
         env,
         cwd: workDir
       })
     }
   }
   ```

2. **Tool-Specific Configurations**
   - Claude Code: Add Memory Service configuration
   - Gemini: Set up API keys if needed
   - Qwen: Configure model paths
   - Custom environment variables per tool

3. **Status Monitoring**
   - Track if tool process is running
   - Monitor for crashes/exits
   - Auto-restart capability
   - Health check pings

### Phase 5: Memory Service Integration (Week 3)
**Goal**: Connect AI tools to our Memory Service

#### Tasks:
1. **Auto-Configuration**
   ```typescript
   // Automatically configure tools to use Memory Service
   async configureToolForMemory(toolId: string) {
     const configPath = await this.getToolConfigPath(toolId)
     const config = {
       memoryService: {
         enabled: true,
         url: 'http://localhost:3457',
         apiKey: await this.generateApiKey(toolId)
       }
     }
     await this.writeToolConfig(configPath, config)
   }
   ```

2. **Connection Status**
   - Show Memory Service connection in UI
   - Display activity when tool queries memory
   - Track usage statistics per tool

3. **Unified Memory Pool**
   - All tools share the same memory database
   - Cross-tool learning and context sharing
   - Conversation continuity across tools

### Phase 6: Advanced Terminal Features (Week 3-4)
**Goal**: Professional terminal experience

#### Tasks:
1. **Terminal Enhancements**
   - Split terminals (horizontal/vertical)
   - Terminal search (Ctrl+F)
   - Copy/paste support
   - Clickable links
   - Custom color themes

2. **Command Palette Integration**
   ```typescript
   // Quick actions via command palette
   registerCommands([
     { id: 'terminal.new', label: 'New Terminal' },
     { id: 'terminal.split', label: 'Split Terminal' },
     { id: 'ai.launch.claude', label: 'Launch Claude Code' },
     { id: 'ai.restart.all', label: 'Restart All AI Tools' }
   ])
   ```

3. **Keyboard Shortcuts**
   - Ctrl+Shift+` : New terminal
   - Ctrl+Tab : Next terminal
   - Ctrl+W : Close terminal
   - Ctrl+Shift+T : Reopen closed terminal

### Phase 7: UI Polish & Integration (Week 4)
**Goal**: Seamless user experience

#### Tasks:
1. **Visual Refinements**
   - Tool-specific terminal themes
   - Status badges on tabs
   - Loading animations
   - Error state handling

2. **Activity Bar Enhancement**
   - Add terminal count badge
   - Show running AI tools count
   - Quick access menu on right-click

3. **Settings Integration**
   - Terminal preferences (font, size, theme)
   - Tool auto-launch on startup
   - Memory Service connection settings
   - Default shell configuration

## 🏗️ Architecture Decisions

### 1. Process Isolation
- Each terminal runs in its own process
- Crashes don't affect other terminals
- Resource limits per terminal

### 2. Data Flow
```
Renderer (xterm.js) ←→ IPC ←→ Main (node-pty) ←→ Shell Process
                    ↓
              Terminal Manager
                    ↓
            Isolated Terminal Panel
```

### 3. State Management
```typescript
interface TerminalState {
  terminals: Map<string, {
    id: string
    type: 'system' | 'user' | 'ai-tool'
    title: string
    process?: IPty
    xterm?: Terminal
    status: 'running' | 'stopped' | 'crashed'
    toolId?: string
    workingDirectory: string
  }>
  activeTerminalId: string
  aiToolStatuses: Map<string, ToolStatus>
}
```

## 🔧 Technical Implementation Details

### Terminal Creation Flow
1. User clicks "+" or AI tool launches
2. IsolatedTerminalPanel creates new tab
3. TerminalManager creates xterm instance
4. IPC request to main process
5. Main spawns node-pty process
6. Bidirectional data flow established
7. Terminal ready for interaction

### AI Tool Launch Flow
1. Tool detection on startup
2. User clicks "Launch" in CLI Tools panel
3. Check if tool is installed
4. Create dedicated terminal tab
5. Configure environment variables
6. Launch tool with proper arguments
7. Monitor process health
8. Update UI status indicators

## 📊 Success Metrics

### Phase 1-2 Success
- [ ] xterm.js terminals working
- [ ] Can type commands and see output
- [ ] Proper resize handling
- [ ] System Log still captures console

### Phase 3-4 Success
- [ ] All major AI CLI tools detected
- [ ] Tools launch in dedicated terminals
- [ ] Clean process management
- [ ] No interference with main app

### Phase 5-6 Success
- [ ] Memory Service auto-configuration
- [ ] Tools successfully query memory
- [ ] Professional terminal features
- [ ] Keyboard shortcuts working

### Phase 7 Success
- [ ] Polished, professional UI
- [ ] Seamless user experience
- [ ] All tools integrated
- [ ] Performance targets met

## 🚦 Risk Mitigation

### Risk: node-pty compatibility
**Mitigation**: Use electron-rebuild, test on all platforms

### Risk: Performance with multiple terminals
**Mitigation**: Lazy loading, virtual scrolling, resource limits

### Risk: Tool detection failures
**Mitigation**: Manual configuration fallback, clear error messages

### Risk: Process management complexity
**Mitigation**: Robust error handling, automatic cleanup, health monitoring

## 📅 Timeline

**Week 1**: xterm.js integration, basic process management
**Week 2**: AI tool detection, tool launching
**Week 3**: Memory Service integration, advanced features
**Week 4**: Polish, testing, documentation

## 🎯 Next Immediate Steps

1. **Install xterm.js dependencies**
2. **Create TerminalManager class**
3. **Update IsolatedTerminalPanel to use xterm**
4. **Implement IPC handlers for terminal operations**
5. **Test with basic shell commands**

This plan builds on our solid foundation and provides a clear path to achieving our terminal and AI CLI integration goals while maintaining the isolation and stability we've established.