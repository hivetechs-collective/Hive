"use strict";
/**
 * TerminalManager - Manages xterm.js terminal instances and their processes
 * This provides the bridge between the UI terminals and the underlying shell processes
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.terminalManager = exports.TerminalManager = void 0;
const xterm_1 = require("xterm");
const xterm_addon_fit_1 = require("xterm-addon-fit");
const xterm_addon_web_links_1 = require("xterm-addon-web-links");
const xterm_addon_search_1 = require("xterm-addon-search");
class TerminalManager {
    constructor() {
        this.terminals = new Map();
        this.activeTerminalId = null;
        console.log('[TerminalManager] Initialized');
    }
    /**
     * Create a new terminal instance
     */
    createTerminal(id, options) {
        console.log(`[TerminalManager] Creating terminal ${id} with options:`, options);
        // Create xterm instance with VS Code-like styling
        const terminal = new xterm_1.Terminal({
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
        const fitAddon = new xterm_addon_fit_1.FitAddon();
        terminal.loadAddon(fitAddon);
        const searchAddon = new xterm_addon_search_1.SearchAddon();
        terminal.loadAddon(searchAddon);
        // Add web links support (clickable URLs)
        const webLinksAddon = new xterm_addon_web_links_1.WebLinksAddon();
        terminal.loadAddon(webLinksAddon);
        // Create terminal instance
        const instance = {
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
    requestProcessCreation(terminalId, options) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const terminalAPI = window.terminalAPI;
                if (!terminalAPI || !terminalAPI.createTerminalProcess) {
                    console.warn('[TerminalManager] Terminal API not available yet');
                    return;
                }
                const result = yield terminalAPI.createTerminalProcess({
                    terminalId,
                    command: options.command,
                    args: options.args,
                    cwd: options.workingDirectory || process.cwd(),
                    env: Object.assign(Object.assign(Object.assign({}, process.env), options.env), { TERM: 'xterm-256color' })
                });
                if (result.success) {
                    const instance = this.terminals.get(terminalId);
                    if (instance) {
                        instance.processId = result.pid;
                        console.log(`[TerminalManager] Process created for ${terminalId}, PID: ${result.pid}`);
                        // Set up data handling
                        this.setupDataHandlers(terminalId);
                    }
                }
                else {
                    console.error(`[TerminalManager] Failed to create process for ${terminalId}:`, result.error);
                }
            }
            catch (error) {
                console.error(`[TerminalManager] Error creating process for ${terminalId}:`, error);
            }
        });
    }
    /**
     * Set up data handlers for terminal I/O
     */
    setupDataHandlers(terminalId) {
        const instance = this.terminals.get(terminalId);
        if (!instance)
            return;
        // Send terminal input to process
        instance.terminal.onData((data) => {
            const terminalAPI = window.terminalAPI;
            if (terminalAPI && terminalAPI.writeToTerminal) {
                terminalAPI.writeToTerminal(terminalId, data);
            }
        });
        // Handle terminal resize
        instance.terminal.onResize((size) => {
            const terminalAPI = window.terminalAPI;
            if (terminalAPI && terminalAPI.resizeTerminal) {
                terminalAPI.resizeTerminal(terminalId, size.cols, size.rows);
            }
        });
    }
    /**
     * Write data to a terminal (from process output)
     */
    writeToTerminal(terminalId, data) {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.terminal.write(data);
        }
    }
    /**
     * Attach terminal to DOM element
     */
    attachToElement(terminalId, element) {
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
    fitTerminal(terminalId) {
        const instance = this.terminals.get(terminalId);
        if (instance && instance.element) {
            instance.fitAddon.fit();
        }
    }
    /**
     * Focus a terminal
     */
    focusTerminal(terminalId) {
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
    destroyTerminal(terminalId) {
        return __awaiter(this, void 0, void 0, function* () {
            const instance = this.terminals.get(terminalId);
            if (instance) {
                // Kill the process if it exists
                if (instance.processId) {
                    const terminalAPI = window.terminalAPI;
                    if (terminalAPI && terminalAPI.killTerminalProcess) {
                        yield terminalAPI.killTerminalProcess(terminalId);
                    }
                }
                // Dispose of the terminal
                instance.terminal.dispose();
                // Remove from map
                this.terminals.delete(terminalId);
                console.log(`[TerminalManager] Terminal ${terminalId} destroyed`);
            }
        });
    }
    /**
     * Get a terminal instance
     */
    getTerminal(terminalId) {
        return this.terminals.get(terminalId);
    }
    /**
     * Get all terminals
     */
    getAllTerminals() {
        return this.terminals;
    }
    /**
     * Search in terminal
     */
    searchInTerminal(terminalId, searchTerm) {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.searchAddon.findNext(searchTerm);
        }
    }
    /**
     * Clear terminal
     */
    clearTerminal(terminalId) {
        const instance = this.terminals.get(terminalId);
        if (instance) {
            instance.terminal.clear();
        }
    }
    /**
     * Handle window resize
     */
    handleWindowResize() {
        this.terminals.forEach((instance) => {
            if (instance.element) {
                instance.fitAddon.fit();
            }
        });
    }
}
exports.TerminalManager = TerminalManager;
// Export singleton instance
exports.terminalManager = new TerminalManager();
// Set up IPC listeners for terminal data
if (typeof window !== 'undefined' && window.terminalAPI) {
    const terminalAPI = window.terminalAPI;
    // Listen for terminal output from main process
    if (terminalAPI.onTerminalData) {
        terminalAPI.onTerminalData((terminalId, data) => {
            exports.terminalManager.writeToTerminal(terminalId, data);
        });
    }
    // Listen for terminal exit
    if (terminalAPI.onTerminalExit) {
        terminalAPI.onTerminalExit((terminalId, code) => {
            console.log(`[TerminalManager] Terminal ${terminalId} exited with code ${code}`);
            exports.terminalManager.writeToTerminal(terminalId, `\r\n[Process exited with code ${code}]\r\n`);
        });
    }
}
// Handle window resize
window.addEventListener('resize', () => {
    exports.terminalManager.handleWindowResize();
});
//# sourceMappingURL=TerminalManager.js.map