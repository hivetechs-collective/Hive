"use strict";
/**
 * Git Decoration Provider for File Explorer
 * Provides Git status decorations for files and folders
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
exports.GitDecorationProvider = exports.GitStatus = void 0;
const events_1 = require("events");
const path = __importStar(require("path"));
// VS Code-style status codes
var GitStatus;
(function (GitStatus) {
    GitStatus["INDEX_MODIFIED"] = "M ";
    GitStatus["MODIFIED"] = " M";
    GitStatus["INDEX_ADDED"] = "A ";
    GitStatus["INDEX_DELETED"] = "D ";
    GitStatus["DELETED"] = " D";
    GitStatus["INDEX_RENAMED"] = "R ";
    GitStatus["INDEX_COPIED"] = "C ";
    GitStatus["UNTRACKED"] = "??";
    GitStatus["IGNORED"] = "!!";
    GitStatus["BOTH_DELETED"] = "DD";
    GitStatus["ADDED_BY_US"] = "AU";
    GitStatus["ADDED_BY_THEM"] = "UA";
    GitStatus["DELETED_BY_US"] = "DU";
    GitStatus["DELETED_BY_THEM"] = "UD";
    GitStatus["BOTH_ADDED"] = "AA";
    GitStatus["BOTH_MODIFIED"] = "UU";
})(GitStatus = exports.GitStatus || (exports.GitStatus = {}));
class GitDecorationProvider extends events_1.EventEmitter {
    constructor(rootPath) {
        super();
        this.decorations = new Map();
        this.updateTimer = null;
        this.gitStatus = new Map();
        this.updateInterval = null;
        // VS Code-inspired color scheme
        this.colors = {
            modified: '#e2c08d',
            added: '#73c991',
            deleted: '#f48771',
            untracked: '#6b6b6b',
            ignored: '#5a5a5a',
            conflicted: '#fd7e14',
            renamed: '#4fc3f7',
            staged: '#007acc' // VS Code blue
        };
        // Badge text for different statuses
        this.badges = {
            modified: 'M',
            added: 'A',
            deleted: 'D',
            untracked: 'U',
            renamed: 'R',
            copied: 'C',
            conflicted: '!',
            ignored: '',
            staged: '●'
        };
        this.rootPath = rootPath;
    }
    initialize() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[GitDecorationProvider] Initializing...');
            // Initial Git status load
            yield this.updateGitStatus();
            // Set up polling for Git status changes (since we can't use file watchers in renderer)
            // Poll every 2 seconds for changes
            this.updateInterval = setInterval(() => {
                this.scheduleUpdate();
            }, 2000);
            console.log('[GitDecorationProvider] Initialized with', this.decorations.size, 'decorations');
        });
    }
    scheduleUpdate() {
        // Debounce updates to avoid excessive Git calls
        if (this.updateTimer) {
            clearTimeout(this.updateTimer);
        }
        this.updateTimer = setTimeout(() => {
            this.updateGitStatus();
        }, 300);
    }
    updateGitStatus() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Get Git status from the main process
                const status = yield window.gitAPI.getStatus();
                if (!status.isRepo) {
                    console.log('[GitDecorationProvider] Not a Git repository');
                    this.decorations.clear();
                    this.emit('decorationsChanged', []);
                    return;
                }
                console.log('[GitDecorationProvider] Git status updated:', status.files.length, 'files');
                // Clear old decorations
                this.decorations.clear();
                this.gitStatus.clear();
                // Process each file status
                for (const file of status.files) {
                    this.gitStatus.set(file.path, file);
                    const decoration = this.createDecoration(file);
                    if (decoration) {
                        const fullPath = path.join(this.rootPath, file.path);
                        this.decorations.set(fullPath, decoration);
                        // Also add decorations for parent folders
                        if (decoration.propagate) {
                            this.propagateToParents(fullPath, decoration);
                        }
                    }
                }
                // Emit change event
                this.emit('decorationsChanged', Array.from(this.decorations.keys()));
            }
            catch (error) {
                console.error('[GitDecorationProvider] Failed to update Git status:', error);
            }
        });
    }
    createDecoration(fileStatus) {
        const statusCode = fileStatus.index + fileStatus.working;
        // Determine the decoration based on Git status
        let badge = '';
        let color = '';
        let tooltip = '';
        let priority = 0;
        let propagate = true;
        // Check for conflicts first (highest priority)
        if (statusCode === 'UU' || statusCode === 'AA' || statusCode === 'DD') {
            badge = this.badges.conflicted;
            color = this.colors.conflicted;
            tooltip = 'Merge conflict';
            priority = 100;
        }
        // Staged modifications
        else if (fileStatus.index === 'M') {
            badge = this.badges.modified;
            color = this.colors.staged;
            tooltip = 'Staged changes';
            priority = 80;
        }
        // Staged additions
        else if (fileStatus.index === 'A') {
            badge = this.badges.added;
            color = this.colors.added;
            tooltip = 'Staged new file';
            priority = 75;
        }
        // Staged deletions
        else if (fileStatus.index === 'D') {
            badge = this.badges.deleted;
            color = this.colors.deleted;
            tooltip = 'Staged for deletion';
            priority = 70;
        }
        // Renamed files
        else if (fileStatus.index === 'R') {
            badge = this.badges.renamed;
            color = this.colors.renamed;
            tooltip = `Renamed from ${fileStatus.renamed}`;
            priority = 65;
        }
        // Working tree modifications
        else if (fileStatus.working === 'M') {
            badge = this.badges.modified;
            color = this.colors.modified;
            tooltip = 'Modified';
            priority = 50;
        }
        // Working tree deletions
        else if (fileStatus.working === 'D') {
            badge = this.badges.deleted;
            color = this.colors.deleted;
            tooltip = 'Deleted';
            priority = 45;
        }
        // Untracked files
        else if (statusCode === '??') {
            badge = this.badges.untracked;
            color = this.colors.untracked;
            tooltip = 'Untracked';
            priority = 30;
            propagate = false; // Don't propagate untracked status to parents
        }
        // Ignored files
        else if (statusCode === '!!') {
            badge = this.badges.ignored;
            color = this.colors.ignored;
            tooltip = 'Ignored';
            priority = 10;
            propagate = false;
        }
        if (badge || color) {
            return {
                badge,
                color,
                tooltip,
                priority,
                propagate
            };
        }
        return null;
    }
    propagateToParents(filePath, decoration) {
        var _a;
        let currentPath = path.dirname(filePath);
        while (currentPath && currentPath !== this.rootPath && currentPath !== path.dirname(currentPath)) {
            const existingDecoration = this.decorations.get(currentPath);
            // Only update parent if this decoration has higher priority
            if (!existingDecoration || (decoration.priority || 0) > (existingDecoration.priority || 0)) {
                this.decorations.set(currentPath, Object.assign(Object.assign({}, decoration), { tooltip: 'Contains ' + ((_a = decoration.tooltip) === null || _a === void 0 ? void 0 : _a.toLowerCase()) }));
            }
            currentPath = path.dirname(currentPath);
        }
    }
    getDecoration(filePath) {
        return this.decorations.get(filePath);
    }
    getAllDecorations() {
        return new Map(this.decorations);
    }
    getFileStatus(filePath) {
        // Convert absolute path to relative path
        const relativePath = path.relative(this.rootPath, filePath);
        return this.gitStatus.get(relativePath);
    }
    refreshStatus() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[GitDecorationProvider] Manual refresh requested');
            yield this.updateGitStatus();
        });
    }
    dispose() {
        if (this.updateTimer) {
            clearTimeout(this.updateTimer);
        }
        if (this.updateInterval) {
            clearInterval(this.updateInterval);
        }
        this.decorations.clear();
        this.gitStatus.clear();
        this.removeAllListeners();
    }
}
exports.GitDecorationProvider = GitDecorationProvider;
//# sourceMappingURL=git-decoration-provider.js.map