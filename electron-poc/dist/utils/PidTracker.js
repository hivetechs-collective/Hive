"use strict";
/**
 * PidTracker - Tracks process IDs across application restarts
 * Helps clean up orphaned processes from previous runs
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
exports.PidTracker = void 0;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const os = __importStar(require("os"));
const SafeLogger_1 = require("./SafeLogger");
class PidTracker {
    /**
     * Record a PID for tracking
     */
    static addPid(pid, name) {
        try {
            let pids = {};
            if (fs.existsSync(this.pidFile)) {
                const content = fs.readFileSync(this.pidFile, 'utf-8');
                try {
                    pids = JSON.parse(content);
                }
                catch (_a) {
                    // Invalid JSON, start fresh
                    pids = {};
                }
            }
            if (!pids[name]) {
                pids[name] = [];
            }
            if (!pids[name].includes(pid)) {
                pids[name].push(pid);
            }
            fs.writeFileSync(this.pidFile, JSON.stringify(pids, null, 2));
            SafeLogger_1.logger.info(`[PidTracker] Recorded PID ${pid} for ${name}`);
        }
        catch (error) {
            SafeLogger_1.logger.error(`[PidTracker] Failed to record PID: ${error}`);
        }
    }
    /**
     * Remove a PID from tracking
     */
    static removePid(pid) {
        try {
            if (!fs.existsSync(this.pidFile))
                return;
            const content = fs.readFileSync(this.pidFile, 'utf-8');
            let pids = {};
            try {
                pids = JSON.parse(content);
            }
            catch (_a) {
                return; // Invalid JSON, nothing to remove
            }
            for (const name in pids) {
                pids[name] = pids[name].filter(p => p !== pid);
                if (pids[name].length === 0) {
                    delete pids[name];
                }
            }
            if (Object.keys(pids).length > 0) {
                fs.writeFileSync(this.pidFile, JSON.stringify(pids, null, 2));
            }
            else {
                // No more PIDs, remove the file
                fs.unlinkSync(this.pidFile);
            }
            SafeLogger_1.logger.info(`[PidTracker] Removed PID ${pid}`);
        }
        catch (error) {
            SafeLogger_1.logger.error(`[PidTracker] Failed to remove PID: ${error}`);
        }
    }
    /**
     * Clean up orphaned processes from previous runs
     */
    static cleanupOrphans() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                if (!fs.existsSync(this.pidFile))
                    return;
                const content = fs.readFileSync(this.pidFile, 'utf-8');
                let pids = {};
                try {
                    pids = JSON.parse(content);
                }
                catch (_a) {
                    // Invalid JSON, remove file
                    fs.unlinkSync(this.pidFile);
                    return;
                }
                SafeLogger_1.logger.info('[PidTracker] Checking for orphaned processes...');
                for (const name in pids) {
                    for (const pid of pids[name]) {
                        if (this.isProcessRunning(pid)) {
                            SafeLogger_1.logger.info(`[PidTracker] Killing orphaned process ${name} (PID: ${pid})`);
                            try {
                                process.kill(pid, 'SIGTERM');
                                // Give it a moment to terminate gracefully
                                yield new Promise(resolve => setTimeout(resolve, 1000));
                                // Force kill if still running
                                if (this.isProcessRunning(pid)) {
                                    process.kill(pid, 'SIGKILL');
                                }
                            }
                            catch (error) {
                                // Process might have already died
                                SafeLogger_1.logger.debug(`[PidTracker] Failed to kill PID ${pid}: ${error}`);
                            }
                        }
                    }
                }
                // Clear the PID file after cleanup
                fs.unlinkSync(this.pidFile);
                SafeLogger_1.logger.info('[PidTracker] Orphaned process cleanup complete');
            }
            catch (error) {
                SafeLogger_1.logger.error(`[PidTracker] Cleanup failed: ${error}`);
            }
        });
    }
    /**
     * Check if a process is running
     */
    static isProcessRunning(pid) {
        try {
            // Sending signal 0 checks if process exists without killing it
            process.kill(pid, 0);
            return true;
        }
        catch (_a) {
            return false;
        }
    }
    /**
     * Get all tracked PIDs
     */
    static getTrackedPids() {
        try {
            if (!fs.existsSync(this.pidFile))
                return {};
            const content = fs.readFileSync(this.pidFile, 'utf-8');
            return JSON.parse(content);
        }
        catch (_a) {
            return {};
        }
    }
}
exports.PidTracker = PidTracker;
PidTracker.pidFile = path.join(os.tmpdir(), 'hive-electron-poc.pids');
//# sourceMappingURL=PidTracker.js.map