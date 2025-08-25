"use strict";
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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.GitManager = void 0;
const simple_git_1 = __importDefault(require("simple-git"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const child_process_1 = require("child_process");
class GitManager {
    constructor(repoPath) {
        this.isRepo = false;
        // Use provided path or no path (for when no folder is open)
        this.repoPath = repoPath || '';
        if (this.repoPath) {
            // Configure simple-git with progress and completion handlers
            this.git = (0, simple_git_1.default)(this.repoPath, {
                progress(data) {
                    console.log('[GitManager] Git progress:', data);
                },
                // Ensure Git uses system credential helper
                config: [
                    'credential.helper=osxkeychain'
                ]
            });
            this.checkIfRepo();
            this.configureGit();
        }
        else {
            // No folder open - definitively not a repo
            this.isRepo = false;
            // Don't create a git instance when no folder is open
            this.git = null;
        }
    }
    configureGit() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Enable automatic upstream for new branches (Git 2.37+)
                yield this.git.addConfig('push.autoSetupRemote', 'true');
                console.log('[GitManager] Configured push.autoSetupRemote');
            }
            catch (error) {
                console.log('[GitManager] Could not set push.autoSetupRemote (may need newer Git):', error);
            }
        });
    }
    checkIfRepo() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const isRepo = yield this.git.checkIsRepo();
                this.isRepo = isRepo;
            }
            catch (error) {
                this.isRepo = false;
            }
        });
    }
    getStatus() {
        return __awaiter(this, void 0, void 0, function* () {
            // Return empty status if no folder is open or not a repo
            if (!this.repoPath || !this.isRepo || !this.git) {
                return {
                    files: [],
                    branch: '',
                    ahead: 0,
                    behind: 0,
                    isRepo: false,
                    repoPath: this.repoPath
                };
            }
            try {
                // Skip fetch on initial load - it's too slow and blocks the UI
                // We can fetch in the background later if needed
                // try {
                //   await this.git.fetch();
                // } catch (fetchError) {
                //   console.log('Fetch failed (may be offline):', fetchError);
                // }
                const status = yield this.git.status();
                const files = [];
                // Process all file statuses
                status.files.forEach(file => {
                    // Filter out submodules (dioxus-fork, src/hive_ui) - these are deprecated
                    // Only include files from electron-poc directory
                    if (file.path.startsWith('electron-poc/') ||
                        (!file.path.includes('dioxus-fork') && !file.path.includes('src/hive_ui'))) {
                        files.push({
                            path: file.path,
                            index: file.index || ' ',
                            working: file.working_dir || ' ',
                            renamed: file.rename || undefined
                        });
                    }
                });
                // Check if branch has tracking to determine ahead/behind
                let ahead = status.ahead || 0;
                let behind = status.behind || 0;
                let hasUpstream = false;
                // Check if current branch has upstream tracking
                if (status.current) {
                    const branches = yield this.git.branch(['-vv']);
                    const currentBranchInfo = branches.branches[status.current];
                    hasUpstream = currentBranchInfo && currentBranchInfo.tracking;
                    // If no upstream, count local commits as "ahead"
                    if (!hasUpstream) {
                        try {
                            // Count commits that would be pushed
                            const log = yield this.git.log(['origin/master..HEAD']);
                            ahead = log.total;
                        }
                        catch (e) {
                            // If we can't determine, assume we have commits to push
                            ahead = 1;
                        }
                    }
                }
                return {
                    files,
                    branch: status.current || 'master',
                    ahead,
                    behind,
                    isRepo: true,
                    repoPath: this.repoPath,
                    hasUpstream
                };
            }
            catch (error) {
                console.error('Git status error:', error);
                throw error;
            }
        });
    }
    getBranches() {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.repoPath || !this.isRepo || !this.git)
                return [];
            try {
                const summary = yield this.git.branchLocal();
                const branches = [];
                for (const [name, branch] of Object.entries(summary.branches)) {
                    branches.push({
                        name,
                        current: branch.current,
                        commit: branch.commit,
                        remote: branch.tracking || undefined
                    });
                }
                return branches;
            }
            catch (error) {
                console.error('Git branches error:', error);
                return [];
            }
        });
    }
    getDiff(file) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return '';
            try {
                let diff;
                if (file) {
                    diff = yield this.git.diff(['--', file]);
                }
                else {
                    diff = yield this.git.diff();
                }
                return diff;
            }
            catch (error) {
                console.error('Git diff error:', error);
                return '';
            }
        });
    }
    getStagedDiff(file) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return '';
            try {
                let diff;
                if (file) {
                    diff = yield this.git.diff(['--cached', '--', file]);
                }
                else {
                    diff = yield this.git.diff(['--cached']);
                }
                return diff;
            }
            catch (error) {
                console.error('Git staged diff error:', error);
                return '';
            }
        });
    }
    getLog(options = {}) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.repoPath || !this.isRepo || !this.git) {
                console.log('[GitManager] Not a repo or no folder open, returning empty log');
                return '';
            }
            try {
                // Use raw git command for more control over format
                const args = ['log'];
                const maxCount = options.maxCount || options.limit || 50;
                console.log('[GitManager] Using maxCount:', maxCount);
                args.push(`-${maxCount}`);
                // For now, skip graph decorations to simplify parsing
                // if (options.graph) {
                //   args.push('--graph');
                // }
                if (options.oneline) {
                    args.push('--oneline');
                }
                else {
                    // Use a simpler format with newlines between commits
                    args.push('--pretty=format:COMMIT_START|%H|%an|%ae|%ad|%s|COMMIT_END%n');
                }
                console.log('[GitManager] Git log args:', args);
                const result = yield this.git.raw(args);
                console.log('[GitManager] Git log result length:', result ? result.length : 0);
                return result || '';
            }
            catch (error) {
                console.error('[GitManager] Git log error:', error);
                return '';
            }
        });
    }
    stage(files) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                // Remove lock file if it exists
                const lockPath = path.join(this.repoPath, '.git', 'index.lock');
                if (fs.existsSync(lockPath)) {
                    fs.unlinkSync(lockPath);
                }
                yield this.git.add(files);
            }
            catch (error) {
                console.error('Git stage error:', error);
                throw error;
            }
        });
    }
    unstage(files) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                // Remove lock file if it exists
                const lockPath = path.join(this.repoPath, '.git', 'index.lock');
                if (fs.existsSync(lockPath)) {
                    fs.unlinkSync(lockPath);
                }
                yield this.git.reset(['HEAD', ...files]);
            }
            catch (error) {
                console.error('Git unstage error:', error);
                throw error;
            }
        });
    }
    commit(message) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                // Remove lock file if it exists
                const lockPath = path.join(this.repoPath, '.git', 'index.lock');
                if (fs.existsSync(lockPath)) {
                    fs.unlinkSync(lockPath);
                }
                yield this.git.commit(message);
            }
            catch (error) {
                console.error('Git commit error:', error);
                throw error;
            }
        });
    }
    discard(files) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                yield this.git.checkout(['--', ...files]);
            }
            catch (error) {
                console.error('Git discard error:', error);
                throw error;
            }
        });
    }
    clean(files) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                // Use git clean -f to remove untracked files
                // We specify each file explicitly for safety
                for (const file of files) {
                    yield this.git.clean('f', ['--', file]);
                }
            }
            catch (error) {
                console.error('Git clean error:', error);
                throw error;
            }
        });
    }
    pushWithSpawn(args) {
        return new Promise((resolve, reject) => {
            console.log(`[GitManager] Spawning: git push ${args.join(' ')}`);
            const gitProcess = (0, child_process_1.spawn)('git', ['push', ...args], {
                cwd: this.repoPath,
                env: Object.assign({}, process.env)
            });
            let output = '';
            let errorOutput = '';
            gitProcess.stdout.on('data', (data) => {
                const text = data.toString();
                output += text;
                console.log('[GitManager] Git stdout:', text);
            });
            gitProcess.stderr.on('data', (data) => {
                const text = data.toString();
                errorOutput += text;
                console.log('[GitManager] Git stderr:', text);
                // Git often sends progress to stderr, so don't treat all stderr as error
                if (text.includes('Enumerating') || text.includes('Counting') ||
                    text.includes('Compressing') || text.includes('Writing') ||
                    text.includes('Total') || text.includes('->')) {
                    // This is progress output, not an error
                    console.log('[GitManager] Git progress:', text);
                }
            });
            gitProcess.on('close', (code) => {
                console.log(`[GitManager] Git process exited with code ${code}`);
                if (code === 0) {
                    resolve(output + errorOutput);
                }
                else {
                    reject(new Error(`Git push failed with code ${code}: ${errorOutput}`));
                }
            });
            gitProcess.on('error', (err) => {
                console.error('[GitManager] Failed to spawn git process:', err);
                reject(err);
            });
        });
    }
    push() {
        var _a, _b;
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo) {
                console.log('[GitManager] Not a repo, cannot push');
                return;
            }
            try {
                console.log('[GitManager] Starting push operation...');
                // Get current branch status first
                const status = yield this.git.status();
                const currentBranch = status.current;
                if (!currentBranch) {
                    throw new Error('No current branch');
                }
                console.log(`[GitManager] Current branch: ${currentBranch}`);
                // First, try to fetch to ensure we have the latest remote info
                try {
                    yield this.git.fetch(['--prune']);
                    console.log('[GitManager] Fetched remote info');
                }
                catch (fetchError) {
                    console.log('[GitManager] Fetch failed (may be offline):', fetchError);
                }
                // Check if branch has upstream
                const branches = yield this.git.branch(['-vv']);
                const currentBranchInfo = branches.branches[currentBranch];
                const hasUpstream = currentBranchInfo && currentBranchInfo.tracking;
                console.log(`[GitManager] Has upstream: ${hasUpstream}`);
                if (!hasUpstream) {
                    console.log(`[GitManager] No upstream for ${currentBranch}, pushing with --set-upstream...`);
                    // Try using spawn for better control and to avoid hanging
                    const result = yield this.pushWithSpawn(['--set-upstream', 'origin', currentBranch]);
                    console.log('[GitManager] Push with upstream result:', result);
                    console.log('[GitManager] Successfully pushed with upstream set');
                }
                else {
                    // Regular push using spawn
                    console.log('[GitManager] Performing regular push...');
                    const result = yield this.pushWithSpawn(['origin', currentBranch]);
                    console.log('[GitManager] Push result:', result);
                    console.log('[GitManager] Successfully pushed');
                }
            }
            catch (error) {
                console.error('[GitManager] Git push error:', error);
                console.error('[GitManager] Error message:', error === null || error === void 0 ? void 0 : error.message);
                console.error('[GitManager] Error stack:', error === null || error === void 0 ? void 0 : error.stack);
                // Check if it's an authentication error
                if (((_a = error === null || error === void 0 ? void 0 : error.message) === null || _a === void 0 ? void 0 : _a.includes('Authentication')) || ((_b = error === null || error === void 0 ? void 0 : error.message) === null || _b === void 0 ? void 0 : _b.includes('could not read Username'))) {
                    throw new Error('Git authentication required. Please ensure your Git credentials are configured.');
                }
                throw error;
            }
        });
    }
    pull() {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                // Get current branch status
                const status = yield this.git.status();
                const currentBranch = status.current;
                if (!currentBranch) {
                    throw new Error('No current branch');
                }
                // Check if branch has upstream
                const branches = yield this.git.branch(['-vv']);
                const currentBranchInfo = branches.branches[currentBranch];
                const hasUpstream = currentBranchInfo && currentBranchInfo.tracking;
                if (!hasUpstream) {
                    console.log(`No upstream for ${currentBranch}, setting upstream first...`);
                    // Set upstream to track origin/branch
                    yield this.git.branch(['--set-upstream-to', `origin/${currentBranch}`, currentBranch]);
                    console.log('Upstream set, now pulling...');
                }
                // Now pull
                yield this.git.pull();
                console.log('Successfully pulled');
            }
            catch (error) {
                // If pull fails because remote branch doesn't exist, that's okay
                if (error.message && error.message.includes('no such ref was fetched')) {
                    console.log('Remote branch does not exist yet - nothing to pull');
                }
                else {
                    console.error('Git pull error:', error);
                    throw error;
                }
            }
        });
    }
    fetch() {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                yield this.git.fetch();
            }
            catch (error) {
                console.error('Git fetch error:', error);
                throw error;
            }
        });
    }
    switchBranch(branchName) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                yield this.git.checkout(branchName);
            }
            catch (error) {
                console.error('Git switch branch error:', error);
                throw error;
            }
        });
    }
    createBranch(branchName) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return;
            try {
                yield this.git.checkoutLocalBranch(branchName);
            }
            catch (error) {
                console.error('Git create branch error:', error);
                throw error;
            }
        });
    }
    getRepoPath() {
        return this.repoPath;
    }
    getIsRepo() {
        return this.isRepo;
    }
    initRepo() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                yield this.git.init();
                this.isRepo = true;
                console.log('Git repository initialized at:', this.repoPath);
            }
            catch (error) {
                console.error('Failed to initialize Git repository:', error);
                throw error;
            }
        });
    }
    getCommitFiles(hash) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return { files: [] };
            try {
                // Get the list of files changed in this commit
                const result = yield this.git.raw(['show', '--name-status', '--format=', hash]);
                const lines = result.split('\n').filter(line => line.trim());
                const files = lines.map(line => {
                    const parts = line.split('\t');
                    if (parts.length >= 2) {
                        return {
                            status: parts[0],
                            path: parts[1],
                            additions: 0,
                            deletions: 0
                        };
                    }
                    return null;
                }).filter(f => f);
                return { files };
            }
            catch (error) {
                console.error('Failed to get commit files:', error);
                return { files: [] };
            }
        });
    }
    getFileDiff(commitHash, filePath) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.isRepo)
                return '';
            try {
                // Get the diff for a specific file in a commit
                // Use proper diff format with unified context
                const result = yield this.git.raw(['diff', `${commitHash}^..${commitHash}`, '--', filePath]);
                // If the file was added in this commit (no parent), show the full file as added
                if (!result || result.trim() === '') {
                    const fileContent = yield this.git.raw(['show', `${commitHash}:${filePath}`]);
                    if (fileContent) {
                        // Format as an addition diff
                        const lines = fileContent.split('\n');
                        const diff = `diff --git a/${filePath} b/${filePath}
new file mode 100644
index 0000000..0000000
--- /dev/null
+++ b/${filePath}
${lines.map(line => '+' + line).join('\n')}`;
                        return diff;
                    }
                }
                return result || '';
            }
            catch (error) {
                // Try alternative method for first commit or added files
                try {
                    const fileContent = yield this.git.raw(['show', `${commitHash}:${filePath}`]);
                    if (fileContent) {
                        const lines = fileContent.split('\n');
                        const diff = `diff --git a/${filePath} b/${filePath}
new file mode 100644
index 0000000..0000000
--- /dev/null
+++ b/${filePath}
${lines.map(line => '+' + line).join('\n')}`;
                        return diff;
                    }
                }
                catch (innerError) {
                    console.error('Failed to get file diff:', innerError);
                }
                return '';
            }
        });
    }
}
exports.GitManager = GitManager;
//# sourceMappingURL=git-manager.js.map