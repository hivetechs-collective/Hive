"use strict";
/**
 * TTYDManager - Manages ttyd terminal server instances for tabbed terminals
 * Works with ProcessManager for port allocation and process lifecycle
 * Updated: 12:49 PM - Added debug logging
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
exports.TTYDManager = void 0;
const child_process_1 = require("child_process");
const events_1 = require("events");
const PortManager_1 = require("../utils/PortManager");
const PidTracker_1 = require("../utils/PidTracker");
const SafeLogger_1 = require("../utils/SafeLogger");
class TTYDManager extends events_1.EventEmitter {
    constructor(processManager) {
        super();
        this.instances = new Map();
        this.ttydBinaryPath = 'ttyd'; // Assume ttyd is in PATH
        this.processManager = processManager;
        this.verifyTTYDInstalled();
    }
    /**
     * Verify ttyd is installed
     */
    verifyTTYDInstalled() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const { execSync } = require('child_process');
                execSync('which ttyd', { stdio: 'ignore' });
                SafeLogger_1.logger.info('[TTYDManager] ttyd binary found in PATH');
                return true;
            }
            catch (error) {
                SafeLogger_1.logger.error('[TTYDManager] ttyd not found! Please install: brew install ttyd');
                this.emit('error', new Error('ttyd not installed'));
                return false;
            }
        });
    }
    /**
     * Create a new ttyd terminal instance
     */
    createTerminal(config) {
        return __awaiter(this, void 0, void 0, function* () {
            SafeLogger_1.logger.info(`[TTYDManager] Creating terminal: ${config.title}`);
            // Allocate port through ProcessManager's PortManager
            // Dynamic port allocation: Start from 7100 (avoiding common system ports like 7000)
            // PortManager will automatically find the next available port if primary is taken
            const port = yield PortManager_1.PortManager.allocatePort({
                port: 7100,
                serviceName: `ttyd-${config.id}`,
                alternativePorts: Array.from({ length: 900 }, (_, i) => 7100 + i) // Large range: 7100-7999
            });
            SafeLogger_1.logger.info(`[TTYDManager] Allocated port ${port} for ${config.title}`);
            // Prepare ttyd arguments
            const ttydArgs = [
                '--port', port.toString(),
                '--interface', '127.0.0.1',
                '--writable', // Allow input
                // Note: --check-origin is a flag, not a key-value option
                // '--base-path', `/terminal/${config.id}`,  // Commented out - may interfere with routing
                // '--title', config.title  // Title doesn't exist as an option in ttyd
            ];
            // Add authentication if needed (for security in production)
            if (process.env.NODE_ENV === 'production') {
                // In production, we might want to add basic auth
                // ttydArgs.push('--credential', 'user:pass');
            }
            // Determine shell and initial command
            const shell = config.shell || '/bin/zsh';
            const shellArgs = [];
            // If we have a command to auto-execute (like 'claude'), prepare it
            if (config.command) {
                // Start an interactive shell that will execute the command after initialization
                // We use a small sleep to ensure the shell is ready, then execute the command
                // The command will appear in the terminal and execute properly
                const initCommand = `sleep 0.5 && ${config.command}`;
                ttydArgs.push('--', shell, '-c', `${initCommand}; exec ${shell} -i`);
            }
            else {
                // Just start the shell normally
                ttydArgs.push('--', shell);
            }
            // Log the full command for debugging
            SafeLogger_1.logger.info(`[TTYDManager] Spawning ttyd: ${this.ttydBinaryPath} ${ttydArgs.join(' ')}`);
            console.log(`[TTYDManager] Spawning ttyd: ${this.ttydBinaryPath} ${ttydArgs.join(' ')}`);
            // Spawn ttyd process
            const ttydProcess = (0, child_process_1.spawn)(this.ttydBinaryPath, ttydArgs, {
                cwd: config.cwd || process.env.HOME,
                env: Object.assign(Object.assign(Object.assign({}, process.env), config.env), { TERM: 'xterm-256color', COLORTERM: 'truecolor', 
                    // Disable zsh's % marker for cleaner output
                    PROMPT_EOL_MARK: '' }),
                detached: false
            });
            // Track the PID for cleanup
            if (ttydProcess.pid) {
                PidTracker_1.PidTracker.addPid(ttydProcess.pid, `ttyd-${config.id}`);
            }
            // Create instance object
            const instance = {
                id: config.id,
                title: config.title,
                toolId: config.toolId,
                port,
                url: `http://localhost:${port}`,
                process: ttydProcess,
                status: 'starting',
                createdAt: new Date(),
                config
            };
            // Store instance
            this.instances.set(config.id, instance);
            // Set up process event handlers
            ttydProcess.on('error', (error) => {
                SafeLogger_1.logger.error(`[TTYDManager] Terminal ${config.title} error:`, error);
                instance.status = 'stopped';
                this.emit('terminal:error', config.id, error);
                this.cleanupTerminal(config.id);
            });
            ttydProcess.on('exit', (code, signal) => {
                SafeLogger_1.logger.info(`[TTYDManager] Terminal ${config.title} exited (code: ${code}, signal: ${signal})`);
                instance.status = 'stopped';
                this.emit('terminal:closed', config.id);
                this.cleanupTerminal(config.id);
            });
            // Capture ttyd output for debugging
            if (ttydProcess.stdout) {
                ttydProcess.stdout.on('data', (data) => {
                    SafeLogger_1.logger.debug(`[TTYDManager] ${config.title} stdout:`, data.toString());
                });
            }
            if (ttydProcess.stderr) {
                ttydProcess.stderr.on('data', (data) => {
                    // ttyd logs to stderr, but most of it is just info
                    const message = data.toString();
                    // Always log stderr for debugging ttyd startup issues
                    SafeLogger_1.logger.error(`[TTYDManager] ${config.title} stderr:`, message);
                    console.error(`[TTYDManager] ${config.title} stderr:`, message);
                    if (message.includes('error') || message.includes('Error')) {
                        SafeLogger_1.logger.error(`[TTYDManager] ${config.title} stderr:`, message);
                    }
                    else {
                        SafeLogger_1.logger.debug(`[TTYDManager] ${config.title} info:`, message);
                    }
                });
            }
            // Wait for ttyd to be ready (port to be listening)
            const isReady = yield this.waitForTerminalReady(port, config.title);
            if (isReady) {
                instance.status = 'running';
                this.emit('terminal:ready', config.id, instance);
                SafeLogger_1.logger.info(`[TTYDManager] Terminal ${config.title} is ready at ${instance.url}`);
                // If we have a command to execute, send it after a short delay
                if (config.command) {
                    setTimeout(() => {
                        this.executeCommand(config.id, config.command);
                    }, 500);
                }
            }
            else {
                SafeLogger_1.logger.error(`[TTYDManager] Terminal ${config.title} failed to start`);
                yield this.closeTerminal(config.id);
                throw new Error(`Failed to start terminal ${config.title}`);
            }
            return instance;
        });
    }
    /**
     * Wait for ttyd to be ready on the specified port
     */
    waitForTerminalReady(port, title) {
        return __awaiter(this, void 0, void 0, function* () {
            const maxAttempts = 30; // 3 seconds total
            const checkInterval = 100; // Check every 100ms
            for (let attempt = 0; attempt < maxAttempts; attempt++) {
                const isListening = yield PortManager_1.PortManager.waitForService(port, checkInterval);
                if (isListening) {
                    SafeLogger_1.logger.info(`[TTYDManager] Port ${port} is ready for ${title}`);
                    return true;
                }
                if (attempt === maxAttempts / 2) {
                    SafeLogger_1.logger.info(`[TTYDManager] Still waiting for ${title} on port ${port}...`);
                }
            }
            SafeLogger_1.logger.error(`[TTYDManager] Timeout waiting for ${title} on port ${port}`);
            return false;
        });
    }
    /**
     * Execute a command in a terminal (via JavaScript injection)
     * Note: This requires the webview to call this after it's loaded
     */
    executeCommand(terminalId, command) {
        const instance = this.instances.get(terminalId);
        if (!instance) {
            SafeLogger_1.logger.error(`[TTYDManager] Terminal ${terminalId} not found`);
            return;
        }
        // Emit event for the renderer to handle
        // The renderer's webview will execute JavaScript to send the command
        this.emit('terminal:execute', terminalId, command);
    }
    /**
     * Close a terminal instance
     */
    closeTerminal(id) {
        return __awaiter(this, void 0, void 0, function* () {
            const instance = this.instances.get(id);
            if (!instance) {
                return true;
            }
            SafeLogger_1.logger.info(`[TTYDManager] Closing terminal: ${instance.title}`);
            instance.status = 'stopping';
            // Kill the ttyd process
            if (instance.process && !instance.process.killed) {
                instance.process.kill('SIGTERM');
                // Give it time to shutdown gracefully
                yield new Promise(resolve => setTimeout(resolve, 500));
                // Force kill if still running
                if (!instance.process.killed) {
                    instance.process.kill('SIGKILL');
                }
            }
            // Cleanup
            yield this.cleanupTerminal(id);
            return true;
        });
    }
    /**
     * Clean up terminal resources
     */
    cleanupTerminal(id) {
        return __awaiter(this, void 0, void 0, function* () {
            const instance = this.instances.get(id);
            if (!instance) {
                return;
            }
            // Remove PID tracking
            if (instance.process && instance.process.pid) {
                PidTracker_1.PidTracker.removePid(instance.process.pid);
            }
            // Release the port
            if (instance.port) {
                PortManager_1.PortManager.releasePort(`ttyd-${id}`);
                SafeLogger_1.logger.info(`[TTYDManager] Released port ${instance.port} for ${instance.title}`);
            }
            // Remove from instances map
            this.instances.delete(id);
            this.emit('terminal:cleaned', id);
        });
    }
    /**
     * Get a terminal instance
     */
    getTerminal(id) {
        return this.instances.get(id);
    }
    /**
     * Get all terminal instances
     */
    getAllTerminals() {
        return Array.from(this.instances.values());
    }
    /**
     * Check if a terminal is running
     */
    isTerminalRunning(id) {
        const instance = this.instances.get(id);
        return (instance === null || instance === void 0 ? void 0 : instance.status) === 'running';
    }
    /**
     * Get terminal by tool ID
     */
    getTerminalByToolId(toolId) {
        return Array.from(this.instances.values()).find(instance => instance.toolId === toolId);
    }
    /**
     * Restart a terminal
     */
    restartTerminal(id) {
        return __awaiter(this, void 0, void 0, function* () {
            const instance = this.instances.get(id);
            if (!instance) {
                return null;
            }
            const config = instance.config;
            yield this.closeTerminal(id);
            // Wait a bit before restarting
            yield new Promise(resolve => setTimeout(resolve, 1000));
            return this.createTerminal(config);
        });
    }
    /**
     * Clean up all terminals
     */
    cleanup() {
        return __awaiter(this, void 0, void 0, function* () {
            SafeLogger_1.logger.info('[TTYDManager] Cleaning up all terminals...');
            const closePromises = Array.from(this.instances.keys()).map(id => this.closeTerminal(id));
            yield Promise.all(closePromises);
            this.instances.clear();
            SafeLogger_1.logger.info('[TTYDManager] All terminals cleaned up');
        });
    }
    /**
     * Get status of all terminals
     */
    getStatus() {
        const terminals = Array.from(this.instances.values()).map(instance => ({
            id: instance.id,
            title: instance.title,
            toolId: instance.toolId,
            port: instance.port,
            status: instance.status,
            url: instance.url,
            uptime: Date.now() - instance.createdAt.getTime()
        }));
        return {
            total: terminals.length,
            running: terminals.filter(t => t.status === 'running').length,
            stopped: terminals.filter(t => t.status === 'stopped').length,
            terminals
        };
    }
}
exports.TTYDManager = TTYDManager;
exports.default = TTYDManager;
//# sourceMappingURL=TTYDManager.js.map