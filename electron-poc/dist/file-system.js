"use strict";
/**
 * High-Performance File System Operations
 * Runs in main process for optimal performance
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
exports.FileSystemManager = void 0;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const util_1 = require("util");
const fs_1 = require("fs");
const readdir = (0, util_1.promisify)(fs.readdir);
const stat = (0, util_1.promisify)(fs.stat);
const readFile = (0, util_1.promisify)(fs.readFile);
const writeFile = (0, util_1.promisify)(fs.writeFile);
class FileSystemManager {
    constructor() {
        this.watchHandlers = new Map();
        this.fileCache = new Map();
        this.CACHE_TTL = 5000; // 5 seconds cache
    }
    /**
     * Get file tree with depth limit for performance
     */
    getFileTree(rootPath, maxDepth = 1) {
        return __awaiter(this, void 0, void 0, function* () {
            // Only get the first level - children will be loaded lazily
            return this.scanDirectory(rootPath, 0, maxDepth);
        });
    }
    scanDirectory(dirPath, currentDepth, maxDepth) {
        return __awaiter(this, void 0, void 0, function* () {
            if (currentDepth >= maxDepth) {
                return [];
            }
            try {
                // Remove console.log statements as they cause freeze with SafeLogger
                const entries = yield readdir(dirPath, { withFileTypes: true });
                const nodes = [];
                // Process entries without expensive stat calls
                for (const entry of entries) {
                    // Skip hidden files, node_modules, and other heavy directories for performance
                    if (entry.name.startsWith('.') ||
                        entry.name === 'node_modules' ||
                        entry.name === 'dist' ||
                        entry.name === 'build' ||
                        entry.name === 'coverage' ||
                        entry.name === '.git' ||
                        entry.name === 'target') {
                        continue;
                    }
                    const fullPath = path.join(dirPath, entry.name);
                    const node = {
                        name: entry.name,
                        path: fullPath,
                        type: entry.isDirectory() ? 'directory' : 'file'
                    };
                    if (entry.isDirectory() && currentDepth < maxDepth - 1) {
                        // Lazy load children - don't load immediately
                        node.children = [];
                    }
                    // Skip stat calls for files - this is the performance killer
                    // We can get size/modified info only when specifically needed
                    nodes.push(node);
                }
                return nodes.sort((a, b) => {
                    // Directories first, then alphabetical
                    if (a.type !== b.type) {
                        return a.type === 'directory' ? -1 : 1;
                    }
                    return a.name.localeCompare(b.name);
                });
            }
            catch (error) {
                console.error(`Error scanning directory ${dirPath}:`, error);
                return [];
            }
        });
    }
    /**
     * Get directory contents (for lazy loading)
     */
    getDirectoryContents(dirPath) {
        return __awaiter(this, void 0, void 0, function* () {
            return this.scanDirectory(dirPath, 0, 1);
        });
    }
    /**
     * Read file with caching and streaming for large files
     */
    readFile(filePath) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check cache first
            const cached = this.fileCache.get(filePath);
            if (cached && Date.now() - cached.timestamp < this.CACHE_TTL) {
                return cached.content;
            }
            const stats = yield stat(filePath);
            // For large files (>1MB), use streaming
            if (stats.size > 1024 * 1024) {
                return this.readLargeFile(filePath);
            }
            // For small files, read normally
            const content = yield readFile(filePath, 'utf-8');
            // Cache the content
            this.fileCache.set(filePath, {
                content,
                timestamp: Date.now()
            });
            return content;
        });
    }
    /**
     * Read large file in chunks
     */
    readLargeFile(filePath) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve, reject) => {
                const chunks = [];
                const stream = (0, fs_1.createReadStream)(filePath, { encoding: 'utf8' });
                stream.on('data', (chunk) => chunks.push(Buffer.from(chunk)));
                stream.on('end', () => resolve(Buffer.concat(chunks).toString()));
                stream.on('error', reject);
            });
        });
    }
    /**
     * Write file with atomic write for safety
     */
    writeFileContent(filePath, content) {
        return __awaiter(this, void 0, void 0, function* () {
            // Write to temp file first
            const tempPath = `${filePath}.tmp`;
            yield writeFile(tempPath, content, 'utf-8');
            // Atomic rename
            fs.renameSync(tempPath, filePath);
            // Invalidate cache
            this.fileCache.delete(filePath);
        });
    }
    /**
     * Watch file for changes
     */
    watchFile(filePath, callback) {
        // Close existing watcher
        this.unwatchFile(filePath);
        const watcher = fs.watch(filePath, { persistent: false }, (eventType) => {
            if (eventType === 'change') {
                // Invalidate cache
                this.fileCache.delete(filePath);
                // Debounce callback
                setTimeout(callback, 100);
            }
        });
        this.watchHandlers.set(filePath, watcher);
    }
    /**
     * Stop watching file
     */
    unwatchFile(filePath) {
        const watcher = this.watchHandlers.get(filePath);
        if (watcher) {
            watcher.close();
            this.watchHandlers.delete(filePath);
        }
    }
    /**
     * Search files with ripgrep for performance
     */
    searchFiles(rootPath, pattern) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve, reject) => {
                const { spawn } = require('child_process');
                const rg = spawn('rg', ['-l', pattern, rootPath]);
                let output = '';
                rg.stdout.on('data', (data) => {
                    output += data.toString();
                });
                rg.on('close', (code) => {
                    if (code === 0) {
                        resolve(output.split('\n').filter(Boolean));
                    }
                    else {
                        // Fallback to Node.js search if ripgrep not available
                        resolve([]);
                    }
                });
                rg.on('error', () => {
                    // Fallback to Node.js search
                    resolve([]);
                });
            });
        });
    }
    /**
     * Get file stats
     */
    getFileStats(filePath) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                return yield stat(filePath);
            }
            catch (error) {
                return null;
            }
        });
    }
    /**
     * Clean up resources
     */
    destroy() {
        // Close all watchers
        this.watchHandlers.forEach(watcher => watcher.close());
        this.watchHandlers.clear();
        // Clear cache
        this.fileCache.clear();
    }
}
exports.FileSystemManager = FileSystemManager;
//# sourceMappingURL=file-system.js.map