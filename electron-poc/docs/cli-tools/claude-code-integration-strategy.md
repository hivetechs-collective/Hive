# Claude Code Integration Strategy

## The Installation & Launch Challenge

Claude Code presents a unique integration challenge because:
1. It's installed globally but used locally
2. It needs project context to be useful
3. Users need to authenticate before first use
4. It's a terminal-based tool, not a GUI application

## Proposed Solution: Smart Installation & Launch

### Phase 1: Installation
When Claude Code is not installed, the Install button should:

1. **Install Globally**:
   ```bash
   npm install -g @anthropic-ai/claude-code
   ```

2. **Verify Installation**:
   ```bash
   claude --version
   ```

3. **Configure Memory Service Integration**:
   - Generate authentication token
   - Update MCP configuration
   - Create wrapper scripts

### Phase 2: Post-Installation Actions

After successful installation, transform the UI to show:

```
Claude Code ✅ Installed v1.0.86
Status: Ready to launch
Memory: Connected ✓

[Launch in Project] [Open Terminal] [Configure]
```

### Phase 3: Launch Options

#### Option A: "Launch in Project" Button
```javascript
async function launchClaudeInProject() {
  // 1. Get current project directory from file explorer
  const projectPath = getCurrentProjectPath();
  
  // 2. Open terminal in that directory
  const terminal = openIntegratedTerminal(projectPath);
  
  // 3. Execute claude command
  terminal.sendText('claude');
  
  // 4. Show instructions overlay
  showClaudeQuickStart();
}
```

#### Option B: "Open Terminal" Button
```javascript
async function openTerminalWithInstructions() {
  // 1. Open integrated terminal
  const terminal = openIntegratedTerminal();
  
  // 2. Show instructions
  terminal.sendText('# Claude Code is installed globally');
  terminal.sendText('# Navigate to your project directory:');
  terminal.sendText('# cd /path/to/your/project');
  terminal.sendText('# Then start Claude Code:');
  terminal.sendText('# claude');
}
```

#### Option C: "Quick Start" Wizard
```javascript
async function claudeQuickStart() {
  // 1. Show dialog
  const result = await showDialog({
    title: 'Launch Claude Code',
    message: 'Select how to launch Claude Code:',
    buttons: [
      'Current Project',
      'Select Directory',
      'Open Terminal'
    ]
  });
  
  // 2. Handle selection
  switch(result) {
    case 'Current Project':
      launchInCurrentProject();
      break;
    case 'Select Directory':
      const dir = await selectDirectory();
      launchInDirectory(dir);
      break;
    case 'Open Terminal':
      openTerminalWithInstructions();
      break;
  }
}
```

## Implementation Recommendations

### 1. Enhanced Install Flow

```typescript
async function installClaudeCode() {
  // Step 1: Install globally
  await exec('npm install -g @anthropic-ai/claude-code');
  
  // Step 2: Verify installation
  const version = await exec('claude --version');
  
  // Step 3: Configure Memory Service
  await configureMemoryService('claude-code');
  
  // Step 4: Check authentication status
  const needsAuth = await checkClaudeAuth();
  
  // Step 5: Update UI
  if (needsAuth) {
    showAuthInstructions();
  } else {
    showLaunchOptions();
  }
}
```

### 2. Authentication Handling

Claude Code requires authentication on first use:

```typescript
interface ClaudeAuthState {
  authenticated: boolean;
  method: 'console' | 'api-key' | null;
  hasActiveBilling: boolean;
}

async function checkClaudeAuth(): Promise<ClaudeAuthState> {
  // Check for API key in environment
  const hasApiKey = !!process.env.ANTHROPIC_API_KEY;
  
  // Check for saved credentials
  const configPath = path.join(os.homedir(), '.claude', 'config.json');
  const hasConfig = fs.existsSync(configPath);
  
  return {
    authenticated: hasApiKey || hasConfig,
    method: hasApiKey ? 'api-key' : hasConfig ? 'console' : null,
    hasActiveBilling: false // Would need API check
  };
}
```

### 3. Project Context Detection

```typescript
interface ProjectContext {
  path: string;
  name: string;
  hasGitRepo: boolean;
  language: string;
  framework?: string;
}

function getCurrentProjectContext(): ProjectContext {
  // Get from file explorer or recent projects
  const explorerPath = fileExplorer.getCurrentPath();
  
  return {
    path: explorerPath,
    name: path.basename(explorerPath),
    hasGitRepo: fs.existsSync(path.join(explorerPath, '.git')),
    language: detectProjectLanguage(explorerPath),
    framework: detectFramework(explorerPath)
  };
}
```

### 4. UI States After Installation

#### State 1: Just Installed (Needs Auth)
```
Claude Code ✅ Installed v1.0.86
⚠️ Authentication Required

First time setup:
1. Click "Setup Claude" below
2. Complete browser authentication
3. Return here to launch

[Setup Claude] [View Guide]
```

#### State 2: Authenticated (Ready to Launch)
```
Claude Code ✅ Installed v1.0.86
✅ Authenticated via Anthropic Console
Memory: Connected ✓

Current Project: /Users/veronelazio/Developer/Private/hive
Language: TypeScript | Framework: Electron

[Launch Claude Here] [Change Directory] [Settings]
```

#### State 3: Running in Terminal
```
Claude Code ✅ Running
Session: Active in Terminal #1
Memory: Syncing ✓

Current Context: 
- Project: hive-electron-poc
- Files: 127 tracked
- Token Usage: 45,231 / 200,000

[View Terminal] [Stop Session] [New Session]
```

## Integration with Integrated Terminal

### Terminal Manager Component

```typescript
class TerminalManager {
  private terminals: Map<string, Terminal> = new Map();
  
  async launchClaude(projectPath: string): Promise<Terminal> {
    // Create new terminal
    const terminal = await this.createTerminal({
      name: `Claude: ${path.basename(projectPath)}`,
      cwd: projectPath,
      env: {
        ...process.env,
        CLAUDE_PROJECT_DIR: projectPath
      }
    });
    
    // Navigate and launch
    terminal.sendText(`cd "${projectPath}"`);
    terminal.sendText('claude');
    
    // Track session
    this.terminals.set(projectPath, terminal);
    
    return terminal;
  }
  
  async stopClaude(projectPath: string) {
    const terminal = this.terminals.get(projectPath);
    if (terminal) {
      terminal.sendText('\x03'); // Ctrl+C
      terminal.dispose();
      this.terminals.delete(projectPath);
    }
  }
}
```

## Best Practices

1. **Always Launch in Context**: Never run Claude without a project directory
2. **Preserve Working Directory**: Don't change the user's current directory unexpectedly
3. **Show Clear Instructions**: Help users understand where Claude is running
4. **Handle Authentication Gracefully**: Guide users through first-time setup
5. **Integrate with File Explorer**: Use the current file explorer context
6. **Track Claude Sessions**: Show which projects have active Claude sessions
7. **Memory Service Auto-Config**: Automatically configure Memory Service after install

## Future Enhancements

1. **Project Templates**: "Launch Claude with React template"
2. **Session Management**: Multiple Claude sessions in different projects
3. **Context Switching**: Quick switch between project contexts
4. **Preset Commands**: Common Claude commands as buttons
5. **Output Capture**: Show Claude's responses in the UI
6. **Token Tracking**: Display token usage in real-time
7. **Team Sharing**: Share Claude sessions with team members

## Conclusion

The Install button should:
1. Install Claude Code globally via npm
2. Configure Memory Service integration automatically
3. Transform into launch controls after installation
4. Provide clear options for launching in the right context
5. Handle authentication flow gracefully
6. Integrate with the app's terminal or file explorer

This approach acknowledges that Claude Code is a terminal tool that needs project context, while making it as seamless as possible to install and launch from within our IDE.