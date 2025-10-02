/**
 * IsolatedTerminalPanel - A completely isolated terminal implementation
 * This component is designed to have zero impact on the rest of the application
 */

// Import xterm.css for terminal styling
import 'xterm/css/xterm.css';

// We'll use dynamic import to avoid build issues for now
let terminalManager: any = null;

// Try to load terminal manager if available
try {
    const terminalModule = require('./terminal/TerminalManager');
    terminalManager = terminalModule.terminalManager;
} catch (e) {
    console.warn('[IsolatedTerminalPanel] TerminalManager not available yet, using fallback');
}

interface TerminalTab {
    id: string;
    title: string;
    type: 'system-log' | 'terminal' | 'ai-tool';
    isActive: boolean;
    element?: HTMLElement;
    terminalInstance?: any; // Use any type to avoid TypeScript issues
    toolId?: string;
}

export class IsolatedTerminalPanel {
    private container: HTMLElement;
    private tabsContainer: HTMLElement;
    private contentContainer: HTMLElement;
    private tabs: Map<string, TerminalTab> = new Map();
    private activeTabId: string | null = null;
    private terminalCounter: number = 1;

    constructor(container: HTMLElement) {
        this.container = container;
        this.tabsContainer = document.getElementById('isolated-terminal-tabs')!;
        this.contentContainer = document.getElementById('isolated-terminal-content')!;
        
        this.initialize();
    }

    private initialize(): void {
        // Create System Log as the first tab
        this.createSystemLogTab();

        // Set up new tab button
        const newTabBtn = document.getElementById('isolated-terminal-new-tab');
        if (newTabBtn) {
            newTabBtn.addEventListener('click', () => this.createTerminalTab());
        }

        // Set up global terminal data listener (ONE listener for ALL terminals)
        const terminalAPI = (window as any).terminalAPI;
        if (terminalAPI) {
            terminalAPI.onTerminalData((terminalId: string, data: string) => {
                const tab = this.tabs.get(terminalId);
                if (tab && (tab as any).terminal) {
                    (tab as any).terminal.write(data);
                }
            });

            terminalAPI.onTerminalExit((terminalId: string, code?: number) => {
                console.log(`[IsolatedTerminalPanel] Terminal ${terminalId} exited with code ${code}`);
                // Optionally remove the tab or show exit status
            });
        }
    }

    private createSystemLogTab(): void {
        const tab: TerminalTab = {
            id: 'system-log',
            title: '📊 System Log',
            type: 'system-log',
            isActive: true
        };

        // Create tab button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'isolated-tab active';
        tabBtn.innerHTML = `
            <span>${tab.title}</span>
        `;
        tabBtn.style.cssText = `
            padding: 6px 12px;
            background: #1e1e1e;
            border: none;
            border-right: 1px solid #3c3c3c;
            color: #cccccc;
            cursor: pointer;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 4px;
        `;
        
        tabBtn.addEventListener('click', () => this.switchToTab(tab.id));
        this.tabsContainer.appendChild(tabBtn);

        // Create content area for terminal
        const content = document.createElement('div');
        content.id = `isolated-content-${tab.id}`;
        content.className = 'isolated-tab-content active';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #1e1e1e;
            display: block;
            overflow-y: auto;
            padding: 10px;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 12px;
            color: #cccccc;
        `;
        
        this.contentContainer.appendChild(content);
        
        // For System Log, always use simple HTML div instead of xterm
        // (xterm seems to be causing the strange characters issue)
        content.innerHTML = `<div style="color: #569cd6;">[INFO] System Log initialized</div>`;
        
        tab.element = content;
        this.tabs.set(tab.id, tab);
        this.activeTabId = tab.id;

        // Start capturing console output
        this.startConsoleCapture(tab.id, content);
    }

    private async createTerminalTab(toolId?: string, command?: string, env?: Record<string, string>): Promise<void> {
        // Use timestamp suffix to ensure unique IDs even after counter reset
        const terminalNum = this.terminalCounter++;
        const tabId = `terminal-${terminalNum}-${Date.now()}`;

        // Determine title based on toolId
        const title = this.getTabTitle(toolId, terminalNum);

        const tab: TerminalTab = {
            id: tabId,
            title: title,
            type: toolId ? 'ai-tool' : 'terminal',
            isActive: false
        };

        // Create tab button with close button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'isolated-tab';
        tabBtn.style.cssText = `
            padding: 6px 12px;
            background: transparent;
            border: none;
            border-right: 1px solid #3c3c3c;
            color: #969696;
            cursor: pointer;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 4px;
            position: relative;
        `;
        
        const tabLabel = document.createElement('span');
        tabLabel.textContent = tab.title;
        tabBtn.appendChild(tabLabel);

        const closeBtn = document.createElement('span');
        closeBtn.textContent = '×';
        closeBtn.style.cssText = `
            margin-left: 8px;
            opacity: 0.6;
            font-size: 16px;
        `;
        closeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            this.closeTab(tabId);
        });
        tabBtn.appendChild(closeBtn);
        
        tabBtn.addEventListener('click', () => this.switchToTab(tabId));
        this.tabsContainer.appendChild(tabBtn);

        // Create content area for xterm.js terminal
        const content = document.createElement('div');
        content.id = `isolated-content-${tabId}`;
        content.className = 'isolated-tab-content';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #000000;
            display: none;
        `;
        
        this.contentContainer.appendChild(content);
        
        tab.element = content;
        this.tabs.set(tabId, tab);
        
        // Switch to the new tab
        this.switchToTab(tabId);
        
        // Create a real xterm.js terminal
        try {
            // Import xterm and create a real terminal
            const { Terminal } = await import('xterm');
            const { FitAddon } = await import('xterm-addon-fit');
            const { Unicode11Addon } = await import('@xterm/addon-unicode11');
            const { WebLinksAddon } = await import('@xterm/addon-web-links');
            
            // Create xterm.js terminal with proper ANSI support
            const terminal = new Terminal({
                fontSize: 13,
                fontFamily: 'Menlo, Monaco, "Courier New", monospace',
                theme: {
                    background: '#1e1e1e',
                    foreground: '#cccccc',
                    cursor: '#aeafad',
                    // Better ANSI color support for Claude Code
                    black: '#000000',
                    red: '#cc0000',
                    green: '#4e9a06',
                    yellow: '#c4a000',
                    blue: '#3465a4',
                    magenta: '#75507b',
                    cyan: '#06989a',
                    white: '#d3d7cf',
                    brightBlack: '#555753',
                    brightRed: '#ef2929',
                    brightGreen: '#8ae234',
                    brightYellow: '#fce94f',
                    brightBlue: '#729fcf',
                    brightMagenta: '#ad7fa8',
                    brightCyan: '#34e2e2',
                    brightWhite: '#eeeeec'
                },
                cursorBlink: true,
                scrollback: 10000,
                // Better handling of control sequences
                convertEol: true,  // Convert line endings properly
                windowsMode: false,  // Use Unix-style line endings
                macOptionIsMeta: true,  // For macOS Option key
                allowProposedApi: true,  // Enable newer xterm.js features
                // Additional settings for better compatibility
                letterSpacing: 0,  // Normal letter spacing
                lineHeight: 1.0,  // Normal line height
                cursorStyle: 'block',  // Block cursor like traditional terminals
                cursorWidth: 1,
                screenReaderMode: false,
                // Shell integration
                allowTransparency: false,
                tabStopWidth: 8,  // Standard tab width
                // Enable alternate screen buffer for TUI apps
                altClickMovesCursor: true
            });
            
            // Add fit addon to make terminal fill container
            const fitAddon = new FitAddon();
            terminal.loadAddon(fitAddon);
            
            // Add Unicode support for better character rendering
            const unicode11Addon = new Unicode11Addon();
            terminal.loadAddon(unicode11Addon);
            
            // Add web links support for clickable URLs
            const webLinksAddon = new WebLinksAddon();
            terminal.loadAddon(webLinksAddon);
            
            // Open terminal in the tab's content element
            terminal.open(tab.element);
            
            // Activate Unicode version 11 support (type cast for compatibility)
            unicode11Addon.activate(terminal as any);
            
            // Fit terminal to container
            fitAddon.fit();
            
            // Force terminal to handle control sequences properly
            terminal.reset();
            terminal.focus();
            
            // Ensure proper terminal dimensions are set
            const dims = fitAddon.proposeDimensions();
            if (dims) {
                console.log(`[Terminal] Setting dimensions: ${dims.cols}x${dims.rows}`);
                terminal.resize(dims.cols, dims.rows);
            }
            
            // Store terminal reference
            (tab as any).terminal = terminal;
            (tab as any).fitAddon = fitAddon;
            
            // Create the PTY process via IPC
            const terminalAPI = (window as any).terminalAPI;
            if (terminalAPI) {
                // Build terminal options with tool parameters
                const options = this.buildTerminalOptions(tabId, toolId, command, env);
                const result = await terminalAPI.createTerminalProcess(options);
                
                if (result.success) {
                    console.log(`[IsolatedTerminalPanel] Terminal ${tabId} created, PID: ${result.pid}`);

                    // Send xterm input to PTY
                    terminal.onData((data: string) => {
                        terminalAPI.writeToTerminal(tabId, data);
                    });

                    // Handle terminal resize
                    terminal.onResize((size: { cols: number; rows: number }) => {
                        terminalAPI.resizeTerminal(tabId, size.cols, size.rows);
                    });
                } else {
                    terminal.write(`\r\n\x1b[91mFailed to create terminal process: ${result.error}\x1b[0m\r\n`);
                }
            } else {
                terminal.write(`\r\n\x1b[93mTerminal API not available\x1b[0m\r\n`);
            }
        } catch (error: any) {
            console.error('[IsolatedTerminalPanel] Failed to create terminal:', error);
            // Fallback to HTML if xterm.js fails
            content.innerHTML = `
                <div style="color: #f44747; padding: 10px;">Failed to create terminal: ${error.message}</div>
            `;
        }
    }

    /**
     * Get display title for terminal tab based on toolId
     * @param toolId - AI tool identifier (optional)
     * @param terminalNum - Terminal number for generic terminals
     * @returns Display title for the tab
     */
    private getTabTitle(toolId?: string, terminalNum?: number): string {
        if (!toolId) {
            return `Terminal ${terminalNum}`;
        }

        // Tool name mapping (matches terminal-ipc-handlers.ts)
        const toolNames: Record<string, string> = {
            'claude-code': 'Claude',
            'claude': 'Claude',
            'gemini-cli': 'Gemini',
            'gemini': 'Gemini',
            'grok': 'Grok',
            'qwen-code': 'Qwen',
            'qwen-coder': 'Qwen',
            'openai-codex': 'Codex',
            'codex': 'Codex',
            'github-copilot': 'Copilot',
            'cursor-cli': 'Cursor',
            'cursor': 'Cursor',
            'cline': 'Cline',
            'aider': 'Aider',
            'continue': 'Continue'
        };

        return toolNames[toolId] || toolId;
    }

    /**
     * Build IPC options object for terminal creation
     * @param terminalId - Unique terminal identifier
     * @param toolId - AI tool identifier (optional)
     * @param command - Command to execute (optional)
     * @param env - Environment variables (optional)
     * @returns Options object for IPC handler
     */
    private buildTerminalOptions(
        terminalId: string,
        toolId?: string,
        command?: string,
        env?: Record<string, string>
    ): any {
        const options: any = {
            terminalId: terminalId
        };

        // Add working directory (use current opened folder or undefined)
        const cwd = (window as any).currentOpenedFolder || undefined;
        if (cwd) {
            options.cwd = cwd;
        }

        // Add tool identifier if provided
        if (toolId) {
            options.toolId = toolId;
        }

        // Add command if provided
        if (command) {
            options.command = command;
        }

        // Add environment variables if provided
        if (env && Object.keys(env).length > 0) {
            options.env = env;
        }

        return options;
    }

    private switchToTab(tabId: string): void {
        // Deactivate all tabs and contents
        this.tabs.forEach((tab, id) => {
            const tabBtn = this.tabsContainer.querySelector(`button:nth-child(${Array.from(this.tabs.keys()).indexOf(id) + 1})`) as HTMLElement;
            if (tabBtn) {
                tabBtn.classList.remove('active');
                if (id === tabId) {
                    tabBtn.style.background = '#1e1e1e';
                    tabBtn.style.color = '#cccccc';
                } else {
                    tabBtn.style.background = 'transparent';
                    tabBtn.style.color = '#969696';
                }
            }
            
            if (tab.element) {
                tab.element.style.display = 'none';
            }
            tab.isActive = false;
        });

        // Activate selected tab
        const selectedTab = this.tabs.get(tabId);
        if (selectedTab) {
            if (selectedTab.element) {
                selectedTab.element.style.display = 'block';
                
                // Resize xterm.js terminal if it exists
                const terminal = (selectedTab as any).terminal;
                const fitAddon = (selectedTab as any).fitAddon;
                if (terminal && fitAddon) {
                    setTimeout(() => fitAddon.fit(), 0);
                }
            }
            selectedTab.isActive = true;
            this.activeTabId = tabId;
        }
    }

    private async closeTab(tabId: string): Promise<void> {
        // Don't allow closing the System Log tab
        if (tabId === 'system-log') return;

        const tab = this.tabs.get(tabId);
        if (tab) {
            // Clean up terminal and PTY process
            const terminal = (tab as any).terminal;
            if (terminal) {
                terminal.dispose();
            }
            
            // Kill the PTY process via IPC
            const terminalAPI = (window as any).terminalAPI;
            if (terminalAPI) {
                try {
                    await terminalAPI.killTerminalProcess(tabId);
                    console.log(`[IsolatedTerminalPanel] Killed terminal process for ${tabId}`);
                } catch (error) {
                    console.error(`[IsolatedTerminalPanel] Failed to kill terminal process for ${tabId}:`, error);
                }
            }
            
            // Remove tab button
            const tabIndex = Array.from(this.tabs.keys()).indexOf(tabId);
            const tabBtn = this.tabsContainer.querySelector(`button:nth-child(${tabIndex + 1})`);
            if (tabBtn) {
                tabBtn.remove();
            }

            // Remove content
            if (tab.element) {
                tab.element.remove();
            }

            // Remove from map
            this.tabs.delete(tabId);

            // Reset terminal counter if only system-log remains
            // Check for any terminal tabs (terminal-N-timestamp format)
            const remainingTerminals = Array.from(this.tabs.keys()).filter(id => 
                id.startsWith('terminal-') && !id.startsWith('terminal-ai-tool-')
            );
            if (remainingTerminals.length === 0) {
                this.terminalCounter = 1;
                console.log('[IsolatedTerminalPanel] All terminals closed, resetting counter to 1');
            }

            // If this was the active tab, switch to system log
            if (this.activeTabId === tabId) {
                this.switchToTab('system-log');
            }
        }
    }

    private startConsoleCapture(terminalId: string, fallbackElement?: HTMLElement): void {
        // Capture console.log output and add to System Log
        const originalLog = console.log;
        const originalError = console.error;
        const originalWarn = console.warn;

        // Override console methods to capture output
        console.log = (...args: any[]) => {
            try {
                const message = args.join(' ');
                this.addLogEntry(terminalId, 'INFO', message, fallbackElement);
            } catch (e) {
                // Ignore any errors in logging
            }
            originalLog.apply(console, args);
        };

        console.error = (...args: any[]) => {
            try {
                const message = args.join(' ');
                this.addLogEntry(terminalId, 'ERROR', message, fallbackElement);
            } catch (e) {
                // Ignore any errors in logging
            }
            originalError.apply(console, args);
        };

        console.warn = (...args: any[]) => {
            try {
                const message = args.join(' ');
                this.addLogEntry(terminalId, 'WARN', message, fallbackElement);
            } catch (e) {
                // Ignore any errors in logging
            }
            originalWarn.apply(console, args);
        };
    }

    /**
     * Create a terminal tab for an AI CLI tool
     * This is called when the user clicks "Launch" on a tool
     */
    public createToolTerminal(toolId: string, toolName: string, workingDirectory: string): string {
        const tabId = `ai-tool-${toolId}`;
        
        // Check if this tool already has a tab
        if (this.tabs.has(tabId)) {
            // Just switch to existing tab
            this.switchToTab(tabId);
            console.log(`[IsolatedTerminalPanel] Tab ${tabId} already exists, switching to it`);
            return tabId;
        }
        
        // Get the tool icon based on toolId
        let icon = '🤖';
        switch (toolId) {
            case 'claude-code': icon = '🤖'; break;
            case 'gemini-cli': icon = '✨'; break;
            case 'qwen-code': icon = '🐉'; break;
            case 'aider': icon = '🔧'; break;
            case 'continue': icon = '▶️'; break;
        }
        
        const tab: TerminalTab = {
            id: tabId,
            title: `${icon} ${toolName}`,
            type: 'ai-tool',
            isActive: false,
            toolId: toolId
        };

        // Create tab button with close button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'isolated-tab';
        tabBtn.style.cssText = `
            padding: 6px 12px;
            background: transparent;
            border: none;
            border-right: 1px solid #3c3c3c;
            color: #969696;
            cursor: pointer;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 4px;
            position: relative;
        `;
        
        const tabLabel = document.createElement('span');
        tabLabel.textContent = tab.title;
        tabBtn.appendChild(tabLabel);

        const closeBtn = document.createElement('span');
        closeBtn.textContent = '×';
        closeBtn.style.cssText = `
            margin-left: 8px;
            opacity: 0.6;
            font-size: 16px;
        `;
        closeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            this.closeTab(tabId);
        });
        tabBtn.appendChild(closeBtn);
        
        tabBtn.addEventListener('click', () => this.switchToTab(tabId));
        this.tabsContainer.appendChild(tabBtn);

        // Create content area for xterm.js terminal
        const content = document.createElement('div');
        content.id = `isolated-content-${tabId}`;
        content.className = 'isolated-tab-content';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #000000;
            display: none;
        `;
        
        this.contentContainer.appendChild(content);
        
        tab.element = content;
        this.tabs.set(tabId, tab);
        
        // Switch to the new tab
        this.switchToTab(tabId);
        
        // Actually launch the tool process via IPC
        this.launchToolProcess(tabId, toolId, workingDirectory).then(() => {
            console.log(`[IsolatedTerminalPanel] Tool ${toolId} process launched`);
        }).catch((error) => {
            console.error(`[IsolatedTerminalPanel] Failed to launch ${toolId}:`, error);
            const errorMsg = document.createElement('div');
            errorMsg.style.color = '#f44747';
            errorMsg.textContent = `✗ Failed to launch ${toolName}: ${error.message}`;
            content.appendChild(errorMsg);
        });
        
        return tabId;
    }
    
    /**
     * Get the command to run for a specific tool
     */
    private getToolCommand(toolId: string): string {
        switch (toolId) {
            case 'claude-code': return 'claude';
            case 'gemini-cli': return 'gemini';
            case 'qwen-code': return 'qwen';
            case 'aider': return 'aider';
            case 'continue': return 'continue';
            default: return toolId;
        }
    }
    
    /**
     * Get the display name for a tool
     */
    private getToolName(toolId: string): string {
        switch (toolId) {
            case 'claude-code': return 'Claude Code';
            case 'gemini-cli': return 'Gemini CLI';
            case 'qwen-code': return 'Qwen Code';
            case 'aider': return 'Aider';
            case 'continue': return 'Continue';
            case 'cursor': return 'Cursor';
            case 'codewhisperer': return 'Amazon Q';
            case 'cody': return 'Cody';
            default: return toolId;
        }
    }
    
    /**
     * Get installation instructions for a tool
     */
    private getInstallInstructions(toolId: string): string[] {
        switch (toolId) {
            case 'claude-code':
                return [
                    '1. Visit: https://claude.ai/download',
                    '2. Download Claude Code for your platform',
                    '3. Install Claude Code following the installer instructions',
                    '4. The "claude" command should be available in your terminal'
                ];
            case 'aider':
                return [
                    '1. Ensure Python 3.7+ is installed',
                    '2. Run: pip install aider-chat',
                    '3. Verify installation: aider --version'
                ];
            case 'cursor':
                return [
                    '1. Visit: https://cursor.sh',
                    '2. Download Cursor for your platform',
                    '3. Install Cursor following the installer instructions'
                ];
            case 'continue':
                return [
                    '1. Install the Continue VS Code extension',
                    '2. Or visit: https://continue.dev for standalone installation'
                ];
            case 'codewhisperer':
                return [
                    '1. Install the AWS Toolkit for VS Code',
                    '2. Or visit: https://aws.amazon.com/q/ for more information'
                ];
            case 'cody':
                return [
                    '1. Visit: https://sourcegraph.com/cody',
                    '2. Install the Cody extension for your IDE'
                ];
            case 'qwen-code':
                return [
                    '1. Ensure Python 3.7+ is installed',
                    '2. Run: pip install qwen-code',
                    '3. Verify installation: qwen --version'
                ];
            case 'gemini-cli':
                return [
                    '1. Visit the Gemini CLI repository',
                    '2. Follow the installation instructions for your platform'
                ];
            default:
                return [
                    `Please install ${this.getToolName(toolId)} and ensure it's available in your PATH.`
                ];
        }
    }
    
    /**
     * Launch the actual tool process via IPC
     */
    private async launchToolProcess(tabId: string, toolId: string, workingDirectory: string): Promise<void> {
        const tab = this.tabs.get(tabId);
        if (!tab || !tab.element) {
            throw new Error('Tab not found');
        }
        
        // Check if terminal already exists for this tab
        if ((tab as any).terminal) {
            console.log(`[IsolatedTerminalPanel] Terminal already exists for ${tabId}, not creating a new one`);
            return;
        }
        
        // Import xterm and create a real terminal
        const { Terminal } = await import('xterm');
        const { FitAddon } = await import('xterm-addon-fit');
        const { Unicode11Addon } = await import('@xterm/addon-unicode11');
        const { WebLinksAddon } = await import('@xterm/addon-web-links');
        
        // Create xterm.js terminal with proper ANSI support
        const terminal = new Terminal({
            fontSize: 13,
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            theme: {
                background: '#1e1e1e',
                foreground: '#cccccc',
                cursor: '#aeafad',
                // Better ANSI color support for Claude Code
                black: '#000000',
                red: '#cc0000',
                green: '#4e9a06',
                yellow: '#c4a000',
                blue: '#3465a4',
                magenta: '#75507b',
                cyan: '#06989a',
                white: '#d3d7cf',
                brightBlack: '#555753',
                brightRed: '#ef2929',
                brightGreen: '#8ae234',
                brightYellow: '#fce94f',
                brightBlue: '#729fcf',
                brightMagenta: '#ad7fa8',
                brightCyan: '#34e2e2',
                brightWhite: '#eeeeec'
            },
            cursorBlink: true,
            scrollback: 10000,
            // Better handling of control sequences
            convertEol: true,  // Convert line endings properly
            windowsMode: false,  // Use Unix-style line endings
            macOptionIsMeta: true,  // For macOS Option key
            allowProposedApi: true,  // Enable newer xterm.js features
            // Enable alternate screen buffer for TUI apps
            altClickMovesCursor: true
        });
        
        // Add fit addon to make terminal fill container
        const fitAddon = new FitAddon();
        terminal.loadAddon(fitAddon);
        
        // Add Unicode support for better character rendering
        const unicode11Addon = new Unicode11Addon();
        terminal.loadAddon(unicode11Addon);
        
        // Add web links support for clickable URLs
        const webLinksAddon = new WebLinksAddon();
        terminal.loadAddon(webLinksAddon);
        
        // Open terminal in the tab's content element
        terminal.open(tab.element);
        
        // Activate Unicode version 11 support (type cast for compatibility)
        unicode11Addon.activate(terminal as any);
        
        // Fit terminal to container
        fitAddon.fit();
        
        // Force terminal to handle control sequences properly
        terminal.reset();
        terminal.focus();
        
        // Ensure proper terminal dimensions are set
        const dims = fitAddon.proposeDimensions();
        if (dims) {
            console.log(`[Terminal] Setting dimensions: ${dims.cols}x${dims.rows}`);
            terminal.resize(dims.cols, dims.rows);
        }
        
        // Store terminal reference
        (tab as any).terminal = terminal;
        (tab as any).fitAddon = fitAddon;
        
        // Now create the PTY process via IPC
        const terminalAPI = (window as any).terminalAPI;
        if (!terminalAPI) {
            throw new Error('Terminal API not available');
        }
        
        const command = this.getToolCommand(toolId);
        console.log(`[IsolatedTerminalPanel] Creating PTY process in ${workingDirectory}`);
        
        const result = await terminalAPI.createTerminalProcess({
            terminalId: tabId,
            cwd: workingDirectory
        });
        console.log(`[IsolatedTerminalPanel] Terminal creation result:`, result);
        
        if (!result.success) {
            // Handle common error cases with user-friendly messages
            const errorMessage = result.error || 'Failed to create terminal process';
            
            if (errorMessage.includes('posix_spawnp failed') || errorMessage.includes('ENOENT')) {
                // Tool not found in PATH - provide helpful instructions
                const content = tab.element;
                content.innerHTML = ''; // Clear loading message
                
                const errorDiv = document.createElement('div');
                errorDiv.style.cssText = 'padding: 20px; font-family: monospace; line-height: 1.6;';
                
                // Error header
                const header = document.createElement('div');
                header.style.cssText = 'color: #f44747; font-weight: bold; margin-bottom: 10px;';
                header.textContent = `✗ ${this.getToolName(toolId)} not found`;
                errorDiv.appendChild(header);
                
                // Explanation
                const explanation = document.createElement('div');
                explanation.style.cssText = 'color: #cccccc; margin-bottom: 15px;';
                explanation.textContent = `The '${command}' command could not be found in your system PATH.`;
                errorDiv.appendChild(explanation);
                
                // Installation instructions
                const instructionsHeader = document.createElement('div');
                instructionsHeader.style.cssText = 'color: #4ec9b0; font-weight: bold; margin-bottom: 10px;';
                instructionsHeader.textContent = 'Installation Instructions:';
                errorDiv.appendChild(instructionsHeader);
                
                const instructions = this.getInstallInstructions(toolId);
                instructions.forEach(instruction => {
                    const step = document.createElement('div');
                    step.style.cssText = 'color: #cccccc; margin-left: 20px; margin-bottom: 5px;';
                    step.textContent = instruction;
                    errorDiv.appendChild(step);
                });
                
                // Additional help
                const helpDiv = document.createElement('div');
                helpDiv.style.cssText = 'color: #969696; margin-top: 15px; font-style: italic;';
                helpDiv.textContent = 'After installation, please restart this application and try again.';
                errorDiv.appendChild(helpDiv);
                
                content.appendChild(errorDiv);
                throw new Error(`${this.getToolName(toolId)} is not installed`);
            } else if (errorMessage.includes('Permission denied')) {
                throw new Error(`Permission denied to execute ${command}. Please check file permissions.`);
            } else {
                throw new Error(errorMessage);
            }
        }
        
        // Connect xterm.js to the PTY process
        // Listen for data from PTY and write to xterm
        terminalAPI.onTerminalData((terminalId: string, data: string) => {
            if (terminalId === tabId && terminal) {
                // Debug: Log control sequences
                if (data.includes('\r') || data.includes('\x1b[')) {
                    const sequences = data.replace(/\x1b/g, '\\x1b')
                        .replace(/\r/g, '\\r')
                        .replace(/\n/g, '\\n');
                    console.log(`[Terminal ${tabId}] Control sequences:`, sequences.substring(0, 100));
                }
                terminal.write(data);
            }
        });
        
        // Send xterm input to PTY
        terminal.onData((data: string) => {
            terminalAPI.writeToTerminal(tabId, data);
        });
        
        // Handle terminal exit
        terminalAPI.onTerminalExit((terminalId: string, code: number) => {
            if (terminalId === tabId && terminal) {
                terminal.write(`\r\n[Process exited with code ${code}]\r\n`);
            }
        });
        
        console.log(`[IsolatedTerminalPanel] Terminal created, PID: ${result.pid}`);
        
        // Now send the command to the terminal (just type it like a user would)
        console.log(`[IsolatedTerminalPanel] Sending command '${command}' to terminal`);
        setTimeout(() => {
            terminalAPI.writeToTerminal(tabId, `${command}\r`);
        }, 500); // Small delay to ensure terminal is ready
    }
    
    private terminalDataListenerSetup = false;
    
    /**
     * Set up global terminal data listener
     */
    private setupTerminalDataListener(): void {
        const terminalAPI = (window as any).terminalAPI;
        if (!terminalAPI) return;
        
        // Listen for terminal output
        terminalAPI.onTerminalData((terminalId: string, data: string) => {
            const tab = this.tabs.get(terminalId);
            if (tab && tab.element) {
                // Parse ANSI codes for basic coloring
                const coloredData = this.parseAnsiToHtml(data);
                
                // Create a span for the new data
                const span = document.createElement('span');
                span.innerHTML = coloredData;
                tab.element.appendChild(span);
                
                // Auto-scroll to bottom
                tab.element.scrollTop = tab.element.scrollHeight;
            }
        });
        
        // Listen for terminal exit
        terminalAPI.onTerminalExit((terminalId: string, code: number) => {
            const tab = this.tabs.get(terminalId);
            if (tab && tab.element) {
                const exitMsg = document.createElement('div');
                exitMsg.style.color = code === 0 ? '#4ec9b0' : '#f44747';
                exitMsg.textContent = `\n[Process exited with code ${code}]`;
                tab.element.appendChild(exitMsg);
            }
        });
    }
    
    /**
     * Basic ANSI to HTML converter
     */
    private parseAnsiToHtml(text: string): string {
        // Very basic ANSI color support
        return text
            .replace(/\x1b\[0m/g, '</span>')
            .replace(/\x1b\[30m/g, '<span style="color: #000000">')
            .replace(/\x1b\[31m/g, '<span style="color: #cd3131">')
            .replace(/\x1b\[32m/g, '<span style="color: #0dbc79">')
            .replace(/\x1b\[33m/g, '<span style="color: #e5e510">')
            .replace(/\x1b\[34m/g, '<span style="color: #2472c8">')
            .replace(/\x1b\[35m/g, '<span style="color: #bc3fbc">')
            .replace(/\x1b\[36m/g, '<span style="color: #11a8cd">')
            .replace(/\x1b\[37m/g, '<span style="color: #e5e5e5">')
            .replace(/\x1b\[90m/g, '<span style="color: #666666">')
            .replace(/\x1b\[91m/g, '<span style="color: #f44747">')
            .replace(/\x1b\[92m/g, '<span style="color: #4ec9b0">')
            .replace(/\x1b\[93m/g, '<span style="color: #dcdcaa">')
            .replace(/\x1b\[94m/g, '<span style="color: #3b8eea">')
            .replace(/\x1b\[95m/g, '<span style="color: #d670d6">')
            .replace(/\x1b\[96m/g, '<span style="color: #29b8db">')
            .replace(/\x1b\[97m/g, '<span style="color: #e5e5e5">')
            .replace(/\r\n/g, '\n')
            .replace(/\n/g, '<br>')
            .replace(/\r/g, '');
    }
    
    private addLogEntry(terminalId: string, level: string, message: string, fallbackElement?: HTMLElement): void {
        const timestamp = new Date().toLocaleTimeString();
        
        // Try to use terminalManager if available
        if (terminalManager && terminalManager.getTerminal && terminalManager.getTerminal(terminalId)) {
            let colorCode = '\x1b[37m'; // Default white
            switch (level) {
                case 'ERROR': colorCode = '\x1b[91m'; break; // Bright red
                case 'WARN': colorCode = '\x1b[93m'; break;  // Bright yellow
                case 'INFO': colorCode = '\x1b[36m'; break;   // Cyan
            }
            
            const logLine = `${colorCode}[${timestamp}] [${level}] ${message}\x1b[0m\r\n`;
            terminalManager.writeToTerminal(terminalId, logLine);
        } else if (fallbackElement) {
            // Fallback to HTML
            const entry = document.createElement('div');
            let color = '#cccccc';
            switch (level) {
                case 'ERROR': color = '#f44747'; break;
                case 'WARN': color = '#dcdcaa'; break;
                case 'INFO': color = '#569cd6'; break;
            }
            
            entry.style.color = color;
            entry.textContent = `[${timestamp}] [${level}] ${message}`;
            fallbackElement.appendChild(entry);
            
            // Auto-scroll to bottom
            fallbackElement.scrollTop = fallbackElement.scrollHeight;
        }
    }
}

// Export singleton instance
export const isolatedTerminalPanel = {
    instance: null as IsolatedTerminalPanel | null,

    initialize(container: HTMLElement): IsolatedTerminalPanel {
        if (!this.instance) {
            this.instance = new IsolatedTerminalPanel(container);
        }
        return this.instance;
    }
};

// Add blinking cursor animation
const style = document.createElement('style');
style.textContent = `
    @keyframes blink {
        0%, 50% { opacity: 1; }
        51%, 100% { opacity: 0; }
    }
`;
document.head.appendChild(style);