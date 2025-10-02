# PtyService - Terminal Process Management

## Overview

`PtyService.ts` provides a production-ready abstraction over node-pty for managing terminal processes in Electron's main process. It handles the complete lifecycle of PTY terminals including spawning, data flow, resizing, and cleanup.

## Location

```
/Users/veronelazio/Developer/Private/hive/electron-poc/src/main/terminal/PtyService.ts
```

## Key Features

- **Process Management**: Spawn and manage multiple PTY terminal processes
- **Event-Driven**: Uses EventEmitter for data and exit events
- **Cross-Platform**: Automatic shell detection for Windows, macOS, and Linux
- **Type-Safe**: Full TypeScript types with comprehensive JSDoc documentation
- **Error Handling**: Comprehensive error handling with proper logging
- **Lifecycle Management**: Complete cleanup on application shutdown

## Architecture

```
┌─────────────────────────────────────────┐
│         Renderer Process                │
│  ┌──────────────────────────────────┐  │
│  │  xterm.js Terminal UI            │  │
│  │  - User input                     │  │
│  │  - Display output                 │  │
│  └──────────────────────────────────┘  │
│              │         ▲                │
│              │ IPC     │ IPC            │
│              ▼         │                │
└─────────────────────────────────────────┘
               │         │
┌─────────────────────────────────────────┐
│         Main Process                    │
│  ┌──────────────────────────────────┐  │
│  │  PtyService                       │  │
│  │  - spawn()                        │  │
│  │  - write()                        │  │
│  │  - resize()                       │  │
│  │  - kill()                         │  │
│  │  - onData()                       │  │
│  │  - onExit()                       │  │
│  └──────────────────────────────────┘  │
│              │         ▲                │
│              ▼         │                │
│  ┌──────────────────────────────────┐  │
│  │  node-pty (IPty)                 │  │
│  │  - Pseudo-terminal interface     │  │
│  └──────────────────────────────────┘  │
│              │         ▲                │
│              ▼         │                │
│  ┌──────────────────────────────────┐  │
│  │  Shell Process                   │  │
│  │  (bash, zsh, PowerShell, etc.)   │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## API Reference

### PtyService Class

#### Constructor

```typescript
constructor(logger?: (message: string, ...args: any[]) => void)
```

- **logger**: Optional custom logging function

#### Methods

##### spawn(options?: PtySpawnOptions): string

Spawn a new PTY terminal process.

**Parameters:**
- `options.shell` - Shell command (default: platform-specific)
- `options.args` - Command-line arguments
- `options.env` - Environment variables
- `options.cwd` - Working directory
- `options.cols` - Terminal columns (default: 80)
- `options.rows` - Terminal rows (default: 24)

**Returns:** Unique terminal ID

**Example:**
```typescript
const terminalId = ptyService.spawn({
  shell: '/bin/zsh',
  args: ['--login'],
  cwd: '/Users/username/projects',
  env: { TERM: 'xterm-256color' },
  cols: 80,
  rows: 24
});
```

##### write(terminalId: string, data: string): void

Write data to a terminal (send user input).

**Example:**
```typescript
// Send a command
ptyService.write(terminalId, 'ls -la\r');

// Send Ctrl+C
ptyService.write(terminalId, '\x03');
```

##### resize(terminalId: string, cols: number, rows: number): void

Resize a terminal.

**Example:**
```typescript
ptyService.resize(terminalId, 120, 30);
```

##### kill(terminalId: string, signal?: string): void

Kill a terminal process.

**Example:**
```typescript
// Graceful shutdown
ptyService.kill(terminalId);

// Force kill
ptyService.kill(terminalId, 'SIGKILL');
```

##### onData(terminalId: string, callback: (data: string) => void): () => void

Register a data event handler for a specific terminal.

**Returns:** Unsubscribe function

**Example:**
```typescript
const unsubscribe = ptyService.onData(terminalId, (data) => {
  console.log('Received:', data);
  // Send data to renderer via IPC
  mainWindow.webContents.send('terminal-data', terminalId, data);
});

// Later, remove listener
unsubscribe();
```

##### onExit(terminalId: string, callback: (exitCode: number, signal?: number) => void): () => void

Register an exit event handler for a specific terminal.

**Returns:** Unsubscribe function

**Example:**
```typescript
const unsubscribe = ptyService.onExit(terminalId, (exitCode, signal) => {
  console.log(`Terminal exited with code ${exitCode}`);
  // Notify renderer
  mainWindow.webContents.send('terminal-exit', terminalId, exitCode);
});
```

##### cleanup(): void

Kill all terminals and cleanup. Should be called on application shutdown.

**Example:**
```typescript
app.on('before-quit', () => {
  ptyService.cleanup();
});
```

## Usage Patterns

### Basic Terminal Spawning

```typescript
import { PtyService } from './PtyService';

const ptyService = new PtyService();

// Spawn a terminal
const terminalId = ptyService.spawn({
  cwd: process.env.HOME,
  cols: 80,
  rows: 24
});

// Listen for output
ptyService.onData(terminalId, (data) => {
  console.log('Output:', data);
});

// Send commands
ptyService.write(terminalId, 'echo "Hello, World!"\r');
```

### IPC Integration (Electron)

```typescript
import { ipcMain } from 'electron';
import { PtyService } from './terminal/PtyService';

const ptyService = new PtyService();

// Spawn terminal
ipcMain.handle('terminal:spawn', async (event, options) => {
  try {
    const terminalId = ptyService.spawn(options);

    // Forward data to renderer
    ptyService.onData(terminalId, (data) => {
      event.sender.send('terminal:data', terminalId, data);
    });

    // Forward exit events
    ptyService.onExit(terminalId, (exitCode) => {
      event.sender.send('terminal:exit', terminalId, exitCode);
    });

    return { success: true, terminalId };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Write to terminal
ipcMain.handle('terminal:write', async (event, terminalId, data) => {
  ptyService.write(terminalId, data);
  return { success: true };
});

// Resize terminal
ipcMain.handle('terminal:resize', async (event, terminalId, cols, rows) => {
  ptyService.resize(terminalId, cols, rows);
  return { success: true };
});

// Kill terminal
ipcMain.handle('terminal:kill', async (event, terminalId) => {
  ptyService.kill(terminalId);
  return { success: true };
});
```

### Custom Shell Configuration

```typescript
// macOS with zsh and custom config
const terminalId = ptyService.spawn({
  shell: '/bin/zsh',
  args: ['--login'],
  env: {
    TERM: 'xterm-256color',
    COLORTERM: 'truecolor',
    LANG: 'en_US.UTF-8'
  },
  cwd: '/Users/username/projects'
});

// Windows with PowerShell
const terminalId = ptyService.spawn({
  shell: 'powershell.exe',
  args: ['-NoLogo'],
  cwd: 'C:\\Users\\username\\projects'
});

// Linux with bash
const terminalId = ptyService.spawn({
  shell: '/bin/bash',
  args: ['--login'],
  cwd: process.env.HOME
});
```

### Error Handling

```typescript
// Listen for errors
ptyService.on('error', (terminalId, error) => {
  console.error(`Terminal ${terminalId} error:`, error);
  // Notify user, cleanup, etc.
});

// Graceful spawn with error handling
try {
  const terminalId = ptyService.spawn({
    shell: '/invalid/shell/path'
  });
} catch (error) {
  console.error('Failed to spawn terminal:', error.message);
  // Show error to user
}
```

## Integration with Existing Codebase

### Replacing TTYDManager

The PtyService can directly replace TTYDManager in `src/terminal-ipc-handlers.ts`:

**Before (TTYD):**
```typescript
import TTYDManager from './services/TTYDManager';
const ttydManager = new TTYDManager(processManager);

const terminal = await ttydManager.createTerminal({
  id, title, cwd, command, env
});
```

**After (PTY):**
```typescript
import { PtyService } from './main/terminal/PtyService';
const ptyService = new PtyService();

const terminalId = ptyService.spawn({
  shell: command || undefined,
  cwd,
  env,
  cols: 80,
  rows: 24
});

// Set up event forwarding
ptyService.onData(terminalId, (data) => {
  mainWindow.webContents.send('terminal-data', terminalId, data);
});
```

### Working with TerminalManager.ts

The PtyService pairs perfectly with the existing `TerminalManager.ts` renderer component:

1. **Renderer** (TerminalManager.ts) manages xterm.js UI
2. **IPC handlers** bridge renderer and main process
3. **Main** (PtyService.ts) manages PTY processes

## Platform-Specific Defaults

| Platform | Default Shell | Environment |
|----------|---------------|-------------|
| macOS    | `/bin/zsh`    | TERM=xterm-256color |
| Linux    | `/bin/bash`   | TERM=xterm-256color |
| Windows  | `powershell.exe` | TERM=xterm-256color |

Override by setting `options.shell` in spawn().

## Testing

```typescript
import { PtyService } from './PtyService';

// Create service
const ptyService = new PtyService();

// Test spawning
const terminalId = ptyService.spawn();
console.assert(ptyService.hasTerminal(terminalId), 'Terminal should exist');

// Test data flow
let receivedData = '';
ptyService.onData(terminalId, (data) => {
  receivedData += data;
});

ptyService.write(terminalId, 'echo test\r');

// Wait for output...
setTimeout(() => {
  console.assert(receivedData.includes('test'), 'Should receive echo output');
  ptyService.cleanup();
}, 1000);
```

## Cleanup

Always cleanup on application shutdown:

```typescript
import { app } from 'electron';
import { ptyService } from './main/terminal/PtyService';

app.on('before-quit', () => {
  console.log('Cleaning up terminals...');
  ptyService.cleanup();
});

app.on('window-all-closed', () => {
  ptyService.cleanup();
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
```

## Singleton Pattern

PtyService exports a singleton instance for convenience:

```typescript
// Import singleton
import { ptyService } from './main/terminal/PtyService';

const terminalId = ptyService.spawn();

// Or create your own instance
import { PtyService } from './main/terminal/PtyService';
const customPtyService = new PtyService(customLogger);
```

## Troubleshooting

### Terminal doesn't spawn
- Check shell path is valid
- Verify working directory exists
- Check environment variables are strings

### Data not received
- Ensure onData() listener is set up before writing
- Check IPC communication is working
- Verify terminal hasn't exited

### Resize not working
- Validate cols and rows are positive integers
- Check terminal still exists (hasn't been killed)

### Process not killed
- Try using SIGKILL signal: `ptyService.kill(terminalId, 'SIGKILL')`
- Check terminal exists before killing
- Verify cleanup() is called on app shutdown

## Next Steps

1. **Refactor IPC handlers** to use PtyService instead of TTYDManager
2. **Update TerminalManager.ts** to use new IPC events
3. **Add comprehensive tests** for terminal lifecycle
4. **Remove deprecated TTYD code** after validation
5. **Add advanced features** (command history, session restore, etc.)
