// LazyGit Terminal Integration for Electron
// Uses xterm.js for terminal emulation and child_process for LazyGit

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';

// Suppress ResizeObserver errors in webpack dev server
if (typeof window !== 'undefined') {
  const originalError = window.onerror;
  window.onerror = function(message, source, lineno, colno, error) {
    // Ignore ResizeObserver errors
    if (message && typeof message === 'string' && message.includes('ResizeObserver')) {
      return true;
    }
    // Call original error handler if it exists
    if (originalError) {
      return originalError(message, source, lineno, colno, error);
    }
    return false;
  };
}

export class LazyGitTerminal {
  private terminal: Terminal;
  private fitAddon: FitAddon;
  private container: HTMLElement;
  private ws: WebSocket | null = null;
  private isConnected: boolean = false;

  constructor(container: HTMLElement) {
    this.container = container;
    
    // Create terminal with LazyGit-appropriate settings
    this.terminal = new Terminal({
      theme: {
        background: '#1a1b26',
        foreground: '#c0caf5',
        cursor: '#f7768e',
        black: '#15161e',
        red: '#f7768e',
        green: '#9ece6a',
        yellow: '#e0af68',
        blue: '#7aa2f7',
        magenta: '#bb9af7',
        cyan: '#7dcfff',
        white: '#a9b1d6',
        brightBlack: '#414868',
        brightRed: '#f7768e',
        brightGreen: '#9ece6a',
        brightYellow: '#e0af68',
        brightBlue: '#7aa2f7',
        brightMagenta: '#bb9af7',
        brightCyan: '#7dcfff',
        brightWhite: '#c0caf5'
      },
      fontSize: 13,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      cursorBlink: true,
      cursorStyle: 'block',
      scrollback: 1000,
      convertEol: true,
      allowProposedApi: true
    });

    // Add fit addon
    this.fitAddon = new FitAddon();
    this.terminal.loadAddon(this.fitAddon);

    // Add web links addon
    const webLinksAddon = new WebLinksAddon();
    this.terminal.loadAddon(webLinksAddon);
  }

  public async initialize(): Promise<void> {
    // Clear container
    this.container.innerHTML = '';
    
    // Open terminal in container
    this.terminal.open(this.container);
    
    // Fit terminal to container
    this.fitAddon.fit();
    
    // Set up resize observer with debouncing to prevent errors
    let resizeTimeout: NodeJS.Timeout;
    const resizeObserver = new ResizeObserver(() => {
      // Debounce resize operations to prevent ResizeObserver loop errors
      clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(() => {
        try {
          // Use requestAnimationFrame to ensure DOM is ready
          requestAnimationFrame(() => {
            if (this.terminal && this.fitAddon) {
              this.fitAddon.fit();
            }
          });
        } catch (e) {
          // Silently ignore resize errors
        }
      }, 100);
    });
    resizeObserver.observe(this.container);
    
    // Connect to LazyGit backend
    await this.connectToBackend();
    
    // Set up terminal input handler
    this.terminal.onData((data) => {
      if (this.isConnected) {
        const electronAPI = (window as any).electronAPI;
        
        if (electronAPI && electronAPI.writeLazyGit) {
          // Send input via IPC
          electronAPI.writeLazyGit(data);
        } else if (this.ws) {
          // Send input via WebSocket
          this.ws.send(JSON.stringify({ type: 'input', data }));
        }
      }
    });
    
    // Welcome message
    this.terminal.writeln('\x1b[1;32m🚀 LazyGit Terminal\x1b[0m');
    this.terminal.writeln('Connecting to LazyGit...\n');
  }

  private async connectToBackend(): Promise<void> {
    // Check if we have Electron IPC available
    const electronAPI = (window as any).electronAPI;
    
    if (electronAPI && electronAPI.startLazyGit) {
      // Use Electron IPC for LazyGit
      return this.connectViaIPC();
    }
    
    // Fall back to WebSocket (for development)
    return this.connectViaWebSocket();
  }
  
  private async connectViaIPC(): Promise<void> {
    const electronAPI = (window as any).electronAPI;
    
    try {
      // Start LazyGit process
      const result = await electronAPI.startLazyGit();
      
      if (result.success) {
        this.isConnected = true;
        this.terminal.writeln('\x1b[1;32m✓ LazyGit started\x1b[0m\n');
        
        // Set up data handler for LazyGit output
        electronAPI.onLazyGitData((data: string) => {
          this.terminal.write(data);
        });
        
        // Set up exit handler
        electronAPI.onLazyGitExit((code: number) => {
          this.terminal.writeln(`\n\x1b[1;33mLazyGit exited (code ${code})\x1b[0m`);
          this.isConnected = false;
        });
      } else {
        throw new Error(result.error || 'Failed to start LazyGit');
      }
    } catch (error: any) {
      console.error('Failed to start LazyGit:', error);
      this.terminal.writeln(`\x1b[1;31m✗ Failed to start LazyGit: ${error.message}\x1b[0m`);
      this.terminal.writeln('Starting in demo mode...\n');
      this.startDemoMode();
    }
  }
  
  private async connectViaWebSocket(): Promise<void> {
    return new Promise((resolve, reject) => {
      // Connect to LazyGit WebSocket endpoint
      this.ws = new WebSocket('ws://localhost:8766/lazygit');
      
      this.ws.onopen = () => {
        this.isConnected = true;
        this.terminal.writeln('\x1b[1;32m✓ Connected to LazyGit\x1b[0m\n');
        
        // Request LazyGit to start
        this.ws!.send(JSON.stringify({ type: 'start' }));
        resolve();
      };
      
      this.ws.onmessage = (event) => {
        const message = JSON.parse(event.data);
        
        if (message.type === 'output') {
          // Write LazyGit output to terminal
          this.terminal.write(message.data);
        } else if (message.type === 'error') {
          this.terminal.writeln(`\x1b[1;31mError: ${message.message}\x1b[0m`);
        } else if (message.type === 'exit') {
          this.terminal.writeln('\n\x1b[1;33mLazyGit exited\x1b[0m');
          this.isConnected = false;
        }
      };
      
      this.ws.onerror = (error) => {
        console.error('LazyGit WebSocket error:', error);
        this.terminal.writeln('\x1b[1;31m✗ Failed to connect to LazyGit backend\x1b[0m');
        this.terminal.writeln('Starting in demo mode...\n');
        
        // Demo mode - simulate LazyGit
        this.startDemoMode();
        resolve(); // Resolve anyway to not block
      };
      
      this.ws.onclose = () => {
        this.isConnected = false;
        this.terminal.writeln('\n\x1b[1;33mConnection to LazyGit closed\x1b[0m');
      };
      
      // Timeout after 3 seconds
      setTimeout(() => {
        if (!this.isConnected) {
          this.terminal.writeln('\x1b[1;33m⚠ LazyGit backend not available\x1b[0m');
          this.terminal.writeln('Starting in demo mode...\n');
          this.startDemoMode();
          resolve();
        }
      }, 3000);
    });
  }
  
  private startDemoMode(): void {
    // Demo mode - show a simulated LazyGit interface
    const demoOutput = [
      '\x1b[2J\x1b[H', // Clear screen and move to top
      '┌─ Files ─────────────────────────────────────┐',
      '│ \x1b[32m●\x1b[0m M  src/lazygit.ts                        │',
      '│ \x1b[32m●\x1b[0m M  src/renderer.ts                       │',
      '│ \x1b[31m●\x1b[0m D  src/old-file.ts                       │',
      '│ \x1b[33m?\x1b[0m ?  src/new-file.ts                       │',
      '└──────────────────────────────────────────────┘',
      '',
      '┌─ Diff ───────────────────────────────────────┐',
      '│ \x1b[32m+ Added LazyGit terminal integration\x1b[0m      │',
      '│ \x1b[31m- Removed old terminal code\x1b[0m              │',
      '└──────────────────────────────────────────────┘',
      '',
      '┌─ Log ────────────────────────────────────────┐',
      '│ \x1b[33m●\x1b[0m feat: Add LazyGit integration            │',
      '│ \x1b[33m●\x1b[0m fix: Analytics timezone handling         │',
      '│ \x1b[33m●\x1b[0m refactor: Update consensus pipeline      │',
      '└──────────────────────────────────────────────┘',
      '',
      '\x1b[7m[a]dd [c]ommit [p]ush [P]ull [r]efresh [q]uit\x1b[0m'
    ];
    
    demoOutput.forEach(line => {
      this.terminal.writeln(line);
    });
    
    // Handle demo mode input
    this.terminal.onData((data) => {
      if (data === 'q') {
        this.terminal.writeln('\n\x1b[1;33mExiting LazyGit demo mode\x1b[0m');
      } else if (data === 'r') {
        this.terminal.write('\x1b[2J\x1b[H'); // Clear and redraw
        demoOutput.forEach(line => this.terminal.writeln(line));
      } else if (data === 'a') {
        this.terminal.writeln('\n\x1b[1;32mStaging all changes...\x1b[0m');
      } else if (data === 'c') {
        this.terminal.writeln('\n\x1b[1;32mOpening commit dialog...\x1b[0m');
      }
    });
  }

  public sendCommand(command: string): void {
    if (this.isConnected && this.ws) {
      this.ws.send(JSON.stringify({ type: 'command', data: command }));
    } else {
      this.terminal.writeln(`\x1b[1;33mNot connected to LazyGit\x1b[0m`);
    }
  }

  public resize(): void {
    this.fitAddon.fit();
    
    // Send resize event to backend
    if (this.isConnected && this.ws) {
      const dimensions = this.fitAddon.proposeDimensions();
      if (dimensions) {
        this.ws.send(JSON.stringify({
          type: 'resize',
          cols: dimensions.cols,
          rows: dimensions.rows
        }));
      }
    }
  }

  public destroy(): void {
    if (this.ws) {
      this.ws.close();
    }
    this.terminal.dispose();
  }
}

// Export singleton instance manager
export class LazyGitManager {
  private static instance: LazyGitTerminal | null = null;

  public static async initialize(container: HTMLElement): Promise<LazyGitTerminal> {
    if (this.instance) {
      this.instance.destroy();
    }
    
    this.instance = new LazyGitTerminal(container);
    await this.instance.initialize();
    return this.instance;
  }

  public static getInstance(): LazyGitTerminal | null {
    return this.instance;
  }

  public static destroy(): void {
    if (this.instance) {
      this.instance.destroy();
      this.instance = null;
    }
  }
}