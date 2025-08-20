"use strict";
/**
 * ProcessManager - Manages child processes lifecycle for production
 * Handles spawning, monitoring, restarting, and cleanup
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
exports.ProcessManager = void 0;
const child_process_1 = require("child_process");
const events_1 = require("events");
const PortManager_1 = require("./PortManager");
class ProcessManager extends events_1.EventEmitter {
    constructor() {
        super();
        this.processes = new Map();
        this.configs = new Map();
        this.healthCheckTimers = new Map();
        this.shutdownInProgress = false;
        this.setupShutdownHandlers();
    }
    /**
     * Register a process configuration
     */
    registerProcess(config) {
        this.configs.set(config.name, config);
        this.processes.set(config.name, {
            name: config.name,
            process: null,
            status: 'stopped',
            restartCount: 0
        });
        console.log(`[ProcessManager] Registered process: ${config.name}`);
    }
    /**
     * Start a managed process
     */
    startProcess(name) {
        return __awaiter(this, void 0, void 0, function* () {
            const config = this.configs.get(name);
            if (!config) {
                throw new Error(`Process ${name} not registered`);
            }
            const info = this.processes.get(name);
            if (info.status === 'running') {
                console.log(`[ProcessManager] Process ${name} is already running`);
                return true;
            }
            info.status = 'starting';
            this.emit('process:starting', name);
            try {
                // Allocate port if needed - PortManager will find an available port
                let port = config.port;
                if (port) {
                    // PortManager will intelligently find an available port
                    port = yield PortManager_1.PortManager.allocatePort({
                        port,
                        serviceName: name,
                        alternativePorts: config.alternativePorts
                    });
                    info.port = port;
                    console.log(`[ProcessManager] ${name} will use port ${port}`);
                }
                // Prepare environment
                const env = Object.assign(Object.assign(Object.assign({}, process.env), config.env), (port ? { PORT: port.toString(), MEMORY_SERVICE_PORT: port.toString() } : {}));
                console.log(`[ProcessManager] Starting ${name} on port ${port || 'N/A'}`);
                // Spawn the process - handle different file types
                let childProcess;
                let binaryReadyPromise = null;
                if (config.scriptPath.endsWith('.ts')) {
                    // For TypeScript files, we need to use fork with ts-node to get IPC
                    // Create a wrapper that uses ts-node/register
                    const tsNodePath = require.resolve('ts-node/register');
                    childProcess = (0, child_process_1.fork)(config.scriptPath, config.args || [], {
                        env,
                        silent: false,
                        detached: false,
                        execArgv: ['-r', tsNodePath]
                    });
                }
                else if (config.scriptPath.endsWith('.js')) {
                    // For JavaScript files, use fork normally
                    childProcess = (0, child_process_1.fork)(config.scriptPath, config.args || [], {
                        env,
                        silent: false,
                        detached: false
                    });
                }
                else {
                    // For binary executables (Rust, Go, etc.), use spawn
                    console.log(`[ProcessManager] Spawning binary executable: ${config.scriptPath}`);
                    // Use 'inherit' for stdio to allow subprocess communication (e.g., Python processes spawned by AI Helpers)
                    // CRITICAL: AI Helpers spawn Python subprocesses that require full stdio access
                    childProcess = (0, child_process_1.spawn)(config.scriptPath, config.args || [], {
                        env,
                        stdio: 'inherit',
                        detached: false
                    });
                    // With 'inherit' stdio, we can't capture output, so we won't have a binaryReadyPromise
                    // We'll rely solely on port checking for readiness detection
                    console.log(`[ProcessManager] Binary process ${name} spawned with inherited stdio`);
                    console.log(`[ProcessManager] Will use port checking for readiness (port ${port})`);
                    // Note: childProcess.stdout and childProcess.stderr are null with 'inherit'
                    // These blocks won't execute with 'inherit' stdio, but keeping them for future reference
                    // if we need to switch back to captured stdio for debugging
                }
                info.process = childProcess;
                info.pid = childProcess.pid;
                info.lastStartTime = new Date();
                // Set up event handlers
                childProcess.on('message', (msg) => {
                    this.handleProcessMessage(name, msg);
                });
                childProcess.on('error', (error) => {
                    console.error(`[ProcessManager] Process ${name} error:`, error);
                    info.lastError = error.message;
                    this.handleProcessCrash(name);
                });
                childProcess.on('exit', (code, signal) => {
                    console.log(`[ProcessManager] Process ${name} exited with code ${code}, signal ${signal}`);
                    if (!this.shutdownInProgress && info.status !== 'stopping') {
                        this.handleProcessCrash(name);
                    }
                    else {
                        info.status = 'stopped';
                        this.emit('process:stopped', name);
                    }
                });
                // Wait for process to be ready - check for 'ready' message or port binding
                // Binary processes like Rust servers may take longer to initialize
                let isReady = false;
                if (config.scriptPath.endsWith('.ts') || config.scriptPath.endsWith('.js')) {
                    // For Node.js processes, wait for IPC 'ready' message
                    const readyPromise = new Promise((resolve) => {
                        const timeout = setTimeout(() => {
                            console.log(`[ProcessManager] Timeout waiting for ${name} ready signal (waited 15000ms)`);
                            resolve(false);
                        }, 15000);
                        const messageHandler = (msg) => {
                            if (msg.type === 'ready') {
                                clearTimeout(timeout);
                                resolve(true);
                            }
                        };
                        childProcess.once('message', messageHandler);
                    });
                    isReady = yield readyPromise;
                }
                else if (binaryReadyPromise) {
                    // For binary processes with captured output, wait for our custom ready detection
                    const timeoutPromise = new Promise((resolve) => {
                        setTimeout(() => {
                            console.log(`[ProcessManager] Timeout waiting for ${name} startup output (waited 30000ms)`);
                            resolve(false);
                        }, 30000);
                    });
                    // Race between the binary ready promise and timeout (binaryReadyPromise is guaranteed to be non-null here)
                    isReady = yield Promise.race([binaryReadyPromise, timeoutPromise]);
                    if (isReady) {
                        console.log(`[ProcessManager] Binary process ${name} confirmed ready via output detection`);
                    }
                }
                else {
                    // For binary processes with 'inherit' stdio, we can't capture output
                    console.log(`[ProcessManager] Binary process ${name} uses inherited stdio - will check port only`);
                }
                // For processes without ready signal, check the port
                if (!isReady && port) {
                    console.log(`[ProcessManager] Checking port ${port} for ${name}...`);
                    // Binary servers may take longer to bind to port after process starts
                    // AI Helpers initialization can take time, so give them enough time to start
                    const isBinary = !config.scriptPath.endsWith('.ts') && !config.scriptPath.endsWith('.js');
                    if (isBinary) {
                        // For binary processes, add initial delay to allow process to initialize
                        console.log(`[ProcessManager] Waiting 2 seconds for ${name} to initialize before port check...`);
                        yield new Promise(resolve => setTimeout(resolve, 2000));
                    }
                    // Fast, efficient port checking (2025 best practice)
                    const maxWaitTime = isBinary ? 15000 : 3000; // 15s for binaries, 3s for Node.js
                    const checkInterval = 250; // Check every 250ms
                    const maxAttempts = Math.floor(maxWaitTime / checkInterval);
                    let attempts = 0;
                    let portReady = false;
                    // Simple, fast checking - no exponential backoff
                    while (attempts < maxAttempts && !portReady) {
                        attempts++;
                        // Quick check if port is listening
                        portReady = yield PortManager_1.PortManager.waitForService(port, checkInterval);
                        if (portReady) {
                            console.log(`[ProcessManager] ✅ Port ${port} is ready for ${name} (${attempts * checkInterval}ms)`);
                            break;
                        }
                        // Only log occasionally to reduce noise
                        if (attempts === maxAttempts / 2) {
                            console.log(`[ProcessManager] Waiting for ${name} on port ${port}...`);
                        }
                    }
                    if (!portReady) {
                        // Try to get more debug info before failing
                        const debugInfo = yield this.debugProcess(name);
                        console.error(`[ProcessManager] Debug info for ${name}:`, JSON.stringify(debugInfo, null, 2));
                        throw new Error(`Process ${name} failed to start properly - port ${port} not responding after ${attempts} attempts (${maxWaitTime}ms)`);
                    }
                    isReady = true;
                }
                info.status = 'running';
                this.emit('process:started', name);
                // Start health checks if configured
                if (config.healthCheckUrl && config.healthCheckInterval) {
                    this.startHealthCheck(name);
                }
                console.log(`[ProcessManager] Process ${name} started successfully (PID: ${info.pid})`);
                return true;
            }
            catch (error) {
                console.error(`[ProcessManager] Failed to start ${name}:`, error.message);
                info.status = 'crashed';
                info.lastError = error.message;
                this.emit('process:failed', name, error);
                // Release port if allocated
                if (info.port) {
                    PortManager_1.PortManager.releasePort(name);
                }
                return false;
            }
        });
    }
    /**
     * Stop a managed process
     */
    stopProcess(name, force = false) {
        return __awaiter(this, void 0, void 0, function* () {
            const info = this.processes.get(name);
            if (!info || info.status === 'stopped') {
                return true;
            }
            info.status = 'stopping';
            this.emit('process:stopping', name);
            // Stop health checks
            this.stopHealthCheck(name);
            if (info.process) {
                try {
                    // Send shutdown message first
                    if (info.process.send) {
                        info.process.send({ type: 'shutdown' });
                    }
                    // Give process time to shutdown gracefully
                    yield new Promise(resolve => setTimeout(resolve, 2000));
                    // Check if process is still running
                    if (info.process.killed === false) {
                        info.process.kill(force ? 'SIGKILL' : 'SIGTERM');
                        // Wait for process to exit
                        yield new Promise(resolve => setTimeout(resolve, 1000));
                    }
                    info.process = null;
                    info.pid = undefined;
                }
                catch (error) {
                    console.error(`[ProcessManager] Error stopping ${name}:`, error);
                }
            }
            // Release port
            if (info.port) {
                PortManager_1.PortManager.releasePort(name);
                info.port = undefined;
            }
            info.status = 'stopped';
            this.emit('process:stopped', name);
            console.log(`[ProcessManager] Process ${name} stopped`);
            return true;
        });
    }
    /**
     * Restart a process
     */
    restartProcess(name) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log(`[ProcessManager] Restarting ${name}...`);
            yield this.stopProcess(name);
            yield new Promise(resolve => setTimeout(resolve, 1000));
            return this.startProcess(name);
        });
    }
    /**
     * Handle process crash and auto-restart if configured
     */
    handleProcessCrash(name) {
        return __awaiter(this, void 0, void 0, function* () {
            const config = this.configs.get(name);
            const info = this.processes.get(name);
            info.status = 'crashed';
            this.emit('process:crashed', name);
            // Clean up process reference
            if (info.process) {
                info.process = null;
                info.pid = undefined;
            }
            // CRITICAL: Release the port when process crashes
            if (info.port) {
                console.log(`[ProcessManager] Releasing port ${info.port} after ${name} crashed`);
                PortManager_1.PortManager.releasePort(name);
                info.port = undefined;
            }
            // Stop health checks
            this.stopHealthCheck(name);
            // Check if auto-restart is enabled
            if ((config === null || config === void 0 ? void 0 : config.autoRestart) && !this.shutdownInProgress) {
                const maxRestarts = config.maxRestarts || 5;
                if (info.restartCount < maxRestarts) {
                    info.restartCount++;
                    const delay = config.restartDelay || 3000;
                    console.log(`[ProcessManager] Auto-restarting ${name} in ${delay}ms (attempt ${info.restartCount}/${maxRestarts})`);
                    setTimeout(() => __awaiter(this, void 0, void 0, function* () {
                        if (!this.shutdownInProgress) {
                            const success = yield this.startProcess(name);
                            if (!success) {
                                console.error(`[ProcessManager] Failed to restart ${name}`);
                            }
                        }
                    }), delay);
                }
                else {
                    console.error(`[ProcessManager] Process ${name} exceeded max restart attempts`);
                    this.emit('process:failed', name, new Error('Max restarts exceeded'));
                }
            }
        });
    }
    /**
     * Handle messages from child processes
     */
    handleProcessMessage(name, message) {
        console.log(`[ProcessManager] Message from ${name}:`, message);
        if (message.type === 'ready') {
            const info = this.processes.get(name);
            info.status = 'running';
            this.emit('process:ready', name, message);
        }
        // Forward message to main process
        this.emit('process:message', name, message);
    }
    /**
     * Start health checks for a process
     */
    startHealthCheck(name) {
        const config = this.configs.get(name);
        const info = this.processes.get(name);
        if (!config.healthCheckUrl || !config.healthCheckInterval) {
            return;
        }
        const timer = setInterval(() => __awaiter(this, void 0, void 0, function* () {
            if (info.status === 'running' && info.port) {
                try {
                    const url = config.healthCheckUrl.replace('{port}', info.port.toString());
                    const controller = new AbortController();
                    const timeout = setTimeout(() => controller.abort(), 5000);
                    const response = yield fetch(url, { signal: controller.signal });
                    clearTimeout(timeout);
                    if (!response.ok) {
                        throw new Error(`Health check failed with status ${response.status}`);
                    }
                    // Reset restart count on successful health check
                    if (info.restartCount > 0) {
                        info.restartCount = 0;
                    }
                }
                catch (error) {
                    console.error(`[ProcessManager] Health check failed for ${name}:`, error.message);
                    this.emit('process:unhealthy', name, error);
                    // Restart if health check fails
                    if (config.autoRestart) {
                        this.handleProcessCrash(name);
                    }
                }
            }
        }), config.healthCheckInterval);
        this.healthCheckTimers.set(name, timer);
    }
    /**
     * Stop health checks for a process
     */
    stopHealthCheck(name) {
        const timer = this.healthCheckTimers.get(name);
        if (timer) {
            clearInterval(timer);
            this.healthCheckTimers.delete(name);
        }
    }
    /**
     * Get process status
     */
    getProcessStatus(name) {
        return this.processes.get(name);
    }
    /**
     * Get all process statuses
     */
    getAllProcesses() {
        return Array.from(this.processes.values());
    }
    /**
     * Get comprehensive status report for all processes
     */
    getFullStatus() {
        const processes = Array.from(this.processes.values()).map(info => ({
            name: info.name,
            status: info.status,
            pid: info.pid,
            port: info.port,
            uptime: info.lastStartTime ? Date.now() - info.lastStartTime.getTime() : undefined,
            restartCount: info.restartCount,
            lastError: info.lastError,
            isPortListening: info.port ? !PortManager_1.PortManager.isPortAvailable(info.port) : undefined
        }));
        const summary = {
            total: processes.length,
            running: processes.filter(p => p.status === 'running').length,
            stopped: processes.filter(p => p.status === 'stopped').length,
            crashed: processes.filter(p => p.status === 'crashed').length,
            starting: processes.filter(p => p.status === 'starting').length
        };
        return {
            processes,
            allocatedPorts: PortManager_1.PortManager.getAllocatedPorts(),
            summary
        };
    }
    /**
     * Get detailed debug information for a specific process
     */
    debugProcess(name) {
        return __awaiter(this, void 0, void 0, function* () {
            const config = this.configs.get(name);
            const info = this.processes.get(name);
            let portStatus;
            if (info === null || info === void 0 ? void 0 : info.port) {
                const isAvailable = yield PortManager_1.PortManager.isPortAvailable(info.port);
                portStatus = {
                    allocated: true,
                    port: info.port,
                    isListening: !isAvailable,
                    canConnect: false
                };
                // Try to connect to the port
                try {
                    const testConnection = yield PortManager_1.PortManager.waitForService(info.port, 100);
                    portStatus.canConnect = testConnection;
                }
                catch (_a) {
                    portStatus.canConnect = false;
                }
            }
            let healthCheck;
            if (config === null || config === void 0 ? void 0 : config.healthCheckUrl) {
                healthCheck = {
                    url: config.healthCheckUrl,
                    status: 'not-configured',
                    error: 'Health check not yet implemented'
                };
            }
            return {
                config,
                info,
                portStatus,
                healthCheck
            };
        });
    }
    /**
     * Log detailed status to console
     */
    logStatus() {
        const status = this.getFullStatus();
        console.log('\n[ProcessManager] === Status Report ===');
        console.log(`Summary: ${status.summary.running}/${status.summary.total} running`);
        console.log('Processes:');
        status.processes.forEach(p => {
            const uptimeStr = p.uptime ? `${Math.floor(p.uptime / 1000)}s` : 'N/A';
            const portStr = p.port ? `:${p.port}` : '';
            const pidStr = p.pid ? `PID:${p.pid}` : '';
            const errorStr = p.lastError ? ` [Error: ${p.lastError}]` : '';
            console.log(`  - ${p.name}${portStr}: ${p.status} ${pidStr} (uptime: ${uptimeStr}, restarts: ${p.restartCount})${errorStr}`);
        });
        if (status.allocatedPorts.size > 0) {
            console.log('\nAllocated Ports:');
            status.allocatedPorts.forEach((port, service) => {
                console.log(`  - ${service}: ${port}`);
            });
        }
        console.log('==================\n');
    }
    /**
     * Set up shutdown handlers
     */
    setupShutdownHandlers() {
        const shutdown = () => __awaiter(this, void 0, void 0, function* () {
            if (this.shutdownInProgress) {
                return;
            }
            this.shutdownInProgress = true;
            console.log('[ProcessManager] Shutting down all processes...');
            // Stop all processes
            const stopPromises = Array.from(this.processes.keys()).map(name => this.stopProcess(name, false));
            yield Promise.all(stopPromises);
            // Clean up ports
            yield PortManager_1.PortManager.cleanup();
            console.log('[ProcessManager] All processes stopped');
        });
        // Handle various shutdown signals
        process.on('SIGTERM', shutdown);
        process.on('SIGINT', shutdown);
        process.on('beforeExit', shutdown);
    }
    /**
     * Clean up all processes
     */
    cleanup() {
        return __awaiter(this, void 0, void 0, function* () {
            this.shutdownInProgress = true;
            // Stop all health checks
            for (const timer of this.healthCheckTimers.values()) {
                clearInterval(timer);
            }
            this.healthCheckTimers.clear();
            // Stop all processes
            for (const name of this.processes.keys()) {
                yield this.stopProcess(name, true);
            }
            // Clean up ports
            yield PortManager_1.PortManager.cleanup();
            this.processes.clear();
            this.configs.clear();
        });
    }
}
exports.ProcessManager = ProcessManager;
exports.default = ProcessManager;
//# sourceMappingURL=ProcessManager.js.map