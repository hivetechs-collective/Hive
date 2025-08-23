"use strict";
/**
 * SafeLogger - Production-ready logging system with EPIPE error handling
 *
 * Replaces console.log/error to prevent EPIPE crashes when child processes
 * use stdio: 'inherit'. Implements 2025 best practices for Electron apps.
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
exports.logger = exports.LogLevel = void 0;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const os = __importStar(require("os"));
var LogLevel;
(function (LogLevel) {
    LogLevel[LogLevel["DEBUG"] = 0] = "DEBUG";
    LogLevel[LogLevel["INFO"] = 1] = "INFO";
    LogLevel[LogLevel["WARN"] = 2] = "WARN";
    LogLevel[LogLevel["ERROR"] = 3] = "ERROR";
    LogLevel[LogLevel["FATAL"] = 4] = "FATAL";
})(LogLevel = exports.LogLevel || (exports.LogLevel = {}));
class SafeLogger {
    constructor(options = {}) {
        var _a, _b, _c, _d, _e;
        this.writeStream = null;
        this.logQueue = [];
        this.isWriting = false;
        this.logLevel = (_a = options.logLevel) !== null && _a !== void 0 ? _a : LogLevel.INFO;
        this.logToFile = (_b = options.logToFile) !== null && _b !== void 0 ? _b : true;
        this.logToConsole = (_c = options.logToConsole) !== null && _c !== void 0 ? _c : (process.env.NODE_ENV === 'development');
        this.maxFileSize = (_d = options.maxFileSize) !== null && _d !== void 0 ? _d : 10 * 1024 * 1024; // 10MB default
        this.maxFiles = (_e = options.maxFiles) !== null && _e !== void 0 ? _e : 5;
        // Use app userData directory for logs, or fallback to home directory for child processes
        if (options.logDir) {
            this.logDir = options.logDir;
        }
        else {
            try {
                // Try to use Electron's app if available (main process)
                const { app } = require('electron');
                this.logDir = path.join(app.getPath('userData'), 'logs');
            }
            catch (_f) {
                // Fallback for child processes or when Electron is not available
                this.logDir = path.join(os.homedir(), '.hive-consensus', 'logs');
            }
        }
        if (this.logToFile) {
            this.initializeFileLogging();
        }
        // Handle process exit gracefully
        process.on('exit', () => this.close());
        process.on('SIGINT', () => this.close());
        process.on('SIGTERM', () => this.close());
    }
    static getInstance(options) {
        if (!SafeLogger.instance) {
            SafeLogger.instance = new SafeLogger(options);
        }
        return SafeLogger.instance;
    }
    initializeFileLogging() {
        try {
            // Ensure log directory exists
            if (!fs.existsSync(this.logDir)) {
                fs.mkdirSync(this.logDir, { recursive: true });
            }
            // Create log filename with timestamp
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
            this.currentLogFile = path.join(this.logDir, `hive-${timestamp}.log`);
            // Create write stream with append flag
            this.writeStream = fs.createWriteStream(this.currentLogFile, {
                flags: 'a',
                encoding: 'utf8'
            });
            this.writeStream.on('error', (error) => {
                // If we can't write to file, fall back to console only
                this.logToFile = false;
                this.safeConsoleLog('ERROR', `Failed to write to log file: ${error.message}`);
            });
            // Rotate logs if needed
            this.rotateLogsIfNeeded();
        }
        catch (error) {
            this.logToFile = false;
            this.safeConsoleLog('ERROR', `Failed to initialize file logging: ${error}`);
        }
    }
    rotateLogsIfNeeded() {
        try {
            const stats = fs.statSync(this.currentLogFile);
            if (stats.size > this.maxFileSize) {
                this.rotateLogs();
            }
        }
        catch (_a) {
            // File doesn't exist yet, that's fine
        }
        // Clean up old log files
        this.cleanOldLogs();
    }
    rotateLogs() {
        if (this.writeStream) {
            this.writeStream.end();
            this.initializeFileLogging();
        }
    }
    cleanOldLogs() {
        try {
            const files = fs.readdirSync(this.logDir)
                .filter(f => f.startsWith('hive-') && f.endsWith('.log'))
                .map(f => ({
                name: f,
                path: path.join(this.logDir, f),
                time: fs.statSync(path.join(this.logDir, f)).mtime.getTime()
            }))
                .sort((a, b) => b.time - a.time);
            // Keep only the most recent files
            if (files.length > this.maxFiles) {
                files.slice(this.maxFiles).forEach(file => {
                    try {
                        fs.unlinkSync(file.path);
                    }
                    catch (_a) {
                        // Ignore errors when deleting old logs
                    }
                });
            }
        }
        catch (_a) {
            // Ignore errors in cleanup
        }
    }
    formatMessage(level, message, meta) {
        const timestamp = new Date().toISOString();
        const pid = process.pid;
        let formatted = `[${timestamp}] [${level}] [PID:${pid}] ${message}`;
        if (meta) {
            try {
                formatted += ` ${JSON.stringify(meta)}`;
            }
            catch (_a) {
                formatted += ` [Unserializable meta data]`;
            }
        }
        return formatted;
    }
    safeConsoleLog(level, message) {
        if (!this.logToConsole)
            return;
        try {
            // Check if stdout is writable before attempting to write
            if (process.stdout && process.stdout.writable) {
                const output = `[${level}] ${message}\n`;
                process.stdout.write(output);
            }
        }
        catch (error) {
            // Silently ignore EPIPE errors - this is the whole point of SafeLogger
            if (error.code !== 'EPIPE' && error.code !== 'EBADF') {
                // For non-EPIPE errors, try stderr as fallback
                try {
                    if (process.stderr && process.stderr.writable) {
                        process.stderr.write(`[LOGGER ERROR] Failed to write to stdout: ${error.message}\n`);
                    }
                }
                catch (_a) {
                    // Give up - we can't write anywhere
                }
            }
        }
    }
    writeToFile(message) {
        if (!this.logToFile || !this.writeStream)
            return;
        // Add to queue
        this.logQueue.push(message + '\n');
        // Process queue if not already processing
        if (!this.isWriting) {
            this.processQueue();
        }
    }
    processQueue() {
        return __awaiter(this, void 0, void 0, function* () {
            if (this.isWriting || this.logQueue.length === 0)
                return;
            this.isWriting = true;
            while (this.logQueue.length > 0) {
                const message = this.logQueue.shift();
                try {
                    yield new Promise((resolve, reject) => {
                        if (!this.writeStream) {
                            resolve();
                            return;
                        }
                        this.writeStream.write(message, (error) => {
                            if (error)
                                reject(error);
                            else
                                resolve();
                        });
                    });
                }
                catch (error) {
                    // If write fails, try to reinitialize
                    this.initializeFileLogging();
                    break;
                }
            }
            this.isWriting = false;
            // Check for rotation after write
            this.rotateLogsIfNeeded();
        });
    }
    writeLog(level, levelName, message, meta) {
        if (level < this.logLevel)
            return;
        const formatted = this.formatMessage(levelName, message, meta);
        // Write to console (safely)
        this.safeConsoleLog(levelName, message);
        // Write to file
        this.writeToFile(formatted);
    }
    // Public logging methods
    debug(message, meta) {
        this.writeLog(LogLevel.DEBUG, 'DEBUG', message, meta);
    }
    info(message, meta) {
        this.writeLog(LogLevel.INFO, 'INFO', message, meta);
    }
    warn(message, meta) {
        this.writeLog(LogLevel.WARN, 'WARN', message, meta);
    }
    error(message, meta) {
        this.writeLog(LogLevel.ERROR, 'ERROR', message, meta);
    }
    fatal(message, meta) {
        this.writeLog(LogLevel.FATAL, 'FATAL', message, meta);
    }
    // Compatibility methods for easy migration from console
    log(message, ...args) {
        const meta = args.length > 0 ? args : undefined;
        this.writeLog(LogLevel.INFO, 'INFO', message, meta);
    }
    close() {
        if (this.writeStream) {
            // Flush any remaining logs
            this.processQueue().then(() => {
                if (this.writeStream) {
                    this.writeStream.end();
                    this.writeStream = null;
                }
            });
        }
    }
    // Get log file path for debugging
    getLogFilePath() {
        return this.currentLogFile;
    }
    // Set log level dynamically
    setLogLevel(level) {
        this.logLevel = level;
    }
}
// Export singleton instance for easy use
exports.logger = SafeLogger.getInstance();
// Export default for convenience
exports.default = SafeLogger;
//# sourceMappingURL=SafeLogger.js.map