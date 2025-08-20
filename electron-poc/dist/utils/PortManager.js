"use strict";
/**
 * PortManager - Handles port allocation and conflicts for production services
 * Ensures clean startup and shutdown of services
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
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
exports.PortManager = void 0;
const net = __importStar(require("net"));
const child_process_1 = require("child_process");
const util_1 = require("util");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
class PortManager {
    /**
     * Check if a port is available
     */
    static isPortAvailable(port) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve) => {
                const server = net.createServer();
                server.once('error', (err) => {
                    if (err.code === 'EADDRINUSE') {
                        resolve(false);
                    }
                    else {
                        resolve(false);
                    }
                });
                server.once('listening', () => {
                    server.close();
                    resolve(true);
                });
                server.listen(port, '127.0.0.1');
            });
        });
    }
    /**
     * Find an available port starting from a preferred port
     */
    static findAvailablePort(preferredPort, alternativePorts) {
        return __awaiter(this, void 0, void 0, function* () {
            // Try preferred port first
            if (yield this.isPortAvailable(preferredPort)) {
                return preferredPort;
            }
            // Try alternative ports
            if (alternativePorts) {
                for (const port of alternativePorts) {
                    if (yield this.isPortAvailable(port)) {
                        return port;
                    }
                }
            }
            // Find next available port
            for (let port = preferredPort + 1; port < preferredPort + 100; port++) {
                if (yield this.isPortAvailable(port)) {
                    return port;
                }
            }
            throw new Error(`No available ports found near ${preferredPort}`);
        });
    }
    /**
     * Kill process using a specific port
     */
    static killProcessOnPort(port) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Find process ID using the port
                const { stdout } = yield execAsync(`lsof -i :${port} -t 2>/dev/null || true`);
                const pids = stdout.trim().split('\n').filter(Boolean);
                if (pids.length > 0) {
                    console.log(`[PortManager] Killing processes on port ${port}: ${pids.join(', ')}`);
                    // Kill each process
                    for (const pid of pids) {
                        try {
                            process.kill(parseInt(pid), 'SIGTERM');
                            // Give it time to terminate gracefully
                            yield new Promise(resolve => setTimeout(resolve, 100));
                            // Force kill if still running
                            try {
                                process.kill(parseInt(pid), 0); // Check if still alive
                                process.kill(parseInt(pid), 'SIGKILL');
                            }
                            catch (_a) {
                                // Process already terminated
                            }
                        }
                        catch (error) {
                            if (error.code !== 'ESRCH') { // Process doesn't exist
                                console.error(`[PortManager] Error killing process ${pid}:`, error.message);
                            }
                        }
                    }
                    // Wait a bit for port to be released
                    yield new Promise(resolve => setTimeout(resolve, 500));
                    return true;
                }
                return false;
            }
            catch (error) {
                console.error('[PortManager] Error finding process on port:', error);
                return false;
            }
        });
    }
    /**
     * Allocate a port for a service with automatic conflict resolution
     */
    static allocatePort(config) {
        return __awaiter(this, void 0, void 0, function* () {
            const { port, serviceName, alternativePorts } = config;
            // Check if service already has an allocated port that's still free
            if (this.allocatedPorts.has(serviceName)) {
                const existingPort = this.allocatedPorts.get(serviceName);
                if (yield this.isPortAvailable(existingPort)) {
                    console.log(`[PortManager] Reusing existing port ${existingPort} for ${serviceName}`);
                    return existingPort;
                }
                // Release the old allocation since it's no longer valid
                this.allocatedPorts.delete(serviceName);
            }
            // Start with preferred port
            let currentPort = port;
            let portToUse = null;
            // Check preferred port first
            if (yield this.isPortAvailable(currentPort)) {
                portToUse = currentPort;
                console.log(`[PortManager] Port ${currentPort} is available for ${serviceName}`);
            }
            else {
                console.log(`[PortManager] Port ${currentPort} is in use, finding next available port...`);
                // Try alternative ports if provided
                if (alternativePorts && alternativePorts.length > 0) {
                    for (const altPort of alternativePorts) {
                        if (yield this.isPortAvailable(altPort)) {
                            portToUse = altPort;
                            console.log(`[PortManager] Using alternative port ${altPort} for ${serviceName}`);
                            break;
                        }
                    }
                }
                // If still no port, scan for the next available port
                if (!portToUse) {
                    currentPort = port + 1;
                    const maxPort = port + 100; // Search up to 100 ports ahead
                    while (currentPort < maxPort) {
                        if (yield this.isPortAvailable(currentPort)) {
                            portToUse = currentPort;
                            console.log(`[PortManager] Found available port ${currentPort} for ${serviceName}`);
                            break;
                        }
                        currentPort++;
                    }
                }
            }
            if (!portToUse) {
                // This should never happen unless all 100 ports are taken
                throw new Error(`Could not find any available port for ${serviceName} (searched ${port} to ${port + 100})`);
            }
            // Allocate the port
            this.allocatedPorts.set(serviceName, portToUse);
            console.log(`[PortManager] ✅ Port ${portToUse} allocated for ${serviceName}`);
            return portToUse;
        });
    }
    /**
     * Release a port allocation
     */
    static releasePort(serviceName) {
        if (this.allocatedPorts.has(serviceName)) {
            const port = this.allocatedPorts.get(serviceName);
            this.allocatedPorts.delete(serviceName);
            console.log(`[PortManager] Released port ${port} for ${serviceName}`);
        }
    }
    /**
     * Clean up all allocated ports
     */
    static cleanup() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[PortManager] Cleaning up all allocated ports...');
            for (const [serviceName, port] of this.allocatedPorts) {
                try {
                    yield this.killProcessOnPort(port);
                    console.log(`[PortManager] Cleaned up port ${port} for ${serviceName}`);
                }
                catch (error) {
                    console.error(`[PortManager] Error cleaning up ${serviceName}:`, error);
                }
            }
            this.allocatedPorts.clear();
        });
    }
    /**
     * Get all allocated ports
     */
    static getAllocatedPorts() {
        return new Map(this.allocatedPorts);
    }
    /**
     * Wait for a service to be ready on a port
     */
    static waitForService(port, timeout = 10000, checkInterval = 100) {
        return __awaiter(this, void 0, void 0, function* () {
            const startTime = Date.now();
            while (Date.now() - startTime < timeout) {
                try {
                    const isInUse = !(yield this.isPortAvailable(port));
                    if (isInUse) {
                        // Port is in use, service is likely ready
                        // Try to make a health check request
                        try {
                            const response = yield fetch(`http://localhost:${port}/health`);
                            if (response.ok) {
                                return true;
                            }
                        }
                        catch (_a) {
                            // Service might not have HTTP endpoint yet
                        }
                        // Port is at least bound
                        return true;
                    }
                }
                catch (_b) {
                    // Ignore errors during wait
                }
                yield new Promise(resolve => setTimeout(resolve, checkInterval));
            }
            return false;
        });
    }
}
exports.PortManager = PortManager;
PortManager.allocatedPorts = new Map();
exports.default = PortManager;
//# sourceMappingURL=PortManager.js.map