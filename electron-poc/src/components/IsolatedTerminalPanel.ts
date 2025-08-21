/**
 * IsolatedTerminalPanel - A completely isolated terminal implementation
 * This component is designed to have zero impact on the rest of the application
 */

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
        
        // Try to use xterm if available, otherwise fallback to simple div
        if (terminalManager) {
            try {
                // Create xterm instance for System Log
                const terminalInstance = terminalManager.createTerminal(tab.id, {
                    title: '📊 System Log',
                    type: 'system-log'
                });
                
                // Attach terminal to DOM
                terminalManager.attachToElement(tab.id, content);
                
                // Write initial messages
                terminalManager.writeToTerminal(tab.id, 
                    '\x1b[36m[INFO] Isolated Terminal System Initialized\r\n' +
                    '\x1b[36m[INFO] System Log tab created\r\n' +
                    '\x1b[96m[INFO] This is a completely isolated terminal panel\r\n' +
                    '\x1b[96m[INFO] It will not affect any other part of the application\r\n'
                );
                
                tab.terminalInstance = terminalInstance;
            } catch (e) {
                console.warn('[IsolatedTerminalPanel] Failed to create xterm, using fallback', e);
                // Fallback to simple HTML
                content.innerHTML = `
                    <div style="color: #569cd6;">[INFO] Isolated Terminal System Initialized</div>
                    <div style="color: #569cd6;">[INFO] System Log tab created</div>
                    <div style="color: #4ec9b0;">[INFO] This is a completely isolated terminal panel</div>
                    <div style="color: #4ec9b0;">[INFO] It will not affect any other part of the application</div>
                `;
            }
        } else {
            // Fallback when terminalManager not available
            content.innerHTML = `
                <div style="color: #569cd6;">[INFO] Isolated Terminal System Initialized</div>
                <div style="color: #569cd6;">[INFO] System Log tab created</div>
                <div style="color: #4ec9b0;">[INFO] This is a completely isolated terminal panel</div>
                <div style="color: #4ec9b0;">[INFO] It will not affect any other part of the application</div>
            `;
        }
        
        tab.element = content;
        this.tabs.set(tab.id, tab);
        this.activeTabId = tab.id;

        // Start capturing console output
        this.startConsoleCapture(tab.id, content);
    }

    private createTerminalTab(): void {
        const tabId = `terminal-${this.terminalCounter++}`;
        const tab: TerminalTab = {
            id: tabId,
            title: `Terminal ${this.terminalCounter - 1}`,
            type: 'terminal',
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

        // Create content area
        const content = document.createElement('div');
        content.id = `isolated-content-${tabId}`;
        content.className = 'isolated-tab-content';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            padding: 10px;
            overflow-y: auto;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 12px;
            color: #cccccc;
            background: #000000;
            display: none;
        `;
        
        // Add placeholder content for now
        content.innerHTML = `
            <div style="color: #4ec9b0;">$ Welcome to ${tab.title}</div>
            <div style="color: #969696;">This will be a fully functional terminal once xterm.js is integrated</div>
            <div style="color: #969696;">For now, it's just a placeholder showing isolation works</div>
            <div style="color: #cccccc;">$ <span style="border-right: 2px solid #cccccc; animation: blink 1s infinite;">_</span></div>
        `;
        
        this.contentContainer.appendChild(content);
        
        tab.element = content;
        this.tabs.set(tabId, tab);
        
        // Switch to the new tab
        this.switchToTab(tabId);
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
            }
            selectedTab.isActive = true;
            this.activeTabId = tabId;
        }
    }

    private closeTab(tabId: string): void {
        // Don't allow closing the System Log tab
        if (tabId === 'system-log') return;

        const tab = this.tabs.get(tabId);
        if (tab) {
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
            this.addLogEntry(terminalId, 'INFO', args.join(' '), fallbackElement);
            originalLog.apply(console, args);
        };

        console.error = (...args: any[]) => {
            this.addLogEntry(terminalId, 'ERROR', args.join(' '), fallbackElement);
            originalError.apply(console, args);
        };

        console.warn = (...args: any[]) => {
            this.addLogEntry(terminalId, 'WARN', args.join(' '), fallbackElement);
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

        // Create content area
        const content = document.createElement('div');
        content.id = `isolated-content-${tabId}`;
        content.className = 'isolated-tab-content';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            padding: 10px;
            overflow-y: auto;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 12px;
            color: #cccccc;
            background: #000000;
            display: none;
        `;
        
        // Add initial content showing launch status
        const folderName = workingDirectory.split('/').pop() || workingDirectory;
        content.innerHTML = `
            <div style="color: #4ec9b0;">$ cd ${workingDirectory}</div>
            <div style="color: #4ec9b0;">$ ${this.getToolCommand(toolId)}</div>
            <div style="color: #dcdcaa;">Launching ${toolName}...</div>
            <div style="color: #969696;">Working directory: ${folderName}</div>
            <div style="color: #969696;">Connecting to Memory Service at http://localhost:3457...</div>
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
        const terminalAPI = (window as any).terminalAPI;
        if (!terminalAPI) {
            throw new Error('Terminal API not available');
        }
        
        const command = this.getToolCommand(toolId);
        const tab = this.tabs.get(tabId);
        if (!tab || !tab.element) {
            throw new Error('Tab not found');
        }
        
        // Create the terminal process
        const result = await terminalAPI.createTerminalProcess({
            terminalId: tabId,
            command: command,
            args: [],
            cwd: workingDirectory,
            env: {
                MEMORY_SERVICE_URL: 'http://localhost:3457',
                HIVE_INTEGRATION: 'true'
            }
        });
        
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
        
        // Set up data listener for this terminal
        if (!this.terminalDataListenerSetup) {
            this.setupTerminalDataListener();
            this.terminalDataListenerSetup = true;
        }
        
        console.log(`[IsolatedTerminalPanel] Process created for ${toolId}, PID: ${result.pid}`);
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

// Add blinking cursor animation
const style = document.createElement('style');
style.textContent = `
    @keyframes blink {
        0%, 50% { opacity: 1; }
        51%, 100% { opacity: 0; }
    }
`;
document.head.appendChild(style);