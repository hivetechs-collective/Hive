/**
 * TerminalManager - Manages xterm.js terminal instances and their processes
 * This provides the bridge between the UI terminals and the underlying shell processes
 */

import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { WebLinksAddon } from 'xterm-addon-web-links';
import { SearchAddon } from 'xterm-addon-search';

export interface TerminalOptions {
    title: string;
    workingDirectory?: string;
    command?: string;
    args?: string[];
    env?: Record<string, string>;
    type: 'system-log' | 'user' | 'ai-tool';
    toolId?: string;
}

export interface TerminalInstance {
    id: string;
    terminal: Terminal;
    fitAddon: FitAddon;
    searchAddon: SearchAddon;
    options: TerminalOptions;
    element?: HTMLElement;
    isActive: boolean;
    processId?: number;
}

export class TerminalManager {
    private terminals: Map<string, TerminalInstance> = new Map();
    private activeTerminalId: string | null = null;
    
    constructor() {
        console.log('[TerminalManager] Initialized');
    }
    
    /**
     * Create a new terminal instance
     */
    createTerminal(id: string, options: TerminalOptions): TerminalInstance {
        console.log(`[TerminalManager] Creating terminal ${id} with options:`, options);
        
        // Create xterm instance with VS Code-like styling
        const terminal = new Terminal({
            fontSize: 13,
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            theme: {
                background: '#1e1e1e',
                foreground: '#cccccc',
                cursor: '#aeafad',
                black: '#000000',
                red: '#cd3131',
                green: '#0dbc79',
                yellow: '#e5e510',
                blue: '#2472c8',
                magenta: '#bc3fbc',
                cyan: '#11a8cd',
                white: '#e5e5e5',
                brightBlack: '#666666',
                brightRed: '#f14c4c',
                brightGreen: '#23d18b',
                brightYellow: '#f5f543',
                brightBlue: '#3b8eea',
                brightMagenta: '#d670d6',
                brightCyan: '#29b8db',
                brightWhite: '#e5e5e5'
            },
            cursorBlink: true,
            cursorStyle: 'block',
            scrollback: 10000,
            allowTransparency: false
        });
        
        // Add addons for enhanced functionality
        const fitAddon = new FitAddon();
        terminal.loadAddon(fitAddon);
        
        const searchAddon = new SearchAddon();
        terminal.loadAddon(searchAddon);
        
        // Add web links support (clickable URLs)
        const webLinksAddon = new WebLinksAddon();
        terminal.loadAddon(webLinksAddon);
        
        // Create terminal instance
        const instance: TerminalInstance = {
            id,
            terminal,
            fitAddon,
            searchAddon,
            options,
            isActive: false
        };
        
        // Store the instance
        this.terminals.set(id, instance);
        
        // For system log, set it as read-only
        if (options.type === 'system-log') {
            terminal.options.disableStdin = true;
            terminal.options.cursorStyle = 'underline';
            terminal.options.cursorBlink = false;
        }
        
        // Request process creation via IPC (will be handled by main process)
        if (options.type !== 'system-log') {
            this.requestProcessCreation(id, options);
        }
        
        return instance;
    }
    
    /**
     * Request main process to create a shell process for this terminal
     */
    private async requestProcessCreation(terminalId: string, options: TerminalOptions): Promise<void> {
        try {
            const terminalAPI = (window as any).terminalAPI;
            if (!terminalAPI || !terminalAPI.createTerminalProcess) {
                console.warn('[TerminalManager] Terminal API not available yet');
                return;
            }
            
            const result = await terminalAPI.createTerminalProcess({
                terminalId,
                command: options.command,
                args: options.args,
                cwd: options.workingDirectory || (window as any).currentOpenedFolder || undefined,
                env: {
                    ...options.env,
                    TERM: 'xterm-256color'
                }
            });
            
            if (result.success) {
                const instance = this.terminals.get(terminalId);
                if (instance) {
                    instance.processId = result.pid;
                    console.log(`[TerminalManager] Process created for ${terminalId}, PID: ${result.pid}`);
                    
                    // Set up data handling
                    this.setupDataHandlers(terminalId);
                }
            } else {
                console.error(`[TerminalManager] Failed to create process for ${terminalId}:`, result.error);
            }
        } catch (error) {
            console.error(`[TerminalManager] Error creating process for ${terminalId}:`, error);
        }
    }
    
    /**
     * Set up data handlers for terminal I/O
     */
    private setupDataHandlers(terminalId: string): void {
        const instance = this.terminals.get(terminalId);
        if (!instance) return;
        
        // Send terminal input to process
        instance.terminal.onData((data) => {
            const terminalAPI = (window as any).terminalAPI;
            if (terminalAPI && terminalAPI.writeToTerminal) {
                terminalAPI.writeToTerminal(terminalId, data);
            }
        });
        
        // Handle terminal resize
        instance.terminal.onResize((size) => {
            const terminalAPI = (window as any).terminalAPI;
            if (terminalAPI && terminalAPI.resizeTerminal) {
                terminalAPI.resizeTerminal(terminalId, size.cols, size.rows);
            }
        });
    }
    
    /**
     * Write data to a terminal (from process output)
     */
    writeToTerminal(terminalId: string, data: string): void {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.terminal.write(data);
        }
    }
    
    /**
     * Attach terminal to DOM element
     */
    attachToElement(terminalId: string, element: HTMLElement): void {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.terminal.open(element);
            instance.element = element;
            
            // Fit terminal to container
            setTimeout(() => {
                instance.fitAddon.fit();
            }, 0);
        }
    }
    
    /**
     * Resize terminal to fit container
     */
    fitTerminal(terminalId: string): void {
        const instance = this.terminals.get(terminalId);
        if (instance && instance.element) {
            instance.fitAddon.fit();
        }
    }
    
    /**
     * Focus a terminal
     */
    focusTerminal(terminalId: string): void {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.terminal.focus();
            this.activeTerminalId = terminalId;
            
            // Update active state
            this.terminals.forEach((term, id) => {
                term.isActive = (id === terminalId);
            });
        }
    }
    
    /**
     * Destroy a terminal and its process
     */
    async destroyTerminal(terminalId: string): Promise<void> {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            // Kill the process if it exists
            if (instance.processId) {
                const terminalAPI = (window as any).terminalAPI;
                if (terminalAPI && terminalAPI.killTerminalProcess) {
                    await terminalAPI.killTerminalProcess(terminalId);
                }
            }
            
            // Dispose of the terminal
            instance.terminal.dispose();
            
            // Remove from map
            this.terminals.delete(terminalId);
            
            console.log(`[TerminalManager] Terminal ${terminalId} destroyed`);
        }
    }
    
    /**
     * Get a terminal instance
     */
    getTerminal(terminalId: string): TerminalInstance | undefined {
        return this.terminals.get(terminalId);
    }
    
    /**
     * Get all terminals
     */
    getAllTerminals(): Map<string, TerminalInstance> {
        return this.terminals;
    }
    
    /**
     * Search in terminal
     */
    searchInTerminal(terminalId: string, searchTerm: string): void {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.searchAddon.findNext(searchTerm);
        }
    }
    
    /**
     * Clear terminal
     */
    clearTerminal(terminalId: string): void {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.terminal.clear();
        }
    }
    
    /**
     * Handle window resize
     */
    handleWindowResize(): void {
        this.terminals.forEach((instance) => {
            if (instance.element) {
                instance.fitAddon.fit();
            }
        });
    }
}

// Export singleton instance
export const terminalManager = new TerminalManager();

// Set up IPC listeners for terminal data
if (typeof window !== 'undefined' && (window as any).terminalAPI) {
    const terminalAPI = (window as any).terminalAPI;
    
    // Listen for terminal output from main process
    if (terminalAPI.onTerminalData) {
        terminalAPI.onTerminalData((terminalId: string, data: string) => {
            terminalManager.writeToTerminal(terminalId, data);
        });
    }
    
    // Listen for terminal exit
    if (terminalAPI.onTerminalExit) {
        terminalAPI.onTerminalExit((terminalId: string, code: number) => {
            console.log(`[TerminalManager] Terminal ${terminalId} exited with code ${code}`);
            terminalManager.writeToTerminal(terminalId, `\r\n[Process exited with code ${code}]\r\n`);
        });
    }
}

// Handle window resize
window.addEventListener('resize', () => {
    terminalManager.handleWindowResize();
});