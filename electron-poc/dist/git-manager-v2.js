"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GitManagerV2 = void 0;
const git_executor_1 = require("./git-executor");
const git_operation_queue_1 = require("./git-operation-queue");
const path = require("path");
const fs = require("fs");
class GitManagerV2 {
    constructor(repoPath) {
        this.isRepo = false;
        this.cachedStatus = null;
        this.statusCacheTime = 0;
        this.CACHE_DURATION = 500; // 500ms cache for status
        this.repoPath = repoPath || '';
        this.queue = new git_operation_queue_1.GitOperationQueue();
        if (this.repoPath) {
            this.executor = new git_executor_1.GitExecutor(this.repoPath);
            this.checkIfRepo();
            this.configureGit();
        }
        else {
            this.isRepo = false;
            this.executor = null;
        }
    }
    async checkIfRepo() {
        try {
            await this.executor.exec(['rev-parse', '--git-dir']);
            this.isRepo = true;
            console.log('[GitManagerV2] Repository detected at:', this.repoPath);
        }
        catch (error) {
            this.isRepo = false;
            console.log('[GitManagerV2] Not a Git repository:', this.repoPath);
        }
    }
    async configureGit() {
        try {
            // Enable automatic upstream for new branches (Git 2.37+)
            await this.executor.exec(['config', 'push.autoSetupRemote', 'true']);
            console.log('[GitManagerV2] Configured push.autoSetupRemote');
            // Ensure credential helper is set for macOS
            if (process.platform === 'darwin') {
                await this.executor.exec(['config', 'credential.helper', 'osxkeychain']);
                console.log('[GitManagerV2] Configured macOS credential helper');
            }
        }
        catch (error) {
            console.log('[GitManagerV2] Could not configure Git settings:', error);
        }
    }
    async getStatus() {
        // Return cached status if still fresh
        if (this.cachedStatus && Date.now() - this.statusCacheTime < this.CACHE_DURATION) {
            return this.cachedStatus;
        }
        // Return empty status if no folder is open or not a repo
        if (!this.repoPath || !this.isRepo || !this.executor) {
            return {
                files: [],
                branch: '',
                ahead: 0,
                behind: 0,
                isRepo: false,
                repoPath: this.repoPath,
            };
        }
        // Use priority execution for status (doesn't wait in queue)
        return await this.queue.executePriority(async () => {
            try {
                console.log('[GitManagerV2] Getting status...');
                // Fetch first to get accurate ahead/behind counts (but don't block on it)
                this.executor.fetch({ prune: true }).catch(err => console.log('[GitManagerV2] Background fetch failed (may be offline):', err));
                const status = await this.executor.status();
                const result = {
                    files: status.files || [],
                    branch: status.branch || '',
                    ahead: status.ahead || 0,
                    behind: status.behind || 0,
                    isRepo: true,
                    repoPath: this.repoPath,
                    hasUpstream: !!status.upstream,
                    upstream: status.upstream,
                };
                // If no upstream but we have local commits, check if we have commits to push
                if (!result.hasUpstream && result.branch) {
                    try {
                        // Count commits ahead of origin/master (or origin/main)
                        let baseRef = 'origin/master';
                        try {
                            // Check if origin/master exists
                            await this.executor.exec(['rev-parse', '--verify', 'origin/master']);
                        }
                        catch {
                            // Try origin/main instead
                            try {
                                await this.executor.exec(['rev-parse', '--verify', 'origin/main']);
                                baseRef = 'origin/main';
                            }
                            catch {
                                // No base branch found, can't count ahead
                                console.log(`[GitManagerV2] No base branch found for comparison`);
                                return result;
                            }
                        }
                        // Count commits ahead of base branch
                        const logResult = await this.executor.exec(['log', '--oneline', `${baseRef}..HEAD`]);
                        const commits = logResult.stdout.trim().split('\n').filter(l => l);
                        result.ahead = commits.length;
                        console.log(`[GitManagerV2] Branch ${result.branch} has ${result.ahead} commits ahead of ${baseRef}`);
                    }
                    catch (error) {
                        console.log('[GitManagerV2] Could not count unpushed commits:', error);
                    }
                }
                // Cache the result
                this.cachedStatus = result;
                this.statusCacheTime = Date.now();
                return result;
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to get status:', error);
                return {
                    files: [],
                    branch: '',
                    ahead: 0,
                    behind: 0,
                    isRepo: false,
                    repoPath: this.repoPath,
                };
            }
        });
    }
    async stage(files) {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('stage', async () => {
            try {
                // Clear any lock file first
                this.clearLockFile();
                await this.executor.add(files);
                console.log('[GitManagerV2] Staged files:', files);
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to stage files:', error);
                throw error;
            }
        });
    }
    async unstage(files) {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('unstage', async () => {
            try {
                // Clear any lock file first
                this.clearLockFile();
                await this.executor.reset(files);
                console.log('[GitManagerV2] Unstaged files:', files);
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to unstage files:', error);
                throw error;
            }
        });
    }
    async commit(message) {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('commit', async () => {
            try {
                // Clear any lock file first
                this.clearLockFile();
                await this.executor.commit(message);
                console.log('[GitManagerV2] Committed with message:', message);
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to commit:', error);
                throw error;
            }
        });
    }
    async push() {
        if (!this.isRepo) {
            console.log('[GitManagerV2] Not a repo, cannot push');
            return;
        }
        return this.queue.enqueue('push', async () => {
            try {
                console.log('[GitManagerV2] Starting push operation...');
                // Get current branch and upstream info
                const status = await this.getStatus();
                const branch = status.branch;
                if (!branch) {
                    throw new Error('No current branch');
                }
                console.log(`[GitManagerV2] Pushing branch: ${branch}, upstream: ${status.upstream}, ahead: ${status.ahead}`);
                // Determine if we need to set upstream
                const needsUpstream = !status.hasUpstream;
                if (needsUpstream) {
                    console.log(`[GitManagerV2] Setting upstream for ${branch}`);
                    await this.executor.push({
                        remote: 'origin',
                        branch: branch,
                        setUpstream: true,
                    });
                    console.log('[GitManagerV2] Successfully pushed with upstream set');
                }
                else {
                    console.log('[GitManagerV2] Performing regular push');
                    await this.executor.push({
                        remote: 'origin',
                        branch: branch,
                    });
                    console.log('[GitManagerV2] Successfully pushed');
                }
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Push failed:', error);
                // Handle specific error codes
                if (error instanceof git_executor_1.GitError) {
                    switch (error.data.gitErrorCode) {
                        case git_executor_1.GitErrorCode.AuthenticationFailed:
                            throw new Error('Git authentication failed. Please check your credentials.');
                        case git_executor_1.GitErrorCode.PushRejected:
                            throw new Error('Push was rejected. You may need to pull first.');
                        case git_executor_1.GitErrorCode.RemoteConnectionError:
                            throw new Error('Could not connect to remote repository.');
                        case git_executor_1.GitErrorCode.NoUpstreamBranch:
                            // This shouldn't happen as we handle it, but just in case
                            throw new Error('No upstream branch configured.');
                        default:
                            throw new Error(error.data.message || 'Push failed');
                    }
                }
                throw error;
            }
        });
    }
    async pull() {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('pull', async () => {
            try {
                console.log('[GitManagerV2] Starting pull operation...');
                // Get current branch info
                const status = await this.getStatus();
                const branch = status.branch;
                if (!branch) {
                    throw new Error('No current branch');
                }
                // Check if we have an upstream
                if (!status.hasUpstream) {
                    console.log(`[GitManagerV2] No upstream for ${branch}, attempting to set...`);
                    // Try to set upstream to origin/branch
                    try {
                        await this.executor.exec(['branch', '--set-upstream-to', `origin/${branch}`, branch]);
                        console.log('[GitManagerV2] Upstream set, now pulling...');
                    }
                    catch (error) {
                        console.log('[GitManagerV2] Could not set upstream (remote branch may not exist)');
                        throw new Error('No remote branch to pull from');
                    }
                }
                // Perform the pull
                await this.executor.pull({
                    remote: 'origin',
                    branch: branch,
                });
                console.log('[GitManagerV2] Successfully pulled');
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Pull failed:', error);
                // Handle specific error codes
                if (error instanceof git_executor_1.GitError) {
                    switch (error.data.gitErrorCode) {
                        case git_executor_1.GitErrorCode.AuthenticationFailed:
                            throw new Error('Git authentication failed. Please check your credentials.');
                        case git_executor_1.GitErrorCode.Conflict:
                            throw new Error('Merge conflicts detected. Please resolve conflicts manually.');
                        case git_executor_1.GitErrorCode.DirtyWorkTree:
                            throw new Error('You have uncommitted changes. Please commit or stash them first.');
                        case git_executor_1.GitErrorCode.RemoteConnectionError:
                            throw new Error('Could not connect to remote repository.');
                        default:
                            throw new Error(error.data.message || 'Pull failed');
                    }
                }
                throw error;
            }
        });
    }
    async sync() {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('sync', async () => {
            try {
                console.log('[GitManagerV2] Starting sync operation (pull then push)...');
                // First pull
                await this.pull();
                // Then push
                await this.push();
                console.log('[GitManagerV2] Successfully synced');
            }
            catch (error) {
                console.error('[GitManagerV2] Sync failed:', error);
                throw error;
            }
        });
    }
    async fetch() {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('fetch', async () => {
            try {
                await this.executor.fetch({ all: true, prune: true });
                console.log('[GitManagerV2] Successfully fetched');
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Fetch failed:', error);
                throw error;
            }
        });
    }
    async getBranches() {
        if (!this.isRepo)
            return [];
        try {
            const branches = await this.executor.branch({ all: true });
            return branches;
        }
        catch (error) {
            console.error('[GitManagerV2] Failed to get branches:', error);
            return [];
        }
    }
    async switchBranch(branchName) {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('checkout', async () => {
            try {
                await this.executor.checkout(branchName);
                console.log('[GitManagerV2] Switched to branch:', branchName);
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to switch branch:', error);
                throw error;
            }
        });
    }
    async createBranch(branchName) {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('branch', async () => {
            try {
                await this.executor.exec(['checkout', '-b', branchName]);
                console.log('[GitManagerV2] Created and switched to branch:', branchName);
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to create branch:', error);
                throw error;
            }
        });
    }
    async getLog(options = {}) {
        if (!this.isRepo)
            return '';
        try {
            // Pass all options including graph
            return await this.executor.log(options);
        }
        catch (error) {
            console.error('[GitManagerV2] Failed to get log:', error);
            return '';
        }
    }
    async getDiff(file) {
        if (!this.isRepo)
            return '';
        try {
            const args = ['diff'];
            if (file) {
                args.push('--', file);
            }
            const result = await this.executor.exec(args);
            return result.stdout;
        }
        catch (error) {
            console.error('[GitManagerV2] Failed to get diff:', error);
            return '';
        }
    }
    async getStagedDiff(file) {
        if (!this.isRepo)
            return '';
        try {
            const args = ['diff', '--cached'];
            if (file) {
                args.push('--', file);
            }
            const result = await this.executor.exec(args);
            return result.stdout;
        }
        catch (error) {
            console.error('[GitManagerV2] Failed to get staged diff:', error);
            return '';
        }
    }
    async discard(files) {
        if (!this.isRepo)
            return;
        return this.queue.enqueue('checkout', async () => {
            try {
                await this.executor.exec(['checkout', '--', ...files]);
                console.log('[GitManagerV2] Discarded changes in files:', files);
                // Invalidate status cache
                this.cachedStatus = null;
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to discard changes:', error);
                throw error;
            }
        });
    }
    async initRepo() {
        try {
            await this.executor.exec(['init']);
            this.isRepo = true;
            console.log('[GitManagerV2] Initialized repository');
        }
        catch (error) {
            console.error('[GitManagerV2] Failed to initialize repository:', error);
            throw error;
        }
    }
    async getCommitFiles(hash) {
        if (!this.isRepo)
            return [];
        try {
            const result = await this.executor.exec(['show', '--name-only', '--pretty=format:', hash]);
            return result.stdout.trim().split('\n').filter(f => f);
        }
        catch (error) {
            console.error('[GitManagerV2] Failed to get commit files:', error);
            return [];
        }
    }
    async getFileDiff(commitHash, filePath) {
        if (!this.isRepo)
            return '';
        try {
            const result = await this.executor.exec(['show', `${commitHash}:${filePath}`]);
            return result.stdout;
        }
        catch (error) {
            console.error('[GitManagerV2] Failed to get file diff:', error);
            return '';
        }
    }
    clearLockFile() {
        if (!this.repoPath)
            return;
        const lockPath = path.join(this.repoPath, '.git', 'index.lock');
        if (fs.existsSync(lockPath)) {
            try {
                fs.unlinkSync(lockPath);
                console.log('[GitManagerV2] Cleared lock file');
            }
            catch (error) {
                console.error('[GitManagerV2] Failed to clear lock file:', error);
            }
        }
    }
}
exports.GitManagerV2 = GitManagerV2;
