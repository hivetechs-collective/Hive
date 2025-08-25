"use strict";
/**
 * Enhanced Git Manager
 * Integrates the authentication system with Git operations
 * Enterprise-class Git management based on VS Code's architecture
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
exports.EnhancedGitManager = void 0;
const simpleGit = __importStar(require("simple-git"));
const GitAuthenticationManager_1 = require("./authentication/GitAuthenticationManager");
const path = __importStar(require("path"));
class EnhancedGitManager {
    constructor(repoPath) {
        this.isInitialized = false;
        this.repoPath = repoPath;
        this.git = simpleGit.simpleGit(repoPath);
        // Use singleton authentication manager to avoid duplicate IPC handlers
        if (!EnhancedGitManager.authManager) {
            EnhancedGitManager.authManager = new GitAuthenticationManager_1.GitAuthenticationManager({
                enableCache: true,
                cacheDuration: 300000,
                useSystemCredentialManager: true,
                enableOAuth: true,
            });
        }
        console.log('[EnhancedGitManager] Initialized for:', repoPath);
    }
    /**
     * Initialize the Git manager
     */
    initialize() {
        return __awaiter(this, void 0, void 0, function* () {
            if (this.isInitialized)
                return;
            console.log('[EnhancedGitManager] Initializing...');
            // Initialize authentication manager
            yield EnhancedGitManager.authManager.initialize();
            // Configure Git settings
            yield this.configureGit();
            this.isInitialized = true;
            console.log('[EnhancedGitManager] Initialization complete');
        });
    }
    /**
     * Configure Git settings
     */
    configureGit() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Set push.autoSetupRemote for easier branch pushing
                yield this.git.addConfig('push.autoSetupRemote', 'true');
                // Ensure credential helper is not interfering
                // We'll handle credentials ourselves
                yield this.git.addConfig('credential.helper', '');
                console.log('[EnhancedGitManager] Git configured');
            }
            catch (error) {
                console.error('[EnhancedGitManager] Error configuring Git:', error);
            }
        });
    }
    /**
     * Get repository status
     */
    getStatus() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // First check if we're actually in a Git repository
                const isRepo = yield this.git.checkIsRepo();
                if (!isRepo) {
                    // Not in a Git repository, return null to show welcome screen
                    console.log('[EnhancedGitManager] Not in a Git repository');
                    return null;
                }
                const status = yield this.git.status();
                // Convert to plain object to avoid serialization issues in IPC
                return {
                    isRepo: true,
                    current: status.current,
                    tracking: status.tracking,
                    ahead: status.ahead,
                    behind: status.behind,
                    files: status.files,
                    staged: status.staged,
                    renamed: status.renamed,
                    deleted: status.deleted,
                    modified: status.modified,
                    created: status.created,
                    conflicted: status.conflicted,
                    isClean: status.isClean(),
                    branch: status.current || 'master',
                    hasUpstream: !!status.tracking
                };
            }
            catch (error) {
                console.error('[EnhancedGitManager] Error getting status:', error);
                // If there's an error, we're probably not in a repo
                return null;
            }
        });
    }
    /**
     * Stage files
     */
    stage(files) {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.git.add(files);
        });
    }
    /**
     * Unstage files
     */
    unstage(files) {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.git.reset(['HEAD', ...files]);
        });
    }
    /**
     * Commit changes
     */
    commit(message) {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.git.commit(message);
        });
    }
    /**
     * Push changes with authentication support
     */
    push() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[EnhancedGitManager] Starting push operation...');
            try {
                // Ensure we're initialized
                yield this.initialize();
                // Get current branch
                const status = yield this.git.status();
                const branch = status.current;
                if (!branch) {
                    return {
                        success: false,
                        error: 'No current branch'
                    };
                }
                console.log(`[EnhancedGitManager] Pushing branch: ${branch}`);
                // Check if branch has upstream
                const branches = yield this.git.branch(['-vv']);
                const currentBranchInfo = branches.branches[branch];
                const hasUpstream = currentBranchInfo && currentBranchInfo.tracking;
                // Prepare push arguments
                const args = hasUpstream
                    ? ['push', 'origin', branch]
                    : ['push', '--set-upstream', 'origin', branch];
                console.log(`[EnhancedGitManager] Push command: git ${args.join(' ')}`);
                // Execute with authentication
                const result = yield EnhancedGitManager.authManager.executeGitCommand(args, this.repoPath);
                if (result.code === 0) {
                    console.log('[EnhancedGitManager] Push successful');
                    return {
                        success: true,
                        output: result.stdout || result.stderr
                    };
                }
                else {
                    console.error('[EnhancedGitManager] Push failed:', result.stderr);
                    return {
                        success: false,
                        error: result.stderr || 'Push failed'
                    };
                }
            }
            catch (error) {
                console.error('[EnhancedGitManager] Push error:', error);
                return {
                    success: false,
                    error: error.message || 'Unknown error'
                };
            }
        });
    }
    /**
     * Pull changes with authentication support
     */
    pull() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[EnhancedGitManager] Starting pull operation...');
            try {
                // Ensure we're initialized
                yield this.initialize();
                // Get current branch
                const status = yield this.git.status();
                const branch = status.current;
                if (!branch) {
                    return {
                        success: false,
                        error: 'No current branch'
                    };
                }
                console.log(`[EnhancedGitManager] Pulling branch: ${branch}`);
                // Execute with authentication
                const result = yield EnhancedGitManager.authManager.executeGitCommand(['pull', 'origin', branch], this.repoPath);
                if (result.code === 0) {
                    console.log('[EnhancedGitManager] Pull successful');
                    return {
                        success: true,
                        output: result.stdout || result.stderr
                    };
                }
                else {
                    console.error('[EnhancedGitManager] Pull failed:', result.stderr);
                    return {
                        success: false,
                        error: result.stderr || 'Pull failed'
                    };
                }
            }
            catch (error) {
                console.error('[EnhancedGitManager] Pull error:', error);
                return {
                    success: false,
                    error: error.message || 'Unknown error'
                };
            }
        });
    }
    /**
     * Sync (pull then push) with authentication support
     */
    sync() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[EnhancedGitManager] Starting sync operation...');
            // First pull
            const pullResult = yield this.pull();
            if (!pullResult.success) {
                return pullResult;
            }
            // Then push
            const pushResult = yield this.push();
            return pushResult;
        });
    }
    /**
     * Fetch remote changes
     */
    fetch() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[EnhancedGitManager] Starting fetch operation...');
            try {
                // Ensure we're initialized
                yield this.initialize();
                // Execute with authentication
                const result = yield EnhancedGitManager.authManager.executeGitCommand(['fetch', '--all', '--prune'], this.repoPath);
                if (result.code === 0) {
                    console.log('[EnhancedGitManager] Fetch successful');
                    return {
                        success: true,
                        output: result.stdout || result.stderr
                    };
                }
                else {
                    console.error('[EnhancedGitManager] Fetch failed:', result.stderr);
                    return {
                        success: false,
                        error: result.stderr || 'Fetch failed'
                    };
                }
            }
            catch (error) {
                console.error('[EnhancedGitManager] Fetch error:', error);
                return {
                    success: false,
                    error: error.message || 'Unknown error'
                };
            }
        });
    }
    /**
     * Clone a repository with authentication support
     */
    clone(url, destination) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log(`[EnhancedGitManager] Cloning ${url} to ${destination}`);
            try {
                // Ensure we're initialized
                yield this.initialize();
                // Execute with authentication
                const result = yield EnhancedGitManager.authManager.executeGitCommand(['clone', url, destination], path.dirname(destination));
                if (result.code === 0) {
                    console.log('[EnhancedGitManager] Clone successful');
                    return {
                        success: true,
                        output: result.stdout || result.stderr
                    };
                }
                else {
                    console.error('[EnhancedGitManager] Clone failed:', result.stderr);
                    return {
                        success: false,
                        error: result.stderr || 'Clone failed'
                    };
                }
            }
            catch (error) {
                console.error('[EnhancedGitManager] Clone error:', error);
                return {
                    success: false,
                    error: error.message || 'Unknown error'
                };
            }
        });
    }
    /**
     * Get commit log
     */
    getLog(options = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            const args = ['log'];
            if (options.graph) {
                args.push('--graph');
            }
            if (options.maxCount) {
                args.push(`-${options.maxCount}`);
            }
            // Use markers to help parsing
            args.push('--pretty=format:COMMIT_START|%H|%an|%ae|%ad|%s|COMMIT_END');
            const result = yield this.git.raw(args);
            return result;
        });
    }
    /**
     * Get branches
     */
    getBranches() {
        return __awaiter(this, void 0, void 0, function* () {
            const branches = yield this.git.branch();
            // Convert to plain object to avoid serialization issues
            return {
                all: branches.all,
                branches: branches.branches,
                current: branches.current,
                detached: branches.detached
            };
        });
    }
    /**
     * Switch branch
     */
    switchBranch(branchName) {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.git.checkout(branchName);
        });
    }
    /**
     * Create new branch
     */
    createBranch(branchName) {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.git.checkoutLocalBranch(branchName);
        });
    }
    /**
     * Get diff
     */
    getDiff(file) {
        return __awaiter(this, void 0, void 0, function* () {
            if (file) {
                return yield this.git.diff(['--', file]);
            }
            return yield this.git.diff();
        });
    }
    /**
     * Get staged diff
     */
    getStagedDiff(file) {
        return __awaiter(this, void 0, void 0, function* () {
            if (file) {
                return yield this.git.diff(['--cached', '--', file]);
            }
            return yield this.git.diff(['--cached']);
        });
    }
    /**
     * Discard changes
     */
    discard(files) {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.git.checkout(['--', ...files]);
        });
    }
    /**
     * Remove untracked files
     */
    clean(files) {
        return __awaiter(this, void 0, void 0, function* () {
            // Use git clean -f to remove untracked files
            // We specify each file explicitly for safety
            for (const file of files) {
                yield this.git.clean('f', ['--', file]);
            }
        });
    }
    /**
     * Initialize Git repository
     */
    initRepo() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.git.init();
        });
    }
    /**
     * Get files changed in a commit
     */
    getCommitFiles(hash) {
        return __awaiter(this, void 0, void 0, function* () {
            const result = yield this.git.raw(['diff-tree', '--no-commit-id', '--name-only', '-r', hash]);
            return result.split('\n').filter(file => file.length > 0);
        });
    }
    /**
     * Get diff for a specific file in a commit
     */
    getFileDiff(commitHash, filePath) {
        return __awaiter(this, void 0, void 0, function* () {
            // Get the diff of the file between the commit and its parent
            return yield this.git.raw(['diff', `${commitHash}~1..${commitHash}`, '--', filePath]);
        });
    }
    /**
     * Get remote info
     */
    getRemoteInfo() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const status = yield this.git.status();
                return {
                    ahead: status.ahead || 0,
                    behind: status.behind || 0
                };
            }
            catch (_a) {
                return { ahead: 0, behind: 0 };
            }
        });
    }
    /**
     * Clean up resources
     */
    dispose() {
        // Don't dispose the singleton auth manager
        // It will be reused across folder changes
    }
}
exports.EnhancedGitManager = EnhancedGitManager;
EnhancedGitManager.authManager = null;
//# sourceMappingURL=EnhancedGitManager.js.map