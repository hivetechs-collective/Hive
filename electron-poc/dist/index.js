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
Object.defineProperty(exports, "__esModule", { value: true });
const electron_1 = require("electron");
const git_chunked_push_main_1 = require("./git-chunked-push-main");
// Set the app name immediately
electron_1.app.setName('Hive Consensus');
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const crypto = __importStar(require("crypto"));
const sqlite3_1 = require("sqlite3");
const git_manager_1 = require("./git-manager");
const git_manager_v2_1 = require("./git-manager-v2");
const EnhancedGitManager_1 = require("./git/EnhancedGitManager");
const file_system_1 = require("./file-system");
const ProcessManager_1 = require("./utils/ProcessManager");
const detector_1 = require("./main/cli-tools/detector");
const cli_tools_1 = require("./shared/types/cli-tools");
// Removed import - functions are now defined locally
const SafeLogger_1 = require("./utils/SafeLogger");
const terminal_ipc_handlers_1 = require("./terminal-ipc-handlers");
const StartupOrchestrator_1 = require("./startup/StartupOrchestrator");
const AIToolsDatabase_1 = require("./services/AIToolsDatabase");
// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
    electron_1.app.quit();
}
// Single source of truth for all process and port management
// Initialize early so it's available for all components
const processManager = new ProcessManager_1.ProcessManager();
let db = null;
let mainWindow = null;
// Initialize SQLite database connection - use the existing hive-ai.db
const initDatabase = () => {
    // Use the actual Hive database location
    const dbPath = path.join(os.homedir(), '.hive', 'hive-ai.db');
    // Create .hive directory if it doesn't exist
    const hiveDir = path.join(os.homedir(), '.hive');
    if (!fs.existsSync(hiveDir)) {
        fs.mkdirSync(hiveDir, { recursive: true });
    }
    db = new sqlite3_1.Database(dbPath);
    // The database already exists with proper schema
    // Just ensure the configurations table exists (matching Rust implementation)
    db.run(`CREATE TABLE IF NOT EXISTS configurations (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    encrypted BOOLEAN DEFAULT 0,
    user_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
  )`);
    // Ensure users table exists with default user
    db.run(`CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT,
    tier TEXT
  )`);
    // Insert default user if not exists
    db.run(`INSERT OR IGNORE INTO users (id, email, tier) VALUES ('default', 'default@hive.ai', 'FREE')`);
    // Create consensus_settings table for active profile
    db.run(`CREATE TABLE IF NOT EXISTS consensus_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
  )`);
    // Create stage_outputs table to track model usage per stage
    db.run(`CREATE TABLE IF NOT EXISTS stage_outputs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    stage_name TEXT NOT NULL,
    model TEXT NOT NULL,
    tokens_used INTEGER DEFAULT 0,
    cost REAL DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
  )`);
    // Keep consensus profiles table matching actual database schema
    db.run(`CREATE TABLE IF NOT EXISTS consensus_profiles (
    id TEXT PRIMARY KEY,
    profile_name TEXT NOT NULL,
    generator_model TEXT NOT NULL,
    refiner_model TEXT NOT NULL,
    validator_model TEXT NOT NULL,
    curator_model TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
  )`);
};
// Toggle to switch between implementations
// 0 = old GitManager, 1 = GitManagerV2, 2 = EnhancedGitManager with Auth
const GIT_MANAGER_VERSION = 2; // Use EnhancedGitManager with authentication
// Git Integration
let gitManager = null;
// Initialize Git manager - pass no path when no folder is open
const initGitManager = (folderPath) => __awaiter(void 0, void 0, void 0, function* () {
    if (!folderPath) {
        SafeLogger_1.logger.info('[Main] No folder path provided, GitManager will return null status');
        // Don't create a manager when no folder is open
        gitManager = null;
        return;
    }
    if (GIT_MANAGER_VERSION === 2) {
        SafeLogger_1.logger.info('[Main] Using EnhancedGitManager with authentication support for:', folderPath);
        gitManager = new EnhancedGitManager_1.EnhancedGitManager(folderPath);
        yield gitManager.initialize();
    }
    else if (GIT_MANAGER_VERSION === 1) {
        SafeLogger_1.logger.info('[Main] Using GitManagerV2 with VS Code-style implementation');
        gitManager = new git_manager_v2_1.GitManagerV2(folderPath);
    }
    else {
        SafeLogger_1.logger.info('[Main] Using old GitManager with simple-git');
        gitManager = new git_manager_1.GitManager(folderPath);
    }
});
// File System Manager
let fileSystemManager = null;
// Initialize File System manager
const initFileSystemManager = () => {
    fileSystemManager = new file_system_1.FileSystemManager();
};
const createWindow = (show = true) => {
    // Don't create duplicate windows
    if (mainWindow && !mainWindow.isDestroyed()) {
        SafeLogger_1.logger.info('[Main] Window already exists, focusing it');
        if (show)
            mainWindow.focus();
        return mainWindow;
    }
    // Create the browser window.
    mainWindow = new electron_1.BrowserWindow({
        height: 600,
        show: false,
        width: 800,
        minWidth: 700,
        minHeight: 400,
        title: 'Hive Consensus',
        icon: path.join(__dirname, '../resources/icon.png'),
        webPreferences: {
            preload: MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY,
            nodeIntegration: false,
            contextIsolation: true,
            webSecurity: false,
            webviewTag: true, // Enable webview tags for ttyd terminals
        },
    });
    // and load the index.html of the app.
    mainWindow.loadURL(MAIN_WINDOW_WEBPACK_ENTRY);
    // Open the DevTools.
    // mainWindow.webContents.openDevTools(); // Disabled to prevent warning overlay
    // Register terminal handlers with the shared ProcessManager
    // This is safe to call multiple times as it updates the window reference
    (0, terminal_ipc_handlers_1.registerTerminalHandlers)(mainWindow, processManager);
    // Create application menu
    createApplicationMenu();
    // Show window if requested
    if (show && mainWindow) {
        mainWindow.show();
        mainWindow.focus();
    }
    return mainWindow;
};
const registerGitHandlers = () => {
    // Git IPC handlers
    electron_1.ipcMain.handle('git-status', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            // No folder is open, return null to show welcome screen
            SafeLogger_1.logger.info('[Main] git-status: No folder open, returning null');
            return null;
        }
        return yield gitManager.getStatus();
    }));
    electron_1.ipcMain.handle('git-branches', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            SafeLogger_1.logger.info('[Main] git-branches: No folder open, returning empty');
            return { all: [], branches: {}, current: null, detached: false };
        }
        return yield gitManager.getBranches();
    }));
    electron_1.ipcMain.handle('git-log', (_, options) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            SafeLogger_1.logger.info('[Main] git-log: No folder open, returning empty');
            return '';
        }
        SafeLogger_1.logger.info('[Main] git-log called with options:', options);
        const result = yield gitManager.getLog(options || {});
        SafeLogger_1.logger.info('[Main] git-log returning:', result ? result.substring(0, 100) + '...' : 'empty');
        return result;
    }));
    electron_1.ipcMain.handle('git-diff', (_, file) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager)
            return '';
        return yield gitManager.getDiff(file);
    }));
    electron_1.ipcMain.handle('git-staged-diff', (_, file) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager)
            return '';
        return yield gitManager.getStagedDiff(file);
    }));
    electron_1.ipcMain.handle('git-stage', (_, files) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        return yield gitManager.stage(files);
    }));
    electron_1.ipcMain.handle('git-unstage', (_, files) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        return yield gitManager.unstage(files);
    }));
    electron_1.ipcMain.handle('git-commit', (_, message) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        return yield gitManager.commit(message);
    }));
    electron_1.ipcMain.handle('git-discard', (_, files) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        return yield gitManager.discard(files);
    }));
    electron_1.ipcMain.handle('git-clean', (_, files) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        return yield gitManager.clean(files);
    }));
    electron_1.ipcMain.handle('git-push', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-push IPC called');
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            if (gitManager instanceof EnhancedGitManager_1.EnhancedGitManager) {
                const result = yield gitManager.push();
                SafeLogger_1.logger.info('[Main] git-push result:', result);
                if (!result.success) {
                    throw new Error(result.error || 'Push failed');
                }
                return result.output;
            }
            else {
                const result = yield gitManager.push();
                SafeLogger_1.logger.info('[Main] git-push completed successfully');
                return result;
            }
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] git-push failed:', error);
            throw error;
        }
    }));
    // Chunked push for large repositories
    electron_1.ipcMain.handle('git-push-chunked', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-push-chunked IPC called');
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            // Get the repository path from the git manager
            let repoPath;
            if ('getRepoPath' in gitManager && typeof gitManager.getRepoPath === 'function') {
                repoPath = gitManager.getRepoPath();
            }
            else {
                // Fallback to accessing the property directly
                repoPath = gitManager.repoPath;
            }
            SafeLogger_1.logger.info('[Main] Using repository path for chunked push:', repoPath);
            const result = yield git_chunked_push_main_1.GitChunkedPushMain.pushInBatches(repoPath);
            SafeLogger_1.logger.info('[Main] git-push-chunked result:', result);
            if (!result.success) {
                throw new Error(result.message);
            }
            return result.message;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] git-push-chunked failed:', error);
            throw error;
        }
    }));
    // Get repository statistics
    electron_1.ipcMain.handle('git-repo-stats', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-repo-stats IPC called');
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            // Get the repository path from the git manager
            let repoPath;
            if ('getRepoPath' in gitManager && typeof gitManager.getRepoPath === 'function') {
                repoPath = gitManager.getRepoPath();
            }
            else {
                // Fallback to accessing the property directly
                repoPath = gitManager.repoPath;
            }
            // Get current git status to know how many commits to push
            const gitStatus = yield gitManager.getStatus();
            SafeLogger_1.logger.info('[Main] Using repository path for stats:', repoPath);
            const stats = yield git_chunked_push_main_1.GitChunkedPushMain.getRepoStats(repoPath, gitStatus);
            SafeLogger_1.logger.info('[Main] Repository stats:', stats);
            return stats;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] git-repo-stats failed:', error);
            throw error;
        }
    }));
    // Push with options support
    electron_1.ipcMain.handle('git-push-with-options', (_event, options) => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-push-with-options IPC called with:', options);
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            const { exec } = require('child_process');
            const { promisify } = require('util');
            const execAsync = promisify(exec);
            // Get the repository path
            let repoPath;
            if ('getRepoPath' in gitManager && typeof gitManager.getRepoPath === 'function') {
                repoPath = gitManager.getRepoPath();
            }
            else {
                repoPath = gitManager.repoPath;
            }
            // Build git push command with options
            let command;
            // Check if we have a custom command FIRST
            if (options.customCommand) {
                command = options.customCommand;
                // Custom command should already have all necessary options
                SafeLogger_1.logger.info('[Main] Using custom command:', command);
            }
            else {
                // Build standard git push command
                command = 'git push';
                if (options.forceWithLease) {
                    command += ' --force-with-lease';
                }
                if (options.includeTags) {
                    command += ' --tags';
                }
                if (options.setUpstream) {
                    // Get current branch name
                    const status = yield gitManager.getStatus();
                    const branch = status.current || 'main';
                    command += ` -u origin ${branch}`;
                }
                if (options.atomic) {
                    command += ' --atomic';
                }
                if (options.signPush) {
                    command += ' --signed';
                }
                if (options.thinPack) {
                    command += ' --thin';
                }
                if (options.commitLimit) {
                    // Push only last N commits
                    const status = yield gitManager.getStatus();
                    const branch = status.current || 'main';
                    command = `git push origin HEAD~${options.commitLimit}:${branch}`;
                    // Add other options if commit limit is set
                    if (options.forceWithLease)
                        command += ' --force-with-lease';
                    if (options.atomic)
                        command += ' --atomic';
                }
            }
            SafeLogger_1.logger.info('[Main] Executing push command:', command);
            const result = yield execAsync(command, { cwd: repoPath, maxBuffer: 10 * 1024 * 1024 });
            SafeLogger_1.logger.info('[Main] Push with options completed successfully');
            return result.stdout || 'Push completed successfully';
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] git-push-with-options failed:', error);
            throw error;
        }
    }));
    // Push with --force-with-lease
    electron_1.ipcMain.handle('git-push-force-lease', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-push-force-lease IPC called');
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            const { exec } = require('child_process');
            const { promisify } = require('util');
            const execAsync = promisify(exec);
            let repoPath;
            if ('getRepoPath' in gitManager && typeof gitManager.getRepoPath === 'function') {
                repoPath = gitManager.getRepoPath();
            }
            else {
                repoPath = gitManager.repoPath;
            }
            const result = yield execAsync('git push --force-with-lease', {
                cwd: repoPath,
                maxBuffer: 10 * 1024 * 1024
            });
            SafeLogger_1.logger.info('[Main] Force with lease push completed successfully');
            return result.stdout || 'Force push with lease completed successfully';
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] git-push-force-lease failed:', error);
            throw error;
        }
    }));
    // Push custom command
    electron_1.ipcMain.handle('git-push-custom', (_event, command) => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-push-custom IPC called with:', command);
        if (!gitManager) {
            throw new Error('No folder open');
        }
        // Security check - ensure command starts with "git push"
        if (!command.trim().startsWith('git push')) {
            throw new Error('Custom command must start with "git push" for security');
        }
        try {
            const { exec } = require('child_process');
            const { promisify } = require('util');
            const execAsync = promisify(exec);
            let repoPath;
            if ('getRepoPath' in gitManager && typeof gitManager.getRepoPath === 'function') {
                repoPath = gitManager.getRepoPath();
            }
            else {
                repoPath = gitManager.repoPath;
            }
            SafeLogger_1.logger.info('[Main] Executing custom push command:', command);
            const result = yield execAsync(command, {
                cwd: repoPath,
                maxBuffer: 10 * 1024 * 1024,
                timeout: 600000 // 10 minute timeout for large pushes
            });
            SafeLogger_1.logger.info('[Main] Custom push completed successfully');
            return result.stdout || 'Custom push completed successfully';
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] git-push-custom failed:', error);
            throw error;
        }
    }));
    // Push dry run
    electron_1.ipcMain.handle('git-push-dry-run', (_event, options) => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-push-dry-run IPC called with:', options);
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            const { exec } = require('child_process');
            const { promisify } = require('util');
            const execAsync = promisify(exec);
            let repoPath;
            if ('getRepoPath' in gitManager && typeof gitManager.getRepoPath === 'function') {
                repoPath = gitManager.getRepoPath();
            }
            else {
                repoPath = gitManager.repoPath;
            }
            let command;
            // Check if we have a custom command
            if (options === null || options === void 0 ? void 0 : options.customCommand) {
                command = options.customCommand;
                // Add --dry-run if not already present
                if (!command.includes('--dry-run')) {
                    command += ' --dry-run';
                }
                // Add --porcelain for consistent output
                if (!command.includes('--porcelain')) {
                    command += ' --porcelain';
                }
            }
            else {
                // Build command with --dry-run
                command = 'git push --dry-run --porcelain';
                if (options === null || options === void 0 ? void 0 : options.forceWithLease) {
                    command += ' --force-with-lease';
                }
                if (options === null || options === void 0 ? void 0 : options.includeTags) {
                    command += ' --tags';
                }
                if (options === null || options === void 0 ? void 0 : options.setUpstream) {
                    const status = yield gitManager.getStatus();
                    const branch = status.current || 'main';
                    command += ` -u origin ${branch}`;
                }
            }
            SafeLogger_1.logger.info('[Main] Executing dry run:', command);
            const result = yield execAsync(command, { cwd: repoPath });
            SafeLogger_1.logger.info('[Main] Dry run completed');
            return result.stdout || 'Dry run completed - no changes made';
        }
        catch (error) {
            // Dry run often returns non-zero exit code, but that's OK
            if (error.stdout) {
                return error.stdout;
            }
            SafeLogger_1.logger.error('[Main] git-push-dry-run failed:', error);
            throw error;
        }
    }));
    electron_1.ipcMain.handle('git-pull', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        if (gitManager instanceof EnhancedGitManager_1.EnhancedGitManager) {
            const result = yield gitManager.pull();
            if (!result.success) {
                throw new Error(result.error || 'Pull failed');
            }
            return result.output;
        }
        else {
            return yield gitManager.pull();
        }
    }));
    electron_1.ipcMain.handle('git-sync', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] git-sync IPC called');
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            if (gitManager instanceof EnhancedGitManager_1.EnhancedGitManager) {
                const result = yield gitManager.sync();
                SafeLogger_1.logger.info('[Main] git-sync result:', result);
                if (!result.success) {
                    throw new Error(result.error || 'Sync failed');
                }
                return result.output;
            }
            else if (gitManager instanceof git_manager_v2_1.GitManagerV2) {
                const result = yield gitManager.sync();
                SafeLogger_1.logger.info('[Main] git-sync completed successfully');
                return result;
            }
            else {
                // Fallback for old GitManager - do pull then push
                yield gitManager.pull();
                yield gitManager.push();
                SafeLogger_1.logger.info('[Main] git-sync (pull+push) completed successfully');
                return;
            }
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] git-sync failed:', error);
            throw error;
        }
    }));
    electron_1.ipcMain.handle('git-fetch', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        if (gitManager instanceof EnhancedGitManager_1.EnhancedGitManager) {
            const result = yield gitManager.fetch();
            if (!result.success) {
                throw new Error(result.error || 'Fetch failed');
            }
            return result.output;
        }
        else {
            return yield gitManager.fetch();
        }
    }));
    electron_1.ipcMain.handle('git-switch-branch', (_, branchName) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        return yield gitManager.switchBranch(branchName);
    }));
    electron_1.ipcMain.handle('git-create-branch', (_, branchName) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            throw new Error('No folder open');
        }
        return yield gitManager.createBranch(branchName);
    }));
    electron_1.ipcMain.handle('git-file-status', (_, filePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager)
            return null;
        const status = yield gitManager.getStatus();
        const filesArray = Array.isArray(status.files) ? status.files : [];
        const file = filesArray.find((f) => f.path === filePath);
        if (file) {
            if (file.index !== ' ' && file.index !== '?')
                return 'staged';
            if (file.working === 'M')
                return 'modified';
            if (file.working === 'D')
                return 'deleted';
            if (file.working === '?')
                return 'untracked';
            if (file.working === 'A')
                return 'added';
        }
        return null;
    }));
    // Initialize Git repository
    electron_1.ipcMain.handle('git-init', (_, repoPath) => __awaiter(void 0, void 0, void 0, function* () {
        const git = new git_manager_1.GitManager(repoPath);
        yield git.initRepo();
        return { success: true };
    }));
    // Get files changed in a commit
    electron_1.ipcMain.handle('git-commit-files', (_, hash) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager)
            return [];
        return yield gitManager.getCommitFiles(hash);
    }));
    // Get diff for a specific file in a commit
    electron_1.ipcMain.handle('git-file-diff', (_, commitHash, filePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager)
            return '';
        return yield gitManager.getFileDiff(commitHash, filePath);
    }));
    // Update Git manager when folder changes
    electron_1.ipcMain.handle('git-set-folder', (_, folderPath) => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Git] Setting folder to:', folderPath || '(none)');
        // If empty string or null, clear the git manager
        if (!folderPath) {
            gitManager = null;
            SafeLogger_1.logger.info('[Git] Cleared Git manager - no folder open');
            return { success: true };
        }
        // Initialize with the new folder
        yield initGitManager(folderPath);
        return { success: true };
    }));
    // Get submodule status
    electron_1.ipcMain.handle('git-submodule-status', (_, submodulePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager)
            return '';
        try {
            // Create a new git instance for the submodule
            const simpleGit = (yield Promise.resolve().then(() => __importStar(require('simple-git')))).default;
            const submoduleGit = simpleGit(submodulePath);
            const result = yield submoduleGit.status();
            let statusText = '';
            // Format the status nicely
            if (result.ahead || result.behind) {
                statusText += `Your branch is ${result.ahead ? `ahead by ${result.ahead}` : ''} ${result.behind ? `behind by ${result.behind}` : ''}\n`;
            }
            // List modified files
            if (result.files && result.files.length > 0) {
                result.files.forEach((file) => {
                    let status = '';
                    // Check both index and working_dir for changes
                    if (file.index === 'M' || file.working_dir === 'M')
                        status = 'modified:';
                    else if (file.index === 'A')
                        status = 'new file:';
                    else if (file.index === 'D' || file.working_dir === 'D')
                        status = 'deleted:';
                    else if (file.working_dir === '?' || file.index === '?')
                        status = 'untracked:';
                    if (status) {
                        statusText += `${status}   ${file.path}\n`;
                    }
                });
            }
            return statusText || 'Working directory clean';
        }
        catch (error) {
            SafeLogger_1.logger.error('[Git] Failed to get submodule status:', error);
            return `Error: ${error}`;
        }
    }));
    // Get submodule diff
    electron_1.ipcMain.handle('git-submodule-diff', (_, submodulePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager)
            return '';
        try {
            // Create a new git instance for the submodule
            const simpleGit = (yield Promise.resolve().then(() => __importStar(require('simple-git')))).default;
            const submoduleGit = simpleGit(submodulePath);
            const diff = yield submoduleGit.diff();
            return diff;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Git] Failed to get submodule diff:', error);
            return '';
        }
    }));
};
const registerDialogHandlers = () => {
    // Dialog IPC handlers
    electron_1.ipcMain.handle('show-open-dialog', (_, options) => __awaiter(void 0, void 0, void 0, function* () {
        const result = yield electron_1.dialog.showOpenDialog(mainWindow, options);
        return result;
    }));
    electron_1.ipcMain.handle('show-save-dialog', (_, options) => __awaiter(void 0, void 0, void 0, function* () {
        const result = yield electron_1.dialog.showSaveDialog(mainWindow, options);
        return result;
    }));
    electron_1.ipcMain.handle('show-message-box', (_, options) => __awaiter(void 0, void 0, void 0, function* () {
        const result = yield electron_1.dialog.showMessageBox(mainWindow, options);
        return result;
    }));
    electron_1.ipcMain.handle('open-external', (_, url) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            yield electron_1.shell.openExternal(url);
            return true;
        }
        catch (error) {
            console.error('Failed to open external URL:', error);
            return false;
        }
    }));
    electron_1.ipcMain.handle('show-input-dialog', (_, title, defaultValue) => __awaiter(void 0, void 0, void 0, function* () {
        // For now, use a simple prompt-like dialog
        // In a real app, you'd create a custom dialog
        const result = yield electron_1.dialog.showMessageBox(mainWindow, {
            type: 'question',
            buttons: ['OK', 'Cancel'],
            defaultId: 0,
            title: title,
            message: title,
            detail: defaultValue || ''
        });
        if (result.response === 0) {
            // In a real implementation, you'd get the actual input value
            // For now, return a placeholder
            return 'https://github.com/user/repo.git';
        }
        return null;
    }));
    electron_1.ipcMain.handle('set-title', (_, title) => {
        if (mainWindow) {
            mainWindow.setTitle(title);
        }
    });
};
const registerFileSystemHandlers = () => {
    // File System IPC handlers
    electron_1.ipcMain.handle('fs-get-tree', (_, rootPath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        // Only return files if a root path is explicitly provided
        if (!rootPath) {
            SafeLogger_1.logger.info('[Main] fs-get-tree called without root path, returning empty');
            return [];
        }
        SafeLogger_1.logger.info('[Main] fs-get-tree called with root:', rootPath);
        const result = yield fileSystemManager.getFileTree(rootPath);
        SafeLogger_1.logger.info(`[Main] fs-get-tree returning ${(result === null || result === void 0 ? void 0 : result.length) || 0} items`);
        return result;
    }));
    electron_1.ipcMain.handle('fs-get-directory', (_, dirPath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        SafeLogger_1.logger.info('[Main] fs-get-directory called for:', dirPath);
        const result = yield fileSystemManager.getDirectoryContents(dirPath);
        SafeLogger_1.logger.info(`[Main] fs-get-directory returning ${(result === null || result === void 0 ? void 0 : result.length) || 0} items for ${dirPath}`);
        return result;
    }));
    electron_1.ipcMain.handle('fs-read-file', (_, filePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        return yield fileSystemManager.readFile(filePath);
    }));
    electron_1.ipcMain.handle('fs-write-file', (_, filePath, content) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        return yield fileSystemManager.writeFileContent(filePath, content);
    }));
    electron_1.ipcMain.handle('fs-watch-file', (_, filePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        fileSystemManager.watchFile(filePath, () => {
            // Send file change event to renderer
            if (mainWindow) {
                mainWindow.webContents.send('file-changed', filePath);
            }
        });
        return true; // Must return something when using ipcMain.handle
    }));
    electron_1.ipcMain.handle('fs-unwatch-file', (_, filePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        fileSystemManager.unwatchFile(filePath);
        return true; // Must return something when using ipcMain.handle
    }));
    electron_1.ipcMain.handle('fs-search', (_, rootPath, pattern) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        return yield fileSystemManager.searchFiles(rootPath, pattern);
    }));
    electron_1.ipcMain.handle('fs-stats', (_, filePath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        return yield fileSystemManager.getFileStats(filePath);
    }));
    electron_1.ipcMain.handle('fs-create-file', (_, dirPath, fileName) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const fs = require('fs').promises;
            const path = require('path');
            const filePath = path.join(dirPath, fileName);
            SafeLogger_1.logger.info('[Main] Creating file:', filePath);
            yield fs.writeFile(filePath, '', 'utf8');
            SafeLogger_1.logger.info('[Main] File created successfully:', filePath);
            return true;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Failed to create file:', error);
            throw error;
        }
    }));
    electron_1.ipcMain.handle('fs-create-folder', (_, dirPath, folderName) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const fs = require('fs').promises;
            const path = require('path');
            const folderPath = path.join(dirPath, folderName);
            SafeLogger_1.logger.info('[Main] Creating folder:', folderPath);
            yield fs.mkdir(folderPath, { recursive: true });
            SafeLogger_1.logger.info('[Main] Folder created successfully:', folderPath);
            return true;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Failed to create folder:', error);
            throw error;
        }
    }));
    electron_1.ipcMain.handle('fs-move-file', (_, sourcePath, targetPath) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const fs = require('fs').promises;
            SafeLogger_1.logger.info(`[Main] Moving: ${sourcePath} to ${targetPath}`);
            yield fs.rename(sourcePath, targetPath);
            SafeLogger_1.logger.info('[Main] Move successful');
            return true;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Failed to move file:', error);
            throw error;
        }
    }));
    electron_1.ipcMain.handle('fs-file-exists', (_, filePath) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const fs = require('fs').promises;
            yield fs.access(filePath);
            return true;
        }
        catch (_a) {
            return false;
        }
    }));
};
const createApplicationMenu = () => {
    const template = [
        {
            label: 'File',
            submenu: [
                {
                    label: 'New File',
                    accelerator: 'CmdOrCtrl+N',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-new-file');
                        }
                    }
                },
                {
                    label: 'Open File',
                    accelerator: 'CmdOrCtrl+O',
                    click: () => __awaiter(void 0, void 0, void 0, function* () {
                        const result = yield electron_1.dialog.showOpenDialog(mainWindow, {
                            properties: ['openFile'],
                            filters: [
                                { name: 'All Files', extensions: ['*'] },
                                { name: 'JavaScript', extensions: ['js', 'jsx'] },
                                { name: 'TypeScript', extensions: ['ts', 'tsx'] },
                                { name: 'HTML', extensions: ['html', 'htm'] },
                                { name: 'CSS', extensions: ['css', 'scss', 'less'] },
                                { name: 'JSON', extensions: ['json'] },
                                { name: 'Markdown', extensions: ['md'] }
                            ]
                        });
                        if (!result.canceled && result.filePaths.length > 0) {
                            if (mainWindow) {
                                mainWindow.webContents.send('menu-open-file', result.filePaths[0]);
                            }
                        }
                    })
                },
                {
                    label: 'Open Folder',
                    accelerator: 'CmdOrCtrl+K CmdOrCtrl+O',
                    click: () => __awaiter(void 0, void 0, void 0, function* () {
                        const result = yield electron_1.dialog.showOpenDialog(mainWindow, {
                            properties: ['openDirectory']
                        });
                        if (!result.canceled && result.filePaths.length > 0) {
                            if (mainWindow) {
                                mainWindow.webContents.send('menu-open-folder', result.filePaths[0]);
                            }
                        }
                    })
                },
                {
                    label: 'Close Folder',
                    accelerator: 'CmdOrCtrl+K F',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-close-folder');
                        }
                    }
                },
                { type: 'separator' },
                {
                    label: 'Save',
                    accelerator: 'CmdOrCtrl+S',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-save');
                        }
                    }
                },
                {
                    label: 'Save As...',
                    accelerator: 'CmdOrCtrl+Shift+S',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-save-as');
                        }
                    }
                },
                { type: 'separator' },
                {
                    label: 'Auto Save',
                    type: 'checkbox',
                    checked: false,
                    click: (menuItem) => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-toggle-auto-save', menuItem.checked);
                        }
                    }
                },
                { type: 'separator' },
                {
                    label: 'Close Tab',
                    accelerator: 'CmdOrCtrl+W',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-close-tab');
                        }
                    }
                },
                {
                    label: 'Close All Tabs',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-close-all-tabs');
                        }
                    }
                },
                { type: 'separator' },
                {
                    label: 'Exit',
                    accelerator: process.platform === 'darwin' ? 'Cmd+Q' : 'Ctrl+Q',
                    click: () => {
                        electron_1.app.quit();
                    }
                }
            ]
        },
        {
            label: 'Edit',
            submenu: [
                { label: 'Undo', accelerator: 'CmdOrCtrl+Z', role: 'undo' },
                { label: 'Redo', accelerator: 'CmdOrCtrl+Y', role: 'redo' },
                { type: 'separator' },
                { label: 'Cut', accelerator: 'CmdOrCtrl+X', role: 'cut' },
                { label: 'Copy', accelerator: 'CmdOrCtrl+C', role: 'copy' },
                { label: 'Paste', accelerator: 'CmdOrCtrl+V', role: 'paste' },
                { type: 'separator' },
                { label: 'Select All', accelerator: 'CmdOrCtrl+A', role: 'selectAll' },
                { type: 'separator' },
                {
                    label: 'Find',
                    accelerator: 'CmdOrCtrl+F',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-find');
                        }
                    }
                },
                {
                    label: 'Replace',
                    accelerator: 'CmdOrCtrl+H',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-replace');
                        }
                    }
                }
            ]
        },
        {
            label: 'View',
            submenu: [
                {
                    label: 'Toggle File Explorer',
                    accelerator: 'CmdOrCtrl+B',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-toggle-explorer');
                        }
                    }
                },
                {
                    label: 'Toggle Source Control',
                    accelerator: 'CmdOrCtrl+Shift+G',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-toggle-git');
                        }
                    }
                },
                {
                    label: 'Toggle Terminal',
                    accelerator: 'CmdOrCtrl+`',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-toggle-terminal');
                        }
                    }
                },
                { type: 'separator' },
                {
                    label: 'Reload',
                    accelerator: 'CmdOrCtrl+R',
                    click: () => {
                        if (mainWindow) {
                            SafeLogger_1.logger.info('[Menu] Reloading with state reset...');
                            // Clear Git folder state before reload
                            mainWindow.webContents.send('menu-reset-state');
                            // Give it a moment to clear state, then reload
                            setTimeout(() => {
                                mainWindow.webContents.reload();
                            }, 100);
                        }
                    }
                },
                { label: 'Force Reload', accelerator: 'CmdOrCtrl+Shift+R', role: 'forceReload' },
                { label: 'Toggle Developer Tools', accelerator: 'F12', role: 'toggleDevTools' },
                { type: 'separator' },
                { label: 'Actual Size', accelerator: 'CmdOrCtrl+0', role: 'resetZoom' },
                { label: 'Zoom In', accelerator: 'CmdOrCtrl+Plus', role: 'zoomIn' },
                { label: 'Zoom Out', accelerator: 'CmdOrCtrl+-', role: 'zoomOut' },
                { type: 'separator' },
                { label: 'Toggle Fullscreen', accelerator: 'F11', role: 'togglefullscreen' }
            ]
        },
        {
            label: 'Go',
            submenu: [
                {
                    label: 'Go to File...',
                    accelerator: 'CmdOrCtrl+P',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-go-to-file');
                        }
                    }
                },
                {
                    label: 'Go to Line...',
                    accelerator: 'CmdOrCtrl+G',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-go-to-line');
                        }
                    }
                }
            ]
        },
        {
            label: 'Window',
            submenu: [
                { label: 'Minimize', accelerator: 'CmdOrCtrl+M', role: 'minimize' },
                { label: 'Close', accelerator: 'CmdOrCtrl+W', role: 'close' }
            ]
        },
        {
            label: 'Help',
            submenu: [
                {
                    label: 'About',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-about');
                        }
                    }
                }
            ]
        }
    ];
    // macOS specific menu adjustments
    if (process.platform === 'darwin') {
        template.unshift({
            label: electron_1.app.getName(),
            submenu: [
                { label: 'About ' + electron_1.app.getName(), role: 'about' },
                { type: 'separator' },
                { label: 'Services', role: 'services', submenu: [] },
                { type: 'separator' },
                { label: 'Hide ' + electron_1.app.getName(), accelerator: 'Command+H', role: 'hide' },
                { label: 'Hide Others', accelerator: 'Command+Shift+H', role: 'hideothers' },
                { label: 'Show All', role: 'unhide' },
                { type: 'separator' },
                { label: 'Quit', accelerator: 'Command+Q', click: () => electron_1.app.quit() }
            ]
        });
    }
    const menu = electron_1.Menu.buildFromTemplate(template);
    electron_1.Menu.setApplicationMenu(menu);
};
// ========== MEMORY SERVICE INTEGRATION ==========
// Memory Service management - uses the shared ProcessManager instance
let memoryServicePort = 3457;
let websocketBackendPort = 8765; // Dynamic port for WebSocket backend
// CLI Tools Manager for AI CLI integration
// Note: The manager is now initialized as a singleton in registerCliToolHandlers()
// let cliToolsManager: CliToolsManager | null = null;  // DEPRECATED - using singleton pattern now
// Function to update MCP configurations for all tools with the actual Memory Service port
// This must be defined before initializeProcessManager where it's used
function updateAllMCPConfigurations(actualPort) {
    var _a, _b, _c, _d, _e;
    const memoryServiceEndpoint = `http://localhost:${actualPort}`;
    SafeLogger_1.logger.info(`[Main] Updating MCP configurations with Memory Service endpoint: ${memoryServiceEndpoint}`);
    try {
        // Read the CLI tools config to get tokens for each tool
        const configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');
        if (!fs.existsSync(configPath)) {
            SafeLogger_1.logger.info('[Main] No CLI tools config found, skipping MCP updates');
            return;
        }
        const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
        // Update MCP wrapper with the correct endpoint
        const wrapperPath = path.join(os.homedir(), '.hive', 'memory-service-mcp-wrapper.js');
        if (fs.existsSync(wrapperPath)) {
            // Read the wrapper and update the ENDPOINT fallback
            let wrapperContent = fs.readFileSync(wrapperPath, 'utf-8');
            // Replace the hardcoded endpoint in the fallback
            wrapperContent = wrapperContent.replace(/const ENDPOINT = process\.env\.MEMORY_SERVICE_ENDPOINT \|\| '[^']+'/, `const ENDPOINT = process.env.MEMORY_SERVICE_ENDPOINT || '${memoryServiceEndpoint}'`);
            fs.writeFileSync(wrapperPath, wrapperContent);
            SafeLogger_1.logger.info('[Main] Updated MCP wrapper with dynamic endpoint');
        }
        // Update Claude Code MCP configuration
        const claudeMcpPath = path.join(os.homedir(), '.claude', '.mcp.json');
        if (fs.existsSync(claudeMcpPath)) {
            try {
                const claudeMcp = JSON.parse(fs.readFileSync(claudeMcpPath, 'utf-8'));
                if ((_a = claudeMcp.servers) === null || _a === void 0 ? void 0 : _a['hive-memory-service']) {
                    const claudeToken = (_c = (_b = config['claude-code']) === null || _b === void 0 ? void 0 : _b.memoryService) === null || _c === void 0 ? void 0 : _c.token;
                    if (claudeToken) {
                        claudeMcp.servers['hive-memory-service'].env = {
                            MEMORY_SERVICE_ENDPOINT: memoryServiceEndpoint,
                            MEMORY_SERVICE_TOKEN: claudeToken
                        };
                        fs.writeFileSync(claudeMcpPath, JSON.stringify(claudeMcp, null, 2));
                        SafeLogger_1.logger.info('[Main] Updated Claude Code MCP configuration');
                    }
                }
            }
            catch (err) {
                SafeLogger_1.logger.error('[Main] Failed to update Claude MCP config:', err);
            }
        }
        // Update or create Grok MCP configuration
        const grokMcpPath = path.join(os.homedir(), '.grok', 'mcp-config.json');
        const grokToken = (_e = (_d = config['grok']) === null || _d === void 0 ? void 0 : _d.memoryService) === null || _e === void 0 ? void 0 : _e.token;
        if (grokToken) {
            try {
                let grokMcp = { servers: {} };
                // Read existing config if it exists
                if (fs.existsSync(grokMcpPath)) {
                    try {
                        grokMcp = JSON.parse(fs.readFileSync(grokMcpPath, 'utf-8'));
                    }
                    catch (_f) {
                        grokMcp = { servers: {} };
                    }
                }
                // Add or update the memory service server
                grokMcp.servers['hive-memory-service'] = {
                    transport: 'stdio',
                    command: 'node',
                    args: [wrapperPath],
                    env: {
                        MEMORY_SERVICE_ENDPOINT: memoryServiceEndpoint,
                        MEMORY_SERVICE_TOKEN: grokToken
                    }
                };
                // Ensure directory exists
                const grokDir = path.dirname(grokMcpPath);
                if (!fs.existsSync(grokDir)) {
                    fs.mkdirSync(grokDir, { recursive: true });
                }
                fs.writeFileSync(grokMcpPath, JSON.stringify(grokMcp, null, 2));
                SafeLogger_1.logger.info('[Main] Updated/Created Grok MCP configuration');
            }
            catch (err) {
                SafeLogger_1.logger.error('[Main] Failed to update Grok MCP config:', err);
            }
        }
        // We could add similar updates for other tools here if they support MCP
    }
    catch (error) {
        SafeLogger_1.logger.error('[Main] Failed to update MCP configurations:', error);
    }
}
// Initialize ProcessManager and register all managed processes
const initializeProcessManager = () => {
    // Register Memory Service configuration with ts-node
    processManager.registerProcess({
        name: 'memory-service',
        scriptPath: path.join(electron_1.app.getAppPath(), 'src', 'memory-service', 'index.ts'),
        args: [],
        env: {
            MEMORY_SERVICE_PORT: memoryServicePort.toString(),
            NODE_ENV: 'development',
            TS_NODE_TRANSPILE_ONLY: 'true' // Use env var instead of arg
        },
        port: memoryServicePort,
        alternativePorts: Array.from({ length: 50 }, (_, i) => memoryServicePort + i + 1),
        autoRestart: true,
        maxRestarts: 2,
        restartDelay: 500 // Reduced for faster recovery
        // Health checks disabled - causing startup issues
    });
    // Register WebSocket Consensus Backend with bundled Python support
    const consensusBackendPath = path.join('/Users/veronelazio/Developer/Private/hive', 'target', 'debug', 'hive-backend-server-enhanced');
    SafeLogger_1.logger.info('[ProcessManager] Registering WebSocket backend at:', consensusBackendPath);
    // For now, use the actual venv Python from the hive directory
    // TODO: In production, bundle a portable Python distribution
    const bundledPythonPath = '/Users/veronelazio/Developer/Private/hive/venv/bin/python3';
    const bundledModelScript = path.join(electron_1.app.getAppPath(), 'resources', 'python-runtime', 'models', 'model_service.py');
    SafeLogger_1.logger.info('[ProcessManager] Bundled Python path:', bundledPythonPath);
    SafeLogger_1.logger.info('[ProcessManager] Bundled model script:', bundledModelScript);
    processManager.registerProcess({
        name: 'websocket-backend',
        scriptPath: consensusBackendPath,
        args: [],
        env: {
            PORT: '8765',
            RUST_LOG: 'info',
            NODE_ENV: 'development',
            HIVE_BUNDLED_PYTHON: bundledPythonPath,
            HIVE_BUNDLED_MODEL_SCRIPT: bundledModelScript
        },
        port: 8765,
        alternativePorts: Array.from({ length: 100 }, (_, i) => 8765 + i + 1),
        autoRestart: true,
        maxRestarts: 2,
        restartDelay: 1000 // Reduced for faster recovery
        // Health checks disabled - causing startup issues
    });
    // Listen for process messages
    processManager.on('process:message', (name, msg) => {
        if (name === 'memory-service') {
            if (msg.type === 'ready') {
                SafeLogger_1.logger.info('[Main] Memory Service ready on port:', msg.port);
                memoryServicePort = msg.port || memoryServicePort;
                // Update MCP configurations for all tools with the actual dynamic port
                updateAllMCPConfigurations(memoryServicePort);
            }
            else if (msg.type === 'db-query') {
                handleMemoryServiceDbQuery(msg);
            }
        }
        else if (name === 'websocket-backend') {
            if (msg.type === 'ready') {
                SafeLogger_1.logger.info('[Main] WebSocket backend ready on port:', msg.port);
                // Store the actual port for WebSocket connections
                websocketBackendPort = msg.port;
            }
        }
    });
    // Listen for process status changes
    processManager.on('process:crashed', (name) => {
        SafeLogger_1.logger.error(`[Main] Process ${name} crashed`);
        mainWindow === null || mainWindow === void 0 ? void 0 : mainWindow.webContents.send('process-status', { name, status: 'crashed' });
    });
    processManager.on('process:started', (name) => {
        SafeLogger_1.logger.info(`[Main] Process ${name} started`);
        mainWindow === null || mainWindow === void 0 ? void 0 : mainWindow.webContents.send('process-status', { name, status: 'running' });
    });
    processManager.on('process:unhealthy', (name, error) => {
        SafeLogger_1.logger.error(`[Main] Process ${name} health check failed:`, error.message);
        mainWindow === null || mainWindow === void 0 ? void 0 : mainWindow.webContents.send('process-health', { name, healthy: false, error: error.message });
    });
};
// IPC handler to get all process statuses
electron_1.ipcMain.handle('process-manager-status', () => __awaiter(void 0, void 0, void 0, function* () {
    const processes = processManager.getAllProcesses();
    return processes.map(p => ({
        name: p.name,
        status: p.status,
        pid: p.pid,
        port: p.port,
        lastError: p.lastError,
        restartCount: p.restartCount
    }));
}));
// IPC handler to get specific service port
electron_1.ipcMain.handle('get-service-port', (_, serviceName) => __awaiter(void 0, void 0, void 0, function* () {
    const processInfo = processManager.getProcessStatus(serviceName);
    return (processInfo === null || processInfo === void 0 ? void 0 : processInfo.port) || null;
}));
// IPC handler to get WebSocket backend port specifically
electron_1.ipcMain.handle('get-websocket-port', () => __awaiter(void 0, void 0, void 0, function* () {
    const processInfo = processManager.getProcessStatus('websocket-backend');
    // Return the actual allocated port, or default if not running
    return (processInfo === null || processInfo === void 0 ? void 0 : processInfo.port) || 8765;
}));
// IPC handler to get full process manager status report
electron_1.ipcMain.handle('process-manager-full-status', () => __awaiter(void 0, void 0, void 0, function* () {
    return processManager.getFullStatus();
}));
// IPC handler to debug a specific process
electron_1.ipcMain.handle('process-manager-debug', (_, processName) => __awaiter(void 0, void 0, void 0, function* () {
    return yield processManager.debugProcess(processName);
}));
// IPC handler to log process manager status to console
electron_1.ipcMain.handle('process-manager-log-status', () => __awaiter(void 0, void 0, void 0, function* () {
    processManager.logStatus();
    return { logged: true };
}));
// Send message to Memory Service process
const sendToMemoryService = (message) => {
    const processInfo = processManager.getProcessStatus('memory-service');
    if (processInfo && processInfo.process && processInfo.process.send) {
        processInfo.process.send(message);
    }
    else {
        SafeLogger_1.logger.error('[Main] Memory Service process not available for IPC');
    }
};
// Handle database queries from Memory Service
const handleMemoryServiceDbQuery = (msg) => {
    SafeLogger_1.logger.info('[Main] Received db-query from Memory Service:', msg.sql);
    if (!db) {
        SafeLogger_1.logger.error('[Main] Database not initialized');
        sendToMemoryService({
            type: 'db-result',
            id: msg.id,
            error: 'Database not initialized',
            data: null
        });
        return;
    }
    // Execute query with callback-based sqlite3
    db.all(msg.sql, msg.params, (error, rows) => {
        SafeLogger_1.logger.info('[Main] Database query result:', error ? `Error: ${error.message}` : `${(rows === null || rows === void 0 ? void 0 : rows.length) || 0} rows`);
        sendToMemoryService({
            type: 'db-result',
            id: msg.id,
            error: error ? error.message : null,
            data: rows || []
        });
    });
};
// Register Memory Service IPC handlers
const registerMemoryServiceHandlers = () => {
    SafeLogger_1.logger.info('[Main] Registering Memory Service IPC handlers');
    electron_1.ipcMain.handle('memory-service-start', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] IPC: memory-service-start');
        const status = processManager.getProcessStatus('memory-service');
        if (status && status.status === 'running') {
            SafeLogger_1.logger.info('[Main] Memory Service already running');
            return true;
        }
        try {
            SafeLogger_1.logger.info('[Main] Starting Memory Service as child process...');
            // Use ts-node to run TypeScript directly from source directory
            const scriptPath = path.join(electron_1.app.getAppPath(), 'src', 'memory-service', 'index.ts');
            SafeLogger_1.logger.info('[Main] Memory Service script path:', scriptPath);
            // Start using ProcessManager
            const started = yield processManager.startProcess('memory-service');
            if (started) {
                // Update port if it changed
                const processInfo = processManager.getProcessStatus('memory-service');
                if (processInfo === null || processInfo === void 0 ? void 0 : processInfo.port) {
                    memoryServicePort = processInfo.port;
                }
            }
            return started;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Failed to start Memory Service:', error);
            return false;
        }
    }));
    electron_1.ipcMain.handle('memory-service-stop', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] IPC: memory-service-stop');
        return yield processManager.stopProcess('memory-service');
    }));
    electron_1.ipcMain.handle('memory-service-status', () => __awaiter(void 0, void 0, void 0, function* () {
        const status = processManager.getProcessStatus('memory-service');
        const isRunning = (status === null || status === void 0 ? void 0 : status.status) === 'running';
        SafeLogger_1.logger.info('[Main] IPC: memory-service-status, result:', isRunning);
        return isRunning;
    }));
    electron_1.ipcMain.handle('memory-service-stats', () => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const response = yield fetch(`http://localhost:${memoryServicePort}/api/v1/memory/stats`);
            if (response.ok) {
                return yield response.json();
            }
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Failed to get memory stats:', error);
        }
        // Return default stats if service is not available
        return {
            totalMemories: 0,
            queriesToday: 0,
            contributionsToday: 0,
            connectedTools: 0,
            hitRate: 0,
            avgResponseTime: 0
        };
    }));
    electron_1.ipcMain.handle('memory-service-tools', () => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const response = yield fetch(`http://localhost:${memoryServicePort}/api/v1/memory/tools`);
            if (response.ok) {
                const data = yield response.json();
                return data.tools || [];
            }
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Failed to get connected tools:', error);
        }
        return [];
    }));
    electron_1.ipcMain.handle('memory-service-activity', (_, limit = 50) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const response = yield fetch(`http://localhost:${memoryServicePort}/api/v1/memory/activity?limit=${limit}`);
            if (response.ok) {
                const data = yield response.json();
                return data.activity || [];
            }
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Failed to get activity stream:', error);
        }
        return [];
    }));
};
// ========== CLI TOOLS MANAGEMENT (DEPRECATED - NOW IN registerSimpleCliToolHandlers) ==========
// Initialize CLI Tools Manager (DEPRECATED - using singleton pattern now)
/* DEPRECATED - Now using singleton pattern in registerSimpleCliToolHandlers()
const initializeCliToolsManager = () => {
  if (!db) {
    logger.error('[Main] Database not initialized for CLI Tools Manager');
    return;
  }
  
  cliToolsManager = new CliToolsManager(db);
  
  // Set up event listeners
  cliToolsManager.on('install-progress', (progress) => {
    // Forward progress to renderer
    if (mainWindow) {
      mainWindow.webContents.send('cli-install-progress', progress);
    }
  });
  
  cliToolsManager.on('update-available', (info) => {
    // Notify renderer about available updates
    if (mainWindow) {
      mainWindow.webContents.send('cli-update-available', info);
    }
  });
  
  // Start automatic update checking
  cliToolsManager.startAutoUpdateCheck();
  
  // Register IPC handlers
  registerCliToolsHandlers();
  
  logger.info('[Main] CLI Tools Manager initialized');
};
*/
// Register CLI Tools IPC handlers (DEPRECATED - duplicate handlers now exist below)
/* DEPRECATED - These handlers conflict with the new singleton implementation below
const registerCliToolsHandlers = () => {
  // Get all tool statuses
  ipcMain.handle('cli-tools-get-all-status', async () => {
    if (!cliToolsManager) return {};
    const statuses = await cliToolsManager.getAllStatuses();
    return Object.fromEntries(statuses);
  });
  
  // Get specific tool status
  ipcMain.handle('cli-tools-check-installed', async (_, toolId: string) => {
    if (!cliToolsManager) return false;
    return await cliToolsManager.checkInstalled(toolId);
  });
  
  // Install a tool
  ipcMain.handle('cli-tools-install', async (_, toolId: string) => {
    if (!cliToolsManager) throw new Error('CLI Tools Manager not initialized');
    await cliToolsManager.install(toolId);
    return { success: true };
  });
  
  // Uninstall a tool
  ipcMain.handle('cli-tools-uninstall', async (_, toolId: string) => {
    if (!cliToolsManager) throw new Error('CLI Tools Manager not initialized');
    await cliToolsManager.uninstall(toolId);
    return { success: true };
  });
  
  // Update a tool
  ipcMain.handle('cli-tools-update', async (_, toolId: string) => {
    if (!cliToolsManager) throw new Error('CLI Tools Manager not initialized');
    await cliToolsManager.update(toolId);
    return { success: true };
  });
  
  // Check for updates for a single tool
  ipcMain.handle('cli-tools-check-update', async (_, toolId: string) => {
    if (!cliToolsManager) return false;
    return await cliToolsManager.checkForUpdates(toolId);
  });
  
  // Check for updates for all tools
  ipcMain.handle('cli-tools-check-all-updates', async () => {
    if (!cliToolsManager) return {};
    const updates = await cliToolsManager.checkAllUpdates();
    return Object.fromEntries(updates);
  });
  
  // Configure a tool (e.g., auth)
  ipcMain.handle('cli-tools-configure', async (_, toolId: string) => {
    if (!cliToolsManager) throw new Error('CLI Tools Manager not initialized');
    await cliToolsManager.configureTool(toolId);
    return { success: true };
  });
  
  // Cancel installation
  ipcMain.handle('cli-tools-cancel-install', async (_, toolId: string) => {
    if (!cliToolsManager) return;
    cliToolsManager.cancelInstallation(toolId);
    return { success: true };
  });
  
  // Get installation logs
  ipcMain.handle('cli-tools-get-logs', async (_, toolId: string) => {
    if (!cliToolsManager) return [];
    return cliToolsManager.getInstallationLogs(toolId);
  });
  
  // Update settings
  ipcMain.handle('cli-tools-update-settings', async (_, settings: any) => {
    if (!cliToolsManager) return;
    cliToolsManager.updateSettings(settings);
    return { success: true };
  });
  
  // Get tool configuration
  ipcMain.handle('cli-tools-get-config', async (_, toolId: string) => {
    if (!cliToolsManager) return null;
    return cliToolsManager.getTool(toolId);
  });
  
  // Get all tools
  ipcMain.handle('cli-tools-get-all', async () => {
    if (!cliToolsManager) return {};
    const tools = cliToolsManager.getAllTools();
    return Object.fromEntries(tools);
  });
  
  // Select directory (for custom install path)
  ipcMain.handle('select-directory', async () => {
    const result = await dialog.showOpenDialog({
      properties: ['openDirectory', 'createDirectory']
    });
    return result.canceled ? null : result.filePaths[0];
  });
  
  // Forward progress events to renderer
  if (cliToolsManager) {
    cliToolsManager.on('install-progress', (data: any) => {
      if (mainWindow) {
        mainWindow.webContents.send('cli-tool-progress', data);
      }
    });
    
    cliToolsManager.on('update-available', (data: any) => {
      if (mainWindow) {
        mainWindow.webContents.send('cli-tool-update-available', data);
      }
    });
  }
};
*/
// Simple CLI tool detection function - DEPRECATED, using cliToolsDetector instead
/* const detectCliToolSimple = async (toolId: string): Promise<any> => {
  const { exec } = require('child_process');
  const { promisify } = require('util');
  const execAsync = promisify(exec);
  
  // Map tool IDs to commands
  const toolCommands: Record<string, string> = {
    'claude-code': 'claude',
    'aider': 'aider',
    'cursor': 'cursor',
    'continue': 'continue',
    'codewhisperer': 'aws',
    'cody': 'cody',
    'qwen-code': 'qwen',
    'gemini-cli': 'gemini'
  };
  
  const command = toolCommands[toolId];
  if (!command) return null;
  
  try {
    // Add common paths to PATH for detection
    const pathAdditions = ['/opt/homebrew/bin', '/usr/local/bin', '/usr/bin', '/bin'];
    const enhancedPath = [...new Set([...pathAdditions, ...(process.env.PATH || '').split(':')])].join(':');
    
    // Try to detect the tool with enhanced PATH
    const { stdout } = await execAsync(`which ${command}`, { env: { ...process.env, PATH: enhancedPath } });
    const path = stdout.trim();
    
    // Try to get version
    let version = 'unknown';
    try {
      const { stdout: versionOut } = await execAsync(`${command} --version 2>&1`, { env: { ...process.env, PATH: enhancedPath } });
      // Extract version from output (different tools have different formats)
      const versionMatch = versionOut.match(/\d+\.\d+\.\d+/) || versionOut.match(/\d+\.\d+/);
      if (versionMatch) {
        version = versionMatch[0];
      } else if (versionOut.includes('(Claude Code)')) {
        // Special handling for Claude Code
        const match = versionOut.match(/^([\d.]+)/);
        if (match) version = match[1];
      }
      logger.info(`[CLI Detector] ${toolId} version output: ${versionOut.substring(0, 100)}`);
      logger.info(`[CLI Detector] Detected version: ${version}`);
    } catch (e) {
      // Version command failed, but tool exists
    }
    
    return {
      id: toolId,
      name: toolId.replace(/-/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
      installed: true,
      version,
      path,
      memoryServiceConnected: false
    };
  } catch (error) {
    // Tool not found
    return {
      id: toolId,
      name: toolId.replace(/-/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
      installed: false
    };
  }
}; */
// Register simple CLI tool detection handlers (without the complex CliToolsManager)
const registerSimpleCliToolHandlers = () => {
    SafeLogger_1.logger.info('[Main] Registering simple CLI tool detection handlers');
    // CLI Tool Detection Handlers
    electron_1.ipcMain.handle('cli-tool-detect', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info(`[Main] Detecting CLI tool: ${toolId}`);
        try {
            const status = yield detector_1.cliToolsDetector.detectTool(toolId);
            return status;
        }
        catch (error) {
            SafeLogger_1.logger.error(`[Main] Error detecting CLI tool ${toolId}:`, error);
            return null;
        }
    }));
    electron_1.ipcMain.handle('cli-tools-detect-all', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] Detecting all CLI tools...');
        try {
            const results = yield detector_1.cliToolsDetector.detectAllTools();
            return results;
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Error detecting CLI tools:', error);
            return [];
        }
    }));
    // Install CLI tool
    electron_1.ipcMain.handle('cli-tool-install', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        var _a, _b, _c, _d;
        SafeLogger_1.logger.info(`[Main] Installing CLI tool: ${toolId}`);
        try {
            const { exec } = require('child_process');
            const { promisify } = require('util');
            const execAsync = promisify(exec);
            // Get tool configuration from registry
            const toolConfig = cli_tools_1.CLI_TOOLS_REGISTRY[toolId];
            if (!toolConfig) {
                SafeLogger_1.logger.error(`[Main] Unknown tool ID for installation: ${toolId}`);
                return { success: false, error: `Unknown tool: ${toolId}` };
            }
            if (!toolConfig.installCommand) {
                SafeLogger_1.logger.error(`[Main] No installation command defined for: ${toolId}`);
                return { success: false, error: `Installation not available for ${toolConfig.name}` };
            }
            // Enhanced PATH for finding package managers
            const enhancedPath = `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}`;
            SafeLogger_1.logger.info(`[Main] Running installation command: ${toolConfig.installCommand}`);
            try {
                // Run the installation command
                const { stdout, stderr } = yield execAsync(toolConfig.installCommand, {
                    env: Object.assign(Object.assign({}, process.env), { PATH: enhancedPath }),
                    timeout: 120000 // 2 minutes timeout for installation
                });
                SafeLogger_1.logger.info(`[Main] Installation output: ${stdout}`);
                if (stderr && !stderr.includes('WARN') && !stderr.includes('warning')) {
                    SafeLogger_1.logger.warn(`[Main] Installation stderr: ${stderr}`);
                }
                // Verify installation by checking if command exists
                let version = 'Unknown';
                try {
                    if (toolConfig.versionCommand) {
                        const versionResult = yield execAsync(toolConfig.versionCommand, {
                            env: Object.assign(Object.assign({}, process.env), { PATH: enhancedPath }),
                            timeout: 5000
                        });
                        // Extract version based on tool
                        if (toolId === 'gemini-cli') {
                            const match = versionResult.stdout.match(/(?:gemini-cli\/|v?)(\d+\.\d+\.\d+)/);
                            version = match ? match[1] : 'Unknown';
                        }
                        else if (toolId === 'claude-code') {
                            const match = versionResult.stdout.match(/claude-code\/(\d+\.\d+\.\d+)/);
                            version = match ? match[1] : 'Unknown';
                        }
                        else if (toolId === 'qwen-code') {
                            const match = versionResult.stdout.match(/(?:qwen\/|v?)(\d+\.\d+\.\d+)/);
                            version = match ? match[1] : 'Unknown';
                        }
                        else if (toolId === 'openai-codex') {
                            const match = versionResult.stdout.match(/codex-cli (\d+\.\d+\.\d+)/);
                            version = match ? match[1] : 'Unknown';
                        }
                        else if (toolId === 'cline') {
                            // Cline outputs just version number like "0.0.1"
                            const match = versionResult.stdout.match(/(\d+\.\d+\.\d+)/);
                            version = match ? match[1] : 'Unknown';
                        }
                        else if (toolId === 'grok') {
                            // Grok CLI outputs version number like "0.0.23"
                            const match = versionResult.stdout.match(/(\d+\.\d+\.\d+)/);
                            version = match ? match[1] : 'Unknown';
                        }
                        else {
                            // Generic version extraction
                            const match = versionResult.stdout.match(/(\d+\.\d+\.\d+)/);
                            version = match ? match[1] : 'Unknown';
                        }
                    }
                }
                catch (versionError) {
                    SafeLogger_1.logger.warn(`[Main] Could not get version after installation:`, versionError);
                }
                // CRITICAL FIX: Clear the detector cache for this tool so UI refresh works
                SafeLogger_1.logger.info(`[Main] Clearing detector cache for ${toolId} after successful install`);
                detector_1.cliToolsDetector.clearCache(toolId);
                // SEAMLESS CONFIGURATION: Automatically configure the tool after installation
                SafeLogger_1.logger.info(`[Main] Automatically configuring ${toolId} after installation...`);
                // 1. Special configuration for Cline - set up OpenRouter API key
                if (toolId === 'cline') {
                    SafeLogger_1.logger.info('[Main] Configuring Cline with OpenRouter API key from Hive');
                    if (db) {
                        const apiKeyRow = yield new Promise((resolve) => {
                            db.get('SELECT value FROM configurations WHERE key = ?', ['openrouter_api_key'], (err, row) => {
                                resolve(row);
                            });
                        });
                        if (apiKeyRow && apiKeyRow.value) {
                            const openrouterKey = apiKeyRow.value;
                            const clineConfigDir = path.join(os.homedir(), '.cline_cli');
                            if (!fs.existsSync(clineConfigDir)) {
                                fs.mkdirSync(clineConfigDir, { recursive: true });
                            }
                            // Create Cline settings file
                            const clineSettingsPath = path.join(clineConfigDir, 'cline_cli_settings.json');
                            const clineSettings = {
                                globalState: {
                                    apiProvider: 'openrouter',
                                    openRouterApiKey: openrouterKey,
                                    apiModelId: '',
                                    autoApprovalSettings: {
                                        enabled: false,
                                        actions: {
                                            readFiles: false,
                                            editFiles: false,
                                            executeSafeCommands: false,
                                            useMcp: false
                                        },
                                        maxRequests: 20
                                    }
                                },
                                settings: {
                                    'cline.enableCheckpoints': false
                                }
                            };
                            fs.writeFileSync(clineSettingsPath, JSON.stringify(clineSettings, null, 2));
                            // Create keys file
                            const keysPath = path.join(clineConfigDir, 'keys.json');
                            fs.writeFileSync(keysPath, JSON.stringify({ openRouterApiKey: openrouterKey }, null, 2));
                            // Create storage directory
                            const storageDir = path.join(clineConfigDir, 'storage');
                            if (!fs.existsSync(storageDir)) {
                                fs.mkdirSync(storageDir, { recursive: true });
                            }
                            SafeLogger_1.logger.info('[Main] Cline configured with OpenRouter API key');
                        }
                    }
                }
                // 2. Register with Memory Service for all tools (except those that don't need it)
                const toolsWithoutMemoryService = ['cursor', 'continue', 'codewhisperer', 'cody'];
                if (!toolsWithoutMemoryService.includes(toolId)) {
                    SafeLogger_1.logger.info(`[Main] Registering ${toolId} with Memory Service`);
                    try {
                        // Get memory service configuration
                        const memoryServiceEndpoint = process.env.MEMORY_SERVICE_ENDPOINT || 'http://localhost:11437';
                        const token = crypto.randomBytes(32).toString('hex');
                        // Create MCP wrapper for the tool
                        const mcpWrapperDir = path.join(os.homedir(), '.hive', 'mcp-wrappers');
                        if (!fs.existsSync(mcpWrapperDir)) {
                            fs.mkdirSync(mcpWrapperDir, { recursive: true });
                        }
                        const wrapperPath = path.join(mcpWrapperDir, `${toolId}-memory-service.js`);
                        const wrapperContent = `#!/usr/bin/env node
// Auto-generated MCP wrapper for ${toolId} to connect with Hive Memory Service

const { McpServer } = require('@modelcontextprotocol/server');

const ENDPOINT = process.env.MEMORY_SERVICE_ENDPOINT || '${memoryServiceEndpoint}';
const TOKEN = process.env.MEMORY_SERVICE_TOKEN || '${token}';

class MemoryServiceMCP extends McpServer {
  constructor() {
    super({
      name: 'hive-memory-service',
      version: '1.0.0',
      description: 'Hive Consensus Memory Service - AI memory and learning system'
    });
  }

  async start() {
    await super.start();
    
    this.registerTool({
      name: 'query_memory',
      description: 'Query the AI memory system for relevant learnings',
      parameters: {
        query: { type: 'string', required: true },
        limit: { type: 'number', default: 5 }
      },
      handler: this.queryMemory.bind(this)
    });

    this.registerTool({
      name: 'contribute_learning',
      description: 'Contribute a new learning to the memory system',
      parameters: {
        type: { type: 'string', required: true },
        category: { type: 'string', required: true },
        content: { type: 'string', required: true },
        code: { type: 'string' }
      },
      handler: this.contributeLearning.bind(this)
    });
  }

  async queryMemory({ query, limit = 5 }) {
    const response = await fetch(\`\${ENDPOINT}/api/v1/memory/query\`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': \`Bearer \${TOKEN}\`,
        'X-Client-Name': '${toolId}-mcp'
      },
      body: JSON.stringify({
        client: '${toolId}',
        context: { file: process.cwd() },
        query,
        options: { limit }
      })
    });

    return await response.json();
  }

  async contributeLearning({ type, category, content, code }) {
    const response = await fetch(\`\${ENDPOINT}/api/v1/memory/contribute\`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': \`Bearer \${TOKEN}\`,
        'X-Client-Name': '${toolId}-mcp'
      },
      body: JSON.stringify({
        source: '${toolId}',
        learning: {
          type,
          category,
          content,
          code,
          context: { file: process.cwd(), success: true }
        }
      })
    });

    return await response.json();
  }
}

// Start the MCP server
const server = new MemoryServiceMCP();
server.start();
`;
                        fs.writeFileSync(wrapperPath, wrapperContent);
                        fs.chmodSync(wrapperPath, '755');
                        // Update MCP configuration for Claude Code
                        const mcpConfigPath = path.join(os.homedir(), '.claude', '.mcp.json');
                        let mcpConfig = { servers: {} };
                        if (fs.existsSync(mcpConfigPath)) {
                            try {
                                mcpConfig = JSON.parse(fs.readFileSync(mcpConfigPath, 'utf-8'));
                            }
                            catch (_e) {
                                mcpConfig = { servers: {} };
                            }
                        }
                        // Add or update the memory service server
                        mcpConfig.servers['hive-memory-service'] = {
                            command: 'node',
                            args: [wrapperPath],
                            env: {
                                MEMORY_SERVICE_ENDPOINT: memoryServiceEndpoint,
                                MEMORY_SERVICE_TOKEN: token
                            },
                            description: 'Hive Consensus Memory Service - AI memory and learning system'
                        };
                        // Ensure directory exists
                        const mcpDir = path.dirname(mcpConfigPath);
                        if (!fs.existsSync(mcpDir)) {
                            fs.mkdirSync(mcpDir, { recursive: true });
                        }
                        fs.writeFileSync(mcpConfigPath, JSON.stringify(mcpConfig, null, 2));
                        SafeLogger_1.logger.info(`[Main] Successfully registered ${toolId} with Memory Service`);
                    }
                    catch (configError) {
                        SafeLogger_1.logger.warn(`[Main] Could not configure Memory Service for ${toolId}:`, configError);
                        // Non-fatal error - tool is still installed
                    }
                }
                SafeLogger_1.logger.info(`[Main] ${toolConfig.name} installed and configured successfully, version: ${version}`);
                return { success: true, version, message: `Installed ${toolConfig.name} version ${version}` };
            }
            catch (error) {
                SafeLogger_1.logger.error(`[Main] Failed to install ${toolConfig.name}:`, error);
                // Check for specific error conditions
                if (((_a = error.message) === null || _a === void 0 ? void 0 : _a.includes('EACCES')) || ((_b = error.message) === null || _b === void 0 ? void 0 : _b.includes('permission'))) {
                    return {
                        success: false,
                        error: `Permission denied. Try running: sudo ${toolConfig.installCommand}`
                    };
                }
                if ((_c = error.message) === null || _c === void 0 ? void 0 : _c.includes('npm: command not found')) {
                    return {
                        success: false,
                        error: 'npm not found. Please install Node.js first.'
                    };
                }
                if ((_d = error.message) === null || _d === void 0 ? void 0 : _d.includes('pip: command not found')) {
                    return {
                        success: false,
                        error: 'pip not found. Please install Python first.'
                    };
                }
                return { success: false, error: error.message || 'Installation failed' };
            }
        }
        catch (error) {
            SafeLogger_1.logger.error(`[Main] Unexpected error installing ${toolId}:`, error);
            return { success: false, error: error.message || 'Unexpected error occurred' };
        }
    }));
    // Update CLI tool
    electron_1.ipcMain.handle('cli-tool-update', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        var _f, _g, _h, _j;
        SafeLogger_1.logger.info(`[Main] Updating CLI tool: ${toolId}`);
        try {
            const { exec } = require('child_process');
            const { promisify } = require('util');
            const execAsync = promisify(exec);
            // Map tool IDs to their NPM packages
            const npmPackages = {
                'claude-code': '@anthropic-ai/claude-code',
                'gemini-cli': '@google/gemini-cli',
                'qwen-code': '@qwen-code/qwen-code',
                'openai-codex': '@openai/codex',
                'cline': '@yaegaki/cline-cli',
                'grok': '@vibe-kit/grok-cli'
            };
            const packageName = npmPackages[toolId];
            if (!packageName) {
                SafeLogger_1.logger.error(`[Main] Unknown tool ID for update: ${toolId}`);
                return { success: false, error: `Unknown tool: ${toolId}` };
            }
            // For Python-based tools like aider, use pip
            if (toolId === 'aider') {
                SafeLogger_1.logger.info(`[Main] Updating ${toolId} via pip...`);
                const command = 'pip install --upgrade aider-chat';
                try {
                    const { stdout, stderr } = yield execAsync(command, {
                        env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                    });
                    SafeLogger_1.logger.info(`[Main] Pip update output: ${stdout}`);
                    if (stderr && !stderr.includes('WARNING')) {
                        SafeLogger_1.logger.warn(`[Main] Pip update stderr: ${stderr}`);
                    }
                    // Get updated version
                    const versionResult = yield execAsync('aider --version', {
                        env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                    });
                    const version = ((_f = versionResult.stdout.trim().match(/\d+\.\d+\.\d+/)) === null || _f === void 0 ? void 0 : _f[0]) || 'Unknown';
                    SafeLogger_1.logger.info(`[Main] ${toolId} updated successfully to version ${version}`);
                    return { success: true, version, message: `Updated to version ${version}` };
                }
                catch (error) {
                    SafeLogger_1.logger.error(`[Main] Failed to update ${toolId}:`, error);
                    return { success: false, error: error.message || 'Update failed' };
                }
            }
            // For NPM-based tools
            SafeLogger_1.logger.info(`[Main] Updating ${toolId} via npm...`);
            // Use npm update to get the latest version
            const updateCommand = `npm update -g ${packageName}`;
            try {
                SafeLogger_1.logger.info(`[Main] Running: ${updateCommand}`);
                const { stdout, stderr } = yield execAsync(updateCommand, {
                    env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                });
                SafeLogger_1.logger.info(`[Main] NPM update output: ${stdout}`);
                if (stderr && !stderr.includes('npm WARN')) {
                    SafeLogger_1.logger.warn(`[Main] NPM update stderr: ${stderr}`);
                }
                // Get the updated version
                let version = 'Unknown';
                try {
                    // For Claude Code, use claude --version
                    if (toolId === 'claude-code') {
                        const versionResult = yield execAsync('claude --version', {
                            env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                        });
                        // Parse version from output like "claude-code/1.0.86 darwin-arm64 node-v23.6.0"
                        const match = versionResult.stdout.match(/claude-code\/(\d+\.\d+\.\d+)/);
                        version = match ? match[1] : 'Unknown';
                    }
                    else if (toolId === 'gemini-cli') {
                        // For Gemini CLI, use gemini --version
                        const versionResult = yield execAsync('gemini --version', {
                            env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                        });
                        // Parse version from output - expecting format like "gemini-cli/0.1.18" or similar
                        const match = versionResult.stdout.match(/(?:gemini-cli\/|v?)(\d+\.\d+\.\d+)/);
                        version = match ? match[1] : 'Unknown';
                    }
                    else if (toolId === 'qwen-code') {
                        // For Qwen Code, use qwen --version
                        const versionResult = yield execAsync('qwen --version', {
                            env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                        });
                        // Parse version from output
                        const match = versionResult.stdout.match(/(?:qwen\/|v?)(\d+\.\d+\.\d+)/);
                        version = match ? match[1] : 'Unknown';
                    }
                    else if (toolId === 'openai-codex') {
                        // For OpenAI Codex, use codex --version
                        const versionResult = yield execAsync('codex --version', {
                            env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                        });
                        // Parse version from output
                        const match = versionResult.stdout.match(/codex-cli (\d+\.\d+\.\d+)/);
                        version = match ? match[1] : 'Unknown';
                    }
                    else if (toolId === 'cline') {
                        // For Cline, use cline-cli --version
                        const versionResult = yield execAsync('cline-cli --version', {
                            env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                        });
                        // Parse version from output
                        const match = versionResult.stdout.match(/(\d+\.\d+\.\d+)/);
                        version = match ? match[1] : 'Unknown';
                    }
                    else if (toolId === 'grok') {
                        // For Grok CLI, use grok --version
                        const versionResult = yield execAsync('grok --version', {
                            env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                        });
                        // Parse version from output
                        const match = versionResult.stdout.match(/(\d+\.\d+\.\d+)/);
                        version = match ? match[1] : 'Unknown';
                    }
                    else {
                        // For other tools, try to get version from npm list
                        const listResult = yield execAsync(`npm list -g ${packageName} --depth=0`, {
                            env: Object.assign(Object.assign({}, process.env), { PATH: `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}` })
                        });
                        const versionMatch = listResult.stdout.match(new RegExp(`${packageName}@(\\d+\\.\\d+\\.\\d+)`));
                        version = versionMatch ? versionMatch[1] : 'Unknown';
                    }
                }
                catch (versionError) {
                    SafeLogger_1.logger.warn(`[Main] Could not get version for ${toolId}:`, versionError);
                }
                // CRITICAL FIX: Clear the detector cache for this tool so UI refresh works
                SafeLogger_1.logger.info(`[Main] Clearing detector cache for ${toolId} after successful update`);
                detector_1.cliToolsDetector.clearCache(toolId);
                SafeLogger_1.logger.info(`[Main] ${toolId} updated successfully to version ${version}`);
                return { success: true, version, message: `Updated to version ${version}` };
            }
            catch (error) {
                SafeLogger_1.logger.error(`[Main] Failed to update ${toolId}:`, error);
                // Check if it's a permission error
                if (((_g = error.message) === null || _g === void 0 ? void 0 : _g.includes('EACCES')) || ((_h = error.message) === null || _h === void 0 ? void 0 : _h.includes('permission'))) {
                    return {
                        success: false,
                        error: 'Permission denied. Try running the app with elevated permissions or update manually with: ' + updateCommand
                    };
                }
                // Check if npm is not found
                if ((_j = error.message) === null || _j === void 0 ? void 0 : _j.includes('npm: command not found')) {
                    return {
                        success: false,
                        error: 'npm not found. Please ensure Node.js and npm are installed.'
                    };
                }
                return { success: false, error: error.message || 'Update failed' };
            }
        }
        catch (error) {
            SafeLogger_1.logger.error(`[Main] Unexpected error updating ${toolId}:`, error);
            return { success: false, error: error.message || 'Unexpected error occurred' };
        }
    }));
    // Uninstall CLI tool
    electron_1.ipcMain.handle('cli-tool-uninstall', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        var _k, _l, _m, _o, _p;
        SafeLogger_1.logger.info(`[Main] Uninstalling CLI tool: ${toolId}`);
        try {
            const { exec } = require('child_process');
            const { promisify } = require('util');
            const execAsync = promisify(exec);
            // Get tool configuration from registry
            const toolConfig = cli_tools_1.CLI_TOOLS_REGISTRY[toolId];
            if (!toolConfig) {
                SafeLogger_1.logger.error(`[Main] Unknown tool ID for uninstall: ${toolId}`);
                return { success: false, error: `Unknown tool: ${toolId}` };
            }
            // Map tool IDs to npm package names
            const npmPackages = {
                'claude-code': '@anthropic-ai/claude-code',
                'gemini-cli': '@google/gemini-cli',
                'qwen-code': '@qwen-code/qwen-code',
                'openai-codex': '@openai/codex',
                'cline': '@yaegaki/cline-cli',
                'grok': '@vibe-kit/grok-cli'
            };
            const packageName = npmPackages[toolId];
            if (!packageName) {
                SafeLogger_1.logger.error(`[Main] No package mapping for tool: ${toolId}`);
                return { success: false, error: `Cannot uninstall ${toolConfig.name}: package mapping not found` };
            }
            // Enhanced PATH for finding npm
            const enhancedPath = `/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:${process.env.PATH}`;
            SafeLogger_1.logger.info(`[Main] Running uninstall command: npm uninstall -g ${packageName}`);
            try {
                // Run the uninstall command
                const { stdout, stderr } = yield execAsync(`npm uninstall -g ${packageName}`, {
                    env: Object.assign(Object.assign({}, process.env), { PATH: enhancedPath }),
                    timeout: 60000 // 1 minute timeout for uninstall
                });
                SafeLogger_1.logger.info(`[Main] Uninstall output: ${stdout}`);
                if (stderr && !stderr.includes('WARN') && !stderr.includes('warning')) {
                    SafeLogger_1.logger.warn(`[Main] Uninstall stderr: ${stderr}`);
                }
                // Verify uninstallation by checking if command still exists
                try {
                    yield execAsync(`which ${toolConfig.command}`, {
                        env: Object.assign(Object.assign({}, process.env), { PATH: enhancedPath }),
                        timeout: 5000
                    });
                    // If we get here, the command still exists - uninstall may have failed
                    SafeLogger_1.logger.warn(`[Main] Command ${toolConfig.command} still exists after uninstall`);
                    // Try to determine if it's a different installation
                    const { stdout: pathOutput } = yield execAsync(`which ${toolConfig.command}`, {
                        env: Object.assign(Object.assign({}, process.env), { PATH: enhancedPath })
                    });
                    if (pathOutput.includes('/usr/local/bin') || pathOutput.includes('/opt/homebrew/bin')) {
                        // It's still in a global location, uninstall might have failed
                        return {
                            success: false,
                            error: `Tool appears to still be installed at ${pathOutput.trim()}. You may need to uninstall manually.`
                        };
                    }
                }
                catch (_q) {
                    // Command not found - uninstall was successful
                    SafeLogger_1.logger.info(`[Main] Command ${toolConfig.command} no longer exists - uninstall successful`);
                }
                // CRITICAL FIX: Clear the detector cache for this tool so UI refresh works
                SafeLogger_1.logger.info(`[Main] Clearing detector cache for ${toolId} after successful uninstall`);
                detector_1.cliToolsDetector.clearCache(toolId);
                // Also clean up any configuration files if they exist
                if (toolId === 'cline') {
                    // Clean up Cline configuration
                    const clineConfigDir = path.join(os.homedir(), '.cline_cli');
                    if (fs.existsSync(clineConfigDir)) {
                        SafeLogger_1.logger.info(`[Main] Removing Cline configuration directory: ${clineConfigDir}`);
                        try {
                            fs.rmSync(clineConfigDir, { recursive: true, force: true });
                        }
                        catch (e) {
                            SafeLogger_1.logger.warn(`[Main] Could not remove Cline config directory:`, e);
                        }
                    }
                }
                else if (toolId === 'grok') {
                    // Optionally clean up Grok configuration (but keep API key for reinstall)
                    SafeLogger_1.logger.info(`[Main] Keeping Grok configuration at ~/.grok for potential reinstall`);
                }
                SafeLogger_1.logger.info(`[Main] ${toolConfig.name} uninstalled successfully`);
                return { success: true, message: `${toolConfig.name} has been uninstalled` };
            }
            catch (error) {
                SafeLogger_1.logger.error(`[Main] Failed to uninstall ${toolConfig.name}:`, error);
                // Check for specific error conditions
                if (((_k = error.message) === null || _k === void 0 ? void 0 : _k.includes('EACCES')) || ((_l = error.message) === null || _l === void 0 ? void 0 : _l.includes('permission'))) {
                    return {
                        success: false,
                        error: 'Permission denied. Try running the app with elevated permissions or uninstall manually with: npm uninstall -g ' + packageName
                    };
                }
                // Check if npm is not found
                if ((_m = error.message) === null || _m === void 0 ? void 0 : _m.includes('npm: command not found')) {
                    return {
                        success: false,
                        error: 'npm not found. Please ensure Node.js and npm are installed.'
                    };
                }
                // Check if package is not installed
                if (((_o = error.message) === null || _o === void 0 ? void 0 : _o.includes('not found')) || ((_p = error.message) === null || _p === void 0 ? void 0 : _p.includes('missing'))) {
                    // This might actually be a success case - tool is already not installed
                    SafeLogger_1.logger.info(`[Main] Tool may already be uninstalled: ${error.message}`);
                    detector_1.cliToolsDetector.clearCache(toolId);
                    return { success: true, message: `${toolConfig.name} was not installed or has been removed` };
                }
                return { success: false, error: error.message || 'Uninstall failed' };
            }
        }
        catch (error) {
            SafeLogger_1.logger.error(`[Main] Unexpected error uninstalling ${toolId}:`, error);
            return { success: false, error: error.message || 'Unexpected error occurred' };
        }
    }));
    // Configure CLI tool
    electron_1.ipcMain.handle('cli-tool-configure', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info(`[Main] Configuring CLI tool: ${toolId}`);
        try {
            // Special handling for Cline - automatically configure with OpenRouter API key
            if (toolId === 'cline') {
                SafeLogger_1.logger.info('[Main] Special configuration for Cline - using OpenRouter API key from Hive');
                // Get OpenRouter API key from Hive's database
                if (!db) {
                    SafeLogger_1.logger.error('[Main] Database not initialized');
                    return {
                        success: false,
                        error: 'Database not initialized. Please restart the application.'
                    };
                }
                const apiKeyRow = yield new Promise((resolve, reject) => {
                    db.get('SELECT value FROM configurations WHERE key = ?', ['openrouter_api_key'], (err, row) => {
                        if (err)
                            reject(err);
                        else
                            resolve(row);
                    });
                });
                if (!apiKeyRow || !apiKeyRow.value) {
                    SafeLogger_1.logger.error('[Main] No OpenRouter API key found in Hive configuration');
                    return {
                        success: false,
                        error: 'OpenRouter API key not configured in Hive. Please configure it in Settings first.'
                    };
                }
                const openrouterKey = apiKeyRow.value;
                SafeLogger_1.logger.info('[Main] Found OpenRouter API key in Hive configuration');
                // Create Cline configuration directory if it doesn't exist
                const clineConfigDir = path.join(os.homedir(), '.cline_cli');
                if (!fs.existsSync(clineConfigDir)) {
                    fs.mkdirSync(clineConfigDir, { recursive: true });
                }
                // Create Cline settings file with OpenRouter configuration
                const clineSettingsPath = path.join(clineConfigDir, 'cline_cli_settings.json');
                // Read existing settings if they exist to preserve user preferences
                let existingSettings = {};
                if (fs.existsSync(clineSettingsPath)) {
                    try {
                        existingSettings = JSON.parse(fs.readFileSync(clineSettingsPath, 'utf-8'));
                    }
                    catch (e) {
                        SafeLogger_1.logger.warn('[Main] Could not parse existing Cline settings, creating new');
                    }
                }
                const clineSettings = {
                    globalState: Object.assign(Object.assign({}, existingSettings.globalState), { apiProvider: 'openrouter', apiModelId: '', openRouterApiKey: openrouterKey, awsRegion: '', awsUseCrossRegionInference: '', awsBedrockUsePromptCache: '', awsBedrockEndpoint: '', awsProfile: '', awsUseProfile: '', vertexProjectId: '', vertexRegion: '', openAiBaseUrl: '', openAiModelId: '', openAiModelInfo: '', ollamaModelId: '', ollamaBaseUrl: '', ollamaApiOptionsCtxNum: '', lmStudioModelId: '', lmStudioBaseUrl: '', anthropicBaseUrl: '', azureApiVersion: '', openRouterModelId: '', openRouterModelInfo: '', openRouterProviderSorting: '', liteLlmBaseUrl: '', liteLlmModelId: '', qwenApiLine: '', requestyModelId: '', requestyModelInfo: '', togetherModelId: '', asksageApiUrl: '', thinkingBudgetTokens: '', reasoningEffort: '', favoritedModelIds: '', autoApprovalSettings: {
                            enabled: false,
                            actions: {
                                readFiles: false,
                                editFiles: false,
                                executeSafeCommands: false,
                                useMcp: false
                            },
                            maxRequests: 20
                        } }),
                    settings: {
                        'cline.enableCheckpoints': false
                    }
                };
                // Write the configuration file
                fs.writeFileSync(clineSettingsPath, JSON.stringify(clineSettings, null, 2));
                // Create a keys file for Cline with the OpenRouter API key
                // Cline reads API keys from a separate keys file
                const keysPath = path.join(clineConfigDir, 'keys.json');
                const keysContent = {
                    openRouterApiKey: openrouterKey
                };
                fs.writeFileSync(keysPath, JSON.stringify(keysContent, null, 2));
                // Also create storage directory if it doesn't exist
                const storageDir = path.join(clineConfigDir, 'storage');
                if (!fs.existsSync(storageDir)) {
                    fs.mkdirSync(storageDir, { recursive: true });
                }
                SafeLogger_1.logger.info('[Main] Successfully configured Cline with OpenRouter API key from Hive');
                // Continue with Memory Service configuration for all tools
            }
            // 1. Get Memory Service port from ProcessManager
            const memoryServiceInfo = processManager.getProcessStatus('memory-service');
            const memoryServicePort = (memoryServiceInfo === null || memoryServiceInfo === void 0 ? void 0 : memoryServiceInfo.port) || 3457;
            const memoryServiceEndpoint = `http://localhost:${memoryServicePort}`;
            // 2. Register with Memory Service to get a token
            const http = require('http');
            const crypto = require('crypto');
            const registerPromise = new Promise((resolve, reject) => {
                const postData = JSON.stringify({
                    toolName: toolId
                });
                const options = {
                    hostname: 'localhost',
                    port: memoryServicePort,
                    path: '/api/v1/memory/register',
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Content-Length': Buffer.byteLength(postData)
                    }
                };
                const req = http.request(options, (res) => {
                    let data = '';
                    res.on('data', (chunk) => data += chunk);
                    res.on('end', () => {
                        try {
                            const result = JSON.parse(data);
                            if (result.token) {
                                resolve(result);
                            }
                            else {
                                reject(new Error('No token received from Memory Service'));
                            }
                        }
                        catch (e) {
                            reject(e);
                        }
                    });
                });
                req.on('error', reject);
                req.write(postData);
                req.end();
            });
            const registrationResult = yield registerPromise;
            const token = registrationResult.token;
            // 3. Save token to cli-tools-config.json
            const configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');
            let config = {};
            if (fs.existsSync(configPath)) {
                config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
            }
            config[toolId] = Object.assign(Object.assign({}, config[toolId]), { memoryService: {
                    endpoint: memoryServiceEndpoint,
                    token: token,
                    connectedAt: new Date().toISOString()
                } });
            fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
            // 4. Create MCP wrapper script
            const wrapperPath = path.join(os.homedir(), '.hive', 'memory-service-mcp-wrapper.js');
            const wrapperContent = `#!/usr/bin/env node
/**
 * MCP Wrapper for Hive Memory Service
 * This script provides an MCP-compatible interface to the Memory Service
 */

const { Server } = require('@modelcontextprotocol/sdk');
const fetch = require('node-fetch');

const ENDPOINT = process.env.MEMORY_SERVICE_ENDPOINT || '${memoryServiceEndpoint}';
const TOKEN = process.env.MEMORY_SERVICE_TOKEN;

class MemoryServiceMCP extends Server {
  constructor() {
    super({
      name: 'hive-memory-service',
      version: '1.0.0',
      description: 'Hive Consensus Memory Service'
    });

    this.registerTool({
      name: 'query_memory',
      description: 'Query the AI memory system for relevant learnings',
      parameters: {
        query: { type: 'string', required: true },
        limit: { type: 'number', default: 5 }
      },
      handler: this.queryMemory.bind(this)
    });

    this.registerTool({
      name: 'contribute_learning',
      description: 'Contribute a new learning to the memory system',
      parameters: {
        type: { type: 'string', required: true },
        category: { type: 'string', required: true },
        content: { type: 'string', required: true },
        code: { type: 'string' }
      },
      handler: this.contributeLearning.bind(this)
    });
  }

  async queryMemory({ query, limit = 5 }) {
    const response = await fetch(\`\${ENDPOINT}/api/v1/memory/query\`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': \`Bearer \${TOKEN}\`,
        'X-Client-Name': '${toolId}-mcp'
      },
      body: JSON.stringify({
        client: '${toolId}',
        context: { file: process.cwd() },
        query,
        options: { limit }
      })
    });

    return await response.json();
  }

  async contributeLearning({ type, category, content, code }) {
    const response = await fetch(\`\${ENDPOINT}/api/v1/memory/contribute\`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': \`Bearer \${TOKEN}\`,
        'X-Client-Name': '${toolId}-mcp'
      },
      body: JSON.stringify({
        source: '${toolId}',
        learning: {
          type,
          category,
          content,
          code,
          context: { file: process.cwd(), success: true }
        }
      })
    });

    return await response.json();
  }
}

// Start the MCP server
const server = new MemoryServiceMCP();
server.start();
`;
            fs.writeFileSync(wrapperPath, wrapperContent);
            fs.chmodSync(wrapperPath, '755');
            // 5. Update MCP configuration for Claude Code
            const mcpConfigPath = path.join(os.homedir(), '.claude', '.mcp.json');
            let mcpConfig = { servers: {} };
            if (fs.existsSync(mcpConfigPath)) {
                try {
                    mcpConfig = JSON.parse(fs.readFileSync(mcpConfigPath, 'utf-8'));
                }
                catch (_r) {
                    mcpConfig = { servers: {} };
                }
            }
            // Add or update the memory service server
            mcpConfig.servers['hive-memory-service'] = {
                command: 'node',
                args: [wrapperPath],
                env: {
                    MEMORY_SERVICE_ENDPOINT: memoryServiceEndpoint,
                    MEMORY_SERVICE_TOKEN: token
                },
                description: 'Hive Consensus Memory Service - AI memory and learning system'
            };
            // Ensure directory exists
            const mcpDir = path.dirname(mcpConfigPath);
            if (!fs.existsSync(mcpDir)) {
                fs.mkdirSync(mcpDir, { recursive: true });
            }
            fs.writeFileSync(mcpConfigPath, JSON.stringify(mcpConfig, null, 2));
            // Also configure MCP for Grok if it's being installed
            if (toolId === 'grok') {
                const grokMcpConfigPath = path.join(os.homedir(), '.grok', 'mcp-config.json');
                let grokMcpConfig = { servers: {} };
                if (fs.existsSync(grokMcpConfigPath)) {
                    try {
                        grokMcpConfig = JSON.parse(fs.readFileSync(grokMcpConfigPath, 'utf-8'));
                    }
                    catch (_s) {
                        grokMcpConfig = { servers: {} };
                    }
                }
                // Add or update the memory service server for Grok
                grokMcpConfig.servers['hive-memory-service'] = {
                    transport: 'stdio',
                    command: 'node',
                    args: [wrapperPath],
                    env: {
                        MEMORY_SERVICE_ENDPOINT: memoryServiceEndpoint,
                        MEMORY_SERVICE_TOKEN: token
                    }
                };
                // Ensure Grok directory exists
                const grokDir = path.dirname(grokMcpConfigPath);
                if (!fs.existsSync(grokDir)) {
                    fs.mkdirSync(grokDir, { recursive: true });
                }
                fs.writeFileSync(grokMcpConfigPath, JSON.stringify(grokMcpConfig, null, 2));
                SafeLogger_1.logger.info(`[Main] Successfully configured Grok MCP with Memory Service`);
            }
            SafeLogger_1.logger.info(`[Main] Successfully configured ${toolId} with Memory Service`);
            return {
                success: true,
                message: `${toolId} successfully connected to Memory Service`,
                token: token.substring(0, 8) + '...' // Show partial token for confirmation
            };
        }
        catch (error) {
            SafeLogger_1.logger.error(`[Main] Failed to configure ${toolId}:`, error);
            return {
                success: false,
                error: `Failed to configure: ${error.message || error}`
            };
        }
    }));
    // Check for updates
    electron_1.ipcMain.handle('cli-tools-check-updates', () => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[Main] Checking for CLI tool updates');
        // TODO: Implement update checking logic
        return [];
    }));
    // Launch CLI tool with folder selection and resume detection
    electron_1.ipcMain.handle('cli-tool-launch', (_, toolId, projectPath) => __awaiter(void 0, void 0, void 0, function* () {
        SafeLogger_1.logger.info(`[Main] Launching CLI tool: ${toolId}${projectPath ? ` in ${projectPath}` : ''}`);
        try {
            // If no project path provided, show folder selection dialog
            let selectedPath = projectPath;
            if (!selectedPath) {
                // Get proper tool name from registry
                const toolConfig = cli_tools_1.CLI_TOOLS_REGISTRY[toolId];
                const toolName = toolConfig ? toolConfig.name : toolId;
                const result = yield electron_1.dialog.showOpenDialog(mainWindow, {
                    properties: ['openDirectory'],
                    title: `Select folder to launch ${toolName}`,
                    buttonLabel: 'Launch Here'
                });
                if (result.canceled || !result.filePaths[0]) {
                    SafeLogger_1.logger.info('[Main] User canceled folder selection');
                    return { success: false, error: 'Folder selection canceled' };
                }
                selectedPath = result.filePaths[0];
            }
            SafeLogger_1.logger.info(`[Main] Selected folder: ${selectedPath}`);
            // Get the AI tools database instance
            SafeLogger_1.logger.info(`[Main] Getting AIToolsDatabase instance...`);
            let aiToolsDb;
            try {
                aiToolsDb = AIToolsDatabase_1.AIToolsDatabase.getInstance();
                SafeLogger_1.logger.info(`[Main] AIToolsDatabase instance obtained`);
            }
            catch (dbError) {
                SafeLogger_1.logger.error(`[Main] Failed to get AIToolsDatabase instance:`, dbError);
                SafeLogger_1.logger.error(`[Main] AIToolsDatabase error message:`, dbError === null || dbError === void 0 ? void 0 : dbError.message);
                SafeLogger_1.logger.error(`[Main] AIToolsDatabase error stack:`, dbError === null || dbError === void 0 ? void 0 : dbError.stack);
                // Continue without the database - just don't track launches
                aiToolsDb = null;
            }
            // Check if tool has been launched in this repository before
            let hasBeenLaunched = false;
            if (aiToolsDb) {
                hasBeenLaunched = aiToolsDb.hasBeenLaunchedBefore(toolId, selectedPath);
                SafeLogger_1.logger.info(`[Main] Database check - Tool: ${toolId}, Path: ${selectedPath}, Previously launched: ${hasBeenLaunched}`);
                // Get launch info for debugging
                const launchInfo = aiToolsDb.getLaunchInfo(toolId, selectedPath);
                if (launchInfo) {
                    SafeLogger_1.logger.info(`[Main] Previous launch info: Count: ${launchInfo.launch_count}, Last: ${launchInfo.last_launched_at}`);
                }
            }
            else {
                SafeLogger_1.logger.warn(`[Main] AIToolsDatabase not available, skipping launch tracking`);
            }
            // Determine the command to run
            let command;
            let apiKeyRow = null; // Store API key for Cline
            if (toolId === 'claude-code') {
                command = hasBeenLaunched ? 'claude --resume' : 'claude';
            }
            else if (toolId === 'gemini-cli') {
                // Gemini CLI doesn't support --resume, always use base command
                command = 'gemini';
            }
            else if (toolId === 'qwen-code') {
                command = 'qwen';
            }
            else if (toolId === 'openai-codex') {
                command = 'codex';
            }
            else if (toolId === 'grok') {
                // SMART GROK LAUNCH: Check if API key is configured
                // Grok stores config in ~/.grok/user-settings.json
                const grokConfigPath = path.join(os.homedir(), '.grok', 'user-settings.json');
                let hasGrokApiKey = false;
                if (fs.existsSync(grokConfigPath)) {
                    try {
                        const grokConfig = JSON.parse(fs.readFileSync(grokConfigPath, 'utf-8'));
                        hasGrokApiKey = !!(grokConfig.apiKey || process.env.GROK_API_KEY);
                    }
                    catch (_t) {
                        // Config exists but couldn't be parsed
                    }
                }
                // Check environment variable as fallback
                if (!hasGrokApiKey && process.env.GROK_API_KEY) {
                    hasGrokApiKey = true;
                }
                if (!hasGrokApiKey) {
                    // First-time launch: We'll create a setup wizard in the terminal
                    SafeLogger_1.logger.info('[Main] Grok API key not configured, will launch setup wizard');
                    // We'll handle this in the terminal creation with a special flag
                    command = 'grok:setup'; // Special command to trigger our setup wizard
                }
                else {
                    // Normal launch - API key is configured
                    command = 'grok';
                }
            }
            else if (toolId === 'cline') {
                // Always sync Cline configuration with latest OpenRouter API key from Hive
                apiKeyRow = yield new Promise((resolve) => {
                    if (!db) {
                        resolve(null);
                        return;
                    }
                    db.get('SELECT value FROM configurations WHERE key = ?', ['openrouter_api_key'], (err, row) => {
                        resolve(row);
                    });
                });
                if (apiKeyRow && apiKeyRow.value) {
                    // Update Cline configuration with current API key
                    const clineConfigDir = path.join(os.homedir(), '.cline_cli');
                    const clineSettingsPath = path.join(clineConfigDir, 'cline_cli_settings.json');
                    const keysPath = path.join(clineConfigDir, 'keys.json');
                    // Ensure directories exist
                    if (!fs.existsSync(clineConfigDir)) {
                        fs.mkdirSync(clineConfigDir, { recursive: true });
                    }
                    // Read existing settings to preserve user preferences
                    let existingSettings = {};
                    if (fs.existsSync(clineSettingsPath)) {
                        try {
                            existingSettings = JSON.parse(fs.readFileSync(clineSettingsPath, 'utf-8'));
                        }
                        catch (e) {
                            SafeLogger_1.logger.warn('[Main] Could not parse existing Cline settings');
                        }
                    }
                    // Update or create settings file
                    const clineSettings = {
                        globalState: Object.assign(Object.assign({}, existingSettings.globalState), { apiProvider: 'openrouter', apiModelId: '', openRouterApiKey: apiKeyRow.value, openRouterModelId: '', openRouterModelInfo: '', autoApprovalSettings: {
                                enabled: false,
                                actions: {
                                    readFiles: false,
                                    editFiles: false,
                                    executeSafeCommands: false,
                                    useMcp: false
                                },
                                maxRequests: 20
                            } }),
                        settings: {
                            'cline.enableCheckpoints': false
                        }
                    };
                    // Fill in other empty fields
                    const fields = ['awsRegion', 'awsUseCrossRegionInference', 'awsBedrockUsePromptCache',
                        'awsBedrockEndpoint', 'awsProfile', 'awsUseProfile', 'vertexProjectId',
                        'vertexRegion', 'openAiBaseUrl', 'openAiModelId', 'openAiModelInfo',
                        'ollamaModelId', 'ollamaBaseUrl', 'ollamaApiOptionsCtxNum', 'lmStudioModelId',
                        'lmStudioBaseUrl', 'anthropicBaseUrl', 'azureApiVersion', 'openRouterProviderSorting',
                        'liteLlmBaseUrl', 'liteLlmModelId', 'qwenApiLine', 'requestyModelId',
                        'requestyModelInfo', 'togetherModelId', 'asksageApiUrl', 'thinkingBudgetTokens',
                        'reasoningEffort', 'favoritedModelIds'];
                    fields.forEach(field => {
                        if (!(field in clineSettings.globalState)) {
                            clineSettings.globalState[field] = '';
                        }
                    });
                    fs.writeFileSync(clineSettingsPath, JSON.stringify(clineSettings, null, 2));
                    // Update keys file with current API key
                    const keysContent = {
                        openRouterApiKey: apiKeyRow.value
                    };
                    fs.writeFileSync(keysPath, JSON.stringify(keysContent, null, 2));
                    SafeLogger_1.logger.info('[Main] Updated Cline config with current OpenRouter API key from Hive');
                }
                // We'll pass the API key through environment variables in the terminal creation
                command = 'cline-cli task';
            }
            else {
                // For other tools, just use their base command
                command = toolId.replace('-cli', '').replace('-code', '');
            }
            SafeLogger_1.logger.info(`[Main] Using command: ${command} (previously launched: ${hasBeenLaunched})`);
            // Record this launch in the database
            if (aiToolsDb) {
                aiToolsDb.recordLaunch(toolId, selectedPath, {
                    context: {
                        resumeUsed: hasBeenLaunched,
                        launchTime: new Date().toISOString()
                    }
                });
            }
            // Send IPC to renderer to create a terminal tab with the tool
            if (mainWindow && !mainWindow.isDestroyed()) {
                // FIRST: Update the global folder context to the selected path
                // This ensures Explorer, Source Control, and Status Bar all update
                SafeLogger_1.logger.info(`[Main] Sending menu-open-folder event for: ${selectedPath}`);
                SafeLogger_1.logger.info(`[Main] MainWindow exists: ${!!mainWindow}, isDestroyed: ${mainWindow === null || mainWindow === void 0 ? void 0 : mainWindow.isDestroyed()}`);
                mainWindow.webContents.send('menu-open-folder', selectedPath);
                // THEN: After a small delay to ensure folder context is set, launch the terminal
                setTimeout(() => __awaiter(void 0, void 0, void 0, function* () {
                    // Get proper tool name from registry
                    const toolConfig = cli_tools_1.CLI_TOOLS_REGISTRY[toolId];
                    const toolName = toolConfig ? toolConfig.name : toolId;
                    // Prepare environment variables for tools if needed
                    let env = {};
                    // Cline needs OpenRouter API key (special case - we manage its API key)
                    if (toolId === 'cline') {
                        // Fetch the OpenRouter API key from database
                        try {
                            const apiKeyRow = yield new Promise((resolve, reject) => {
                                if (!db) {
                                    reject(new Error('Database not initialized'));
                                    return;
                                }
                                db.get('SELECT value FROM configurations WHERE key = ?', ['openrouter_api_key'], (err, row) => {
                                    if (err)
                                        reject(err);
                                    else
                                        resolve(row);
                                });
                            });
                            if (apiKeyRow && apiKeyRow.value) {
                                // Cline CLI seems to be looking for OPENAI_API_KEY even when configured for OpenRouter
                                // This appears to be a bug in Cline CLI, so we'll set both variables
                                env.OPENAI_API_KEY = apiKeyRow.value; // Work around Cline CLI bug
                                env.OPEN_ROUTER_API_KEY = apiKeyRow.value;
                                env.OPENROUTER_API_KEY = apiKeyRow.value; // Try different variations
                                SafeLogger_1.logger.info('[Main] Adding OpenRouter API key to environment for Cline (multiple env vars for compatibility)');
                            }
                            else {
                                SafeLogger_1.logger.warn('[Main] No OpenRouter API key found for Cline');
                            }
                        }
                        catch (error) {
                            SafeLogger_1.logger.error('[Main] Error fetching OpenRouter API key for Cline:', error);
                        }
                    }
                    SafeLogger_1.logger.info(`[Main] Sending launch-ai-tool-terminal event with command: ${command}`);
                    SafeLogger_1.logger.info(`[Main] Event data:`, {
                        toolId: toolId,
                        toolName: toolName,
                        command: command,
                        cwd: selectedPath,
                        hasEnv: !!env
                    });
                    // Check if mainWindow is ready
                    if (!mainWindow.webContents.isLoading()) {
                        SafeLogger_1.logger.info(`[Main] Window is ready, sending event now`);
                        mainWindow.webContents.send('launch-ai-tool-terminal', {
                            toolId: toolId,
                            toolName: toolName,
                            command: command,
                            cwd: selectedPath,
                            env: env // Pass environment variables
                        });
                        SafeLogger_1.logger.info(`[Main] Event sent successfully`);
                    }
                    else {
                        SafeLogger_1.logger.warn(`[Main] Window is still loading, waiting for ready...`);
                        mainWindow.webContents.once('did-finish-load', () => {
                            SafeLogger_1.logger.info(`[Main] Window finished loading, sending event now`);
                            mainWindow.webContents.send('launch-ai-tool-terminal', {
                                toolId: toolId,
                                toolName: toolName,
                                command: command,
                                cwd: selectedPath,
                                env: env // Pass environment variables
                            });
                            SafeLogger_1.logger.info(`[Main] Event sent successfully after wait`);
                        });
                    }
                }), 100); // Small delay to ensure folder opens first
            }
            SafeLogger_1.logger.info(`[Main] Completed launch sequence for ${toolId} in ${selectedPath}`);
            return { success: true, path: selectedPath, command: command };
        }
        catch (error) {
            SafeLogger_1.logger.error(`[Main] Failed to launch ${toolId}:`, error);
            SafeLogger_1.logger.error(`[Main] Error message:`, error === null || error === void 0 ? void 0 : error.message);
            SafeLogger_1.logger.error(`[Main] Error stack:`, error === null || error === void 0 ? void 0 : error.stack);
            return { success: false, error: (error === null || error === void 0 ? void 0 : error.message) || String(error) };
        }
    }));
    // TODO: Implement progress events when installation logic is added
};
// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
electron_1.app.on('ready', () => __awaiter(void 0, void 0, void 0, function* () {
    var _a;
    try {
        // Use StartupOrchestrator for clean, visual startup
        const orchestrator = new StartupOrchestrator_1.StartupOrchestrator({
            initDatabase,
            initializeProcessManager,
            registerMemoryServiceHandlers,
            registerGitHandlers,
            registerFileSystemHandlers,
            registerDialogHandlers,
            registerSimpleCliToolHandlers,
            processManager
        });
        // The orchestrator will handle all initialization and show splash screen
        const result = yield orchestrator.showSplashAndInitialize(createWindow);
        if (!result.success) {
            SafeLogger_1.logger.error('[Main] Startup failed:', result.error);
            electron_1.dialog.showErrorBox('Startup Failed', `Unable to initialize required services.\n\n${((_a = result.error) === null || _a === void 0 ? void 0 : _a.message) || 'Unknown error'}`);
            electron_1.app.quit();
            return;
        }
        // Success - app is now running with main window shown
        SafeLogger_1.logger.info('[Main] ✅ Application started successfully');
    }
    catch (error) {
        SafeLogger_1.logger.error('[Main] Unexpected startup error:', error);
        electron_1.dialog.showErrorBox('Startup Error', `An unexpected error occurred during startup.\n\n${error}`);
        electron_1.app.quit();
    }
}));
// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
electron_1.app.on('window-all-closed', () => __awaiter(void 0, void 0, void 0, function* () {
    // Clean up all processes before quitting
    yield processManager.cleanup();
    if (process.platform !== 'darwin') {
        electron_1.app.quit();
    }
}));
electron_1.app.on('activate', () => {
    // On OS X it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (electron_1.BrowserWindow.getAllWindows().length === 0) {
        createWindow();
    }
});
// Track cleanup state to prevent duplicate cleanup
let isCleaningUp = false;
// Unified cleanup function
function performCleanup(reason) {
    return __awaiter(this, void 0, void 0, function* () {
        if (isCleaningUp) {
            SafeLogger_1.logger.info(`[Main] Cleanup already in progress (triggered by ${reason})`);
            return;
        }
        isCleaningUp = true;
        SafeLogger_1.logger.info(`[Main] Starting cleanup (reason: ${reason})...`);
        try {
            // Clean up all terminals first
            (0, terminal_ipc_handlers_1.cleanupTerminals)();
            // Stop memory service if running
            yield processManager.stopProcess('memory-service');
            // Clean up all other processes
            yield processManager.cleanup();
            SafeLogger_1.logger.info('[Main] Cleanup completed successfully');
        }
        catch (error) {
            SafeLogger_1.logger.error('[Main] Error during cleanup:', error);
        }
    });
}
// Clean up processes on app quit
electron_1.app.on('before-quit', (event) => __awaiter(void 0, void 0, void 0, function* () {
    event.preventDefault();
    yield performCleanup('before-quit');
    electron_1.app.exit(0);
}));
// Handle unexpected termination
process.on('SIGINT', () => __awaiter(void 0, void 0, void 0, function* () {
    yield performCleanup('SIGINT');
    process.exit(0);
}));
process.on('SIGTERM', () => __awaiter(void 0, void 0, void 0, function* () {
    yield performCleanup('SIGTERM');
    process.exit(0);
}));
// Handle uncaught exceptions and unhandled rejections
process.on('uncaughtException', (error) => __awaiter(void 0, void 0, void 0, function* () {
    SafeLogger_1.logger.error('[Main] Uncaught exception:', error);
    yield performCleanup('uncaughtException');
    process.exit(1);
}));
process.on('unhandledRejection', (reason) => __awaiter(void 0, void 0, void 0, function* () {
    SafeLogger_1.logger.error('[Main] Unhandled rejection:', reason);
    yield performCleanup('unhandledRejection');
    process.exit(1);
}));
// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.
// Set up IPC handlers for backend communication
electron_1.ipcMain.handle('backend-health', () => __awaiter(void 0, void 0, void 0, function* () {
    try {
        const backendInfo = processManager.getProcessStatus('websocket-backend');
        const backendPort = (backendInfo === null || backendInfo === void 0 ? void 0 : backendInfo.port) || 8765;
        const response = yield fetch(`http://localhost:${backendPort}/health`);
        return yield response.json();
    }
    catch (error) {
        throw error;
    }
}));
electron_1.ipcMain.handle('backend-test', () => __awaiter(void 0, void 0, void 0, function* () {
    try {
        const backendInfo = processManager.getProcessStatus('websocket-backend');
        const backendPort = (backendInfo === null || backendInfo === void 0 ? void 0 : backendInfo.port) || 8765;
        const response = yield fetch(`http://localhost:${backendPort}/test`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify('Hello from Electron via IPC')
        });
        return yield response.json();
    }
    catch (error) {
        throw error;
    }
}));
electron_1.ipcMain.handle('backend-consensus', (_, query) => __awaiter(void 0, void 0, void 0, function* () {
    try {
        const backendInfo = processManager.getProcessStatus('websocket-backend');
        const backendPort = (backendInfo === null || backendInfo === void 0 ? void 0 : backendInfo.port) || 8765;
        const response = yield fetch(`http://localhost:${backendPort}/api/consensus`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ query })
        });
        return yield response.json();
    }
    catch (error) {
        throw error;
    }
}));
electron_1.ipcMain.handle('backend-consensus-quick', (_, data) => __awaiter(void 0, void 0, void 0, function* () {
    try {
        const backendInfo = processManager.getProcessStatus('websocket-backend');
        const backendPort = (backendInfo === null || backendInfo === void 0 ? void 0 : backendInfo.port) || 8765;
        const response = yield fetch(`http://127.0.0.1:${backendPort}/api/consensus/quick`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        return yield response.json();
    }
    catch (error) {
        SafeLogger_1.logger.error('Quick consensus error:', error);
        throw error;
    }
}));
// WebSocket proxy - main process handles WebSocket connection
const WebSocket = require('ws');
let wsConnection = null;
let wsCallbacks = new Map();
electron_1.ipcMain.handle('websocket-connect', (event, url) => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => {
        try {
            if (wsConnection) {
                wsConnection.close();
            }
            // If the URL is for our backend, use the dynamic port
            if (url.includes('127.0.0.1:8765') || url.includes('localhost:8765')) {
                const backendInfo = processManager.getProcessStatus('websocket-backend');
                const backendPort = (backendInfo === null || backendInfo === void 0 ? void 0 : backendInfo.port) || 8765;
                url = url.replace(':8765', `:${backendPort}`);
                SafeLogger_1.logger.info(`[Main] Using dynamic WebSocket port: ${backendPort}`);
            }
            wsConnection = new WebSocket(url);
            wsConnection.on('open', () => {
                SafeLogger_1.logger.info('WebSocket connected in main process');
                resolve({ connected: true });
            });
            wsConnection.on('message', (data) => {
                // Forward message to renderer
                event.sender.send('websocket-message', data.toString());
            });
            wsConnection.on('error', (error) => {
                SafeLogger_1.logger.error('WebSocket error in main:', error);
                event.sender.send('websocket-error', error.message);
                reject(error);
            });
            wsConnection.on('close', () => {
                SafeLogger_1.logger.info('WebSocket closed in main process');
                event.sender.send('websocket-closed');
                wsConnection = null;
            });
        }
        catch (error) {
            reject(error);
        }
    });
}));
electron_1.ipcMain.handle('websocket-send', (_, message) => __awaiter(void 0, void 0, void 0, function* () {
    // Check if WebSocket is open
    if (wsConnection && wsConnection.readyState === WebSocket.OPEN) {
        wsConnection.send(message);
        return { sent: true };
    }
    // If not connected, try to reconnect once
    SafeLogger_1.logger.info('WebSocket not ready, attempting reconnection...');
    try {
        yield new Promise((resolve, reject) => {
            const timeout = setTimeout(() => reject('Connection timeout'), 3000);
            if (!wsConnection || wsConnection.readyState === WebSocket.CLOSED) {
                // Get the actual backend port dynamically
                const backendInfo = processManager.getProcessStatus('websocket-backend');
                const backendPort = (backendInfo === null || backendInfo === void 0 ? void 0 : backendInfo.port) || 8765;
                // Reconnect to WebSocket using dynamic port
                wsConnection = new WebSocket(`ws://127.0.0.1:${backendPort}/ws`);
                wsConnection.once('open', () => {
                    clearTimeout(timeout);
                    SafeLogger_1.logger.info('WebSocket reconnected successfully');
                    resolve(true);
                });
                wsConnection.once('error', (err) => {
                    clearTimeout(timeout);
                    reject(err);
                });
                // Re-attach message handler for all windows
                wsConnection.on('message', (data) => {
                    // Send to all windows
                    electron_1.BrowserWindow.getAllWindows().forEach(window => {
                        window.webContents.send('websocket-message', data.toString());
                    });
                });
            }
            else if (wsConnection.readyState === WebSocket.CONNECTING) {
                // Wait for existing connection
                wsConnection.once('open', () => {
                    clearTimeout(timeout);
                    resolve(true);
                });
            }
            else {
                reject('Unknown WebSocket state');
            }
        });
        // Now send the message
        wsConnection.send(message);
        return { sent: true };
    }
    catch (error) {
        SafeLogger_1.logger.error('Failed to reconnect WebSocket:', error);
        throw new Error('WebSocket not connected and reconnection failed');
    }
}));
electron_1.ipcMain.handle('websocket-close', () => __awaiter(void 0, void 0, void 0, function* () {
    if (wsConnection) {
        wsConnection.close();
        wsConnection = null;
    }
    return { closed: true };
}));
// Settings API handlers
electron_1.ipcMain.handle('settings-load', () => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => {
        if (!db) {
            reject('Database not initialized');
            return;
        }
        const settings = {};
        // Load API keys from configurations table (matching Rust implementation)
        db.get('SELECT value FROM configurations WHERE key = ?', ['openrouter_api_key'], (err, row) => {
            if (row)
                settings.openrouterKey = row.value;
            db.get('SELECT value FROM configurations WHERE key = ?', ['hive_license_key'], (err2, row2) => {
                if (row2)
                    settings.hiveKey = row2.value;
                // Load active profile from consensus_settings table (matching Rust implementation)
                db.get('SELECT value FROM consensus_settings WHERE key = ?', ['active_profile_id'], (err3, row3) => {
                    if (row3) {
                        settings.activeProfileId = row3.value;
                        // Also get the profile name for better matching
                        db.get('SELECT profile_name FROM consensus_profiles WHERE id = ?', [row3.value], (errName, rowName) => {
                            if (rowName)
                                settings.activeProfileName = rowName.profile_name;
                        });
                    }
                    // Load license tier and usage information
                    db.get('SELECT value FROM configurations WHERE key = ?', ['hive_tier'], (errTier, rowTier) => {
                        if (rowTier)
                            settings.hiveTier = rowTier.value;
                        db.get('SELECT value FROM configurations WHERE key = ?', ['hive_daily_limit'], (errLimit, rowLimit) => {
                            if (rowLimit)
                                settings.hiveDailyLimit = parseInt(rowLimit.value);
                            db.get('SELECT value FROM configurations WHERE key = ?', ['hive_remaining'], (errRemaining, rowRemaining) => {
                                if (rowRemaining)
                                    settings.hiveRemaining = parseInt(rowRemaining.value);
                                // Load other settings
                                db.get('SELECT value FROM configurations WHERE key = ?', ['auto_save'], (err4, row4) => {
                                    if (row4)
                                        settings.autoSave = row4.value === 'true';
                                    db.get('SELECT value FROM configurations WHERE key = ?', ['show_costs'], (err5, row5) => {
                                        if (row5)
                                            settings.showCosts = row5.value === 'true';
                                        db.get('SELECT value FROM configurations WHERE key = ?', ['max_daily_conversations'], (err6, row6) => {
                                            if (row6)
                                                settings.maxDailyConversations = row6.value;
                                            resolve(settings);
                                        });
                                    });
                                });
                            });
                        });
                    });
                });
            });
        });
    });
}));
electron_1.ipcMain.handle('settings-test-keys', (_, { openrouterKey, hiveKey }) => __awaiter(void 0, void 0, void 0, function* () {
    var _b, _c, _d, _e;
    const result = { openrouterValid: false, hiveValid: false, licenseInfo: null };
    // Test OpenRouter key
    if (openrouterKey && openrouterKey.startsWith('sk-or-')) {
        try {
            const response = yield fetch('https://openrouter.ai/api/v1/models', {
                headers: {
                    'Authorization': `Bearer ${openrouterKey}`,
                    'HTTP-Referer': 'https://hivetechs.io',
                    'X-Title': 'hive-ai'
                }
            });
            result.openrouterValid = response.status === 200;
        }
        catch (error) {
            SafeLogger_1.logger.error('Failed to test OpenRouter key:', error);
        }
    }
    // Test Hive key - real D1 authentication
    if (hiveKey) {
        const upperKey = hiveKey.toUpperCase();
        if (upperKey.startsWith('HIVE-')) {
            const parts = upperKey.split('-');
            // Validate format first (HIVE-XXXX-XXXX-XXXX or longer)
            if (parts.length >= 4 && parts.slice(1).every((segment) => segment.length === 4 && /^[A-Z0-9]{4}$/.test(segment))) {
                try {
                    // Create device fingerprint (matching Rust implementation)
                    const crypto = require('crypto');
                    const hostname = os.hostname();
                    const username = os.userInfo().username;
                    const platform = os.platform();
                    const arch = os.arch();
                    const release = os.release();
                    const cpus = os.cpus().length;
                    const memory = Math.floor(os.totalmem() / 1024 / 1024); // MB
                    const deviceData = {
                        platform,
                        arch,
                        release,
                        cpus,
                        memory
                    };
                    const deviceString = JSON.stringify(deviceData);
                    const fingerprint = crypto.createHash('sha256')
                        .update(deviceString)
                        .digest('hex')
                        .substring(0, 32);
                    // Make request to Cloudflare D1 gateway
                    const response = yield fetch('https://gateway.hivetechs.io/v1/session/validate', {
                        method: 'POST',
                        headers: {
                            'Authorization': `Bearer ${upperKey}`,
                            'Content-Type': 'application/json',
                            'User-Agent': 'hive-electron/2.0.0'
                        },
                        body: JSON.stringify({
                            client_id: 'hive-tools',
                            session_token: upperKey,
                            fingerprint: fingerprint,
                            nonce: String(Date.now())
                        })
                    });
                    if (response.ok) {
                        const data = yield response.json();
                        // Log the D1 response to understand what fields are available
                        SafeLogger_1.logger.info('D1 validation response:', JSON.stringify(data, null, 2));
                        if (data.valid) {
                            // Parse tier information
                            const tier = data.tier || ((_b = data.user) === null || _b === void 0 ? void 0 : _b.subscription_tier) || 'free';
                            const dailyLimit = data.daily_limit || ((_c = data.limits) === null || _c === void 0 ? void 0 : _c.daily) || 10;
                            const email = data.email || ((_d = data.user) === null || _d === void 0 ? void 0 : _d.email) || '';
                            const userId = data.user_id || ((_e = data.user) === null || _e === void 0 ? void 0 : _e.id) || '';
                            // Check if D1 returned usage information
                            let remaining = undefined;
                            let dailyUsed = undefined;
                            // D1 is the source of truth for usage - only use if provided
                            if (data.usage) {
                                // D1 returned usage info - this is authoritative
                                remaining = data.usage.remaining;
                                const limit = data.usage.limit || dailyLimit;
                                // Calculate used from remaining (D1 tracks this)
                                if (remaining !== undefined && limit !== undefined) {
                                    // Handle "unlimited" case where remaining might be max value
                                    if (remaining === 4294967295 || remaining === 2147483647) {
                                        dailyUsed = 0;
                                        remaining = 'unlimited';
                                    }
                                    else {
                                        dailyUsed = Math.max(0, limit - remaining);
                                    }
                                }
                            }
                            // Build license info object
                            const licenseInfo = {
                                valid: true,
                                tier: tier.charAt(0).toUpperCase() + tier.slice(1).toLowerCase(),
                                dailyLimit: dailyLimit,
                                email: email,
                                userId: userId,
                                features: data.features || ['consensus']
                            };
                            // Only include usage data if D1 provided it
                            if (remaining !== undefined) {
                                licenseInfo.remaining = remaining;
                            }
                            if (dailyUsed !== undefined) {
                                licenseInfo.dailyUsed = dailyUsed;
                            }
                            result.hiveValid = true;
                            result.licenseInfo = licenseInfo;
                            // Store validated license info in database
                            if (db) {
                                const timestamp = new Date().toISOString();
                                db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
                   VALUES (?, ?, ?, ?, ?, ?)
                   ON CONFLICT(key) DO UPDATE SET
                   value = excluded.value,
                   updated_at = excluded.updated_at`, ['hive_tier', tier, 0, userId || 'default', timestamp, timestamp]);
                                db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
                   VALUES (?, ?, ?, ?, ?, ?)
                   ON CONFLICT(key) DO UPDATE SET
                   value = excluded.value,
                   updated_at = excluded.updated_at`, ['hive_daily_limit', String(dailyLimit), 0, userId || 'default', timestamp, timestamp]);
                                // Only store remaining if D1 provided it
                                if (remaining !== undefined) {
                                    db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
                     VALUES (?, ?, ?, ?, ?, ?)
                     ON CONFLICT(key) DO UPDATE SET
                     value = excluded.value,
                     updated_at = excluded.updated_at`, ['hive_remaining', String(remaining), 0, userId || 'default', timestamp, timestamp]);
                                }
                            }
                        }
                        else {
                            result.hiveValid = false;
                            result.licenseInfo = {
                                valid: false,
                                error: data.error || 'Invalid license key'
                            };
                        }
                    }
                    else {
                        // Handle error responses
                        const errorText = yield response.text();
                        SafeLogger_1.logger.error(`License validation failed: ${response.status} - ${errorText}`);
                        result.hiveValid = false;
                        result.licenseInfo = {
                            valid: false,
                            error: `Validation failed: ${response.status}`
                        };
                    }
                }
                catch (error) {
                    SafeLogger_1.logger.error('Failed to validate Hive license:', error);
                    result.hiveValid = false;
                    result.licenseInfo = {
                        valid: false,
                        error: 'Network error - unable to validate license'
                    };
                }
            }
            else {
                result.hiveValid = false;
                result.licenseInfo = {
                    valid: false,
                    error: 'Invalid license key format'
                };
            }
        }
        else {
            result.hiveValid = false;
            result.licenseInfo = {
                valid: false,
                error: 'License key must start with HIVE-'
            };
        }
    }
    return result;
}));
electron_1.ipcMain.handle('settings-save-keys', (_, { openrouterKey, hiveKey }) => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => {
        if (!db) {
            reject('Database not initialized');
            return;
        }
        const timestamp = new Date().toISOString();
        // Save OpenRouter key
        if (openrouterKey) {
            db.run('INSERT OR REPLACE INTO configuration (key, value, updated_at) VALUES (?, ?, ?)', ['openrouter_api_key', openrouterKey, timestamp], (err) => {
                if (err) {
                    reject(err);
                    return;
                }
                // Save Hive key
                if (hiveKey) {
                    db.run('INSERT OR REPLACE INTO configuration (key, value, updated_at) VALUES (?, ?, ?)', ['hive_license_key', hiveKey, timestamp], (err2) => {
                        if (err2) {
                            reject(err2);
                        }
                        else {
                            resolve(true);
                        }
                    });
                }
                else {
                    resolve(true);
                }
            });
        }
        else if (hiveKey) {
            db.run('INSERT OR REPLACE INTO configuration (key, value, updated_at) VALUES (?, ?, ?)', ['hive_license_key', hiveKey, timestamp], (err) => {
                if (err) {
                    reject(err);
                }
                else {
                    resolve(true);
                }
            });
        }
        else {
            resolve(true);
        }
    });
}));
electron_1.ipcMain.handle('settings-save-profile', (_, profile) => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => {
        if (!db) {
            reject('Database not initialized');
            return;
        }
        const timestamp = new Date().toISOString();
        // Insert or update the profile (no is_default column in actual database)
        db.run(`INSERT OR REPLACE INTO consensus_profiles 
       (id, profile_name, generator_model, refiner_model, validator_model, curator_model, updated_at) 
       VALUES (?, ?, ?, ?, ?, ?, ?)`, [profile.id, profile.name, profile.generator, profile.refiner, profile.validator, profile.curator, timestamp], (err2) => {
            if (err2) {
                reject(err2);
            }
            else {
                // Save to consensus_settings table (matching Rust implementation)
                db.run(`INSERT INTO consensus_settings (key, value, updated_at) 
             VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at`, ['active_profile_id', profile.id, timestamp], (err3) => {
                    if (err3) {
                        reject(err3);
                    }
                    else {
                        resolve(true);
                    }
                });
            }
        });
    });
}));
electron_1.ipcMain.handle('settings-save-all', (_, settings) => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const timestamp = new Date().toISOString();
            const userId = 'default';
            // Save API keys to configurations table (matching Rust implementation)
            if (settings.openrouterKey || settings.hiveKey) {
                if (settings.openrouterKey) {
                    db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at`, ['openrouter_api_key', settings.openrouterKey, 0, userId, timestamp, timestamp]);
                }
                if (settings.hiveKey) {
                    db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at`, ['hive_license_key', settings.hiveKey, 0, userId, timestamp, timestamp]);
                }
            }
            // Save profile to consensus_settings (matching Rust implementation)
            if (settings.selectedProfile) {
                // Save to consensus_settings table
                db.run(`INSERT INTO consensus_settings (key, value, updated_at) 
           VALUES (?, ?, ?)
           ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at`, ['active_profile_id', settings.selectedProfile, timestamp], (err) => {
                    if (err)
                        SafeLogger_1.logger.error('Failed to save active profile:', err);
                });
                // Note: No is_default column in actual consensus_profiles table
            }
            // Save other settings to configurations table
            if (settings.autoSave !== undefined) {
                db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
           VALUES (?, ?, ?, ?, ?, ?)
           ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at`, ['auto_save', settings.autoSave.toString(), 0, userId, timestamp, timestamp]);
            }
            if (settings.showCosts !== undefined) {
                db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
           VALUES (?, ?, ?, ?, ?, ?)
           ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at`, ['show_costs', settings.showCosts.toString(), 0, userId, timestamp, timestamp]);
            }
            if (settings.maxDailyConversations !== undefined) {
                db.run(`INSERT INTO configurations (key, value, encrypted, user_id, created_at, updated_at) 
           VALUES (?, ?, ?, ?, ?, ?)
           ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at`, ['max_daily_conversations', settings.maxDailyConversations.toString(), 0, userId, timestamp, timestamp]);
            }
            resolve(true);
        }
        catch (error) {
            reject(error);
        }
    }));
}));
electron_1.ipcMain.handle('settings-load-profiles', () => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => {
        if (!db) {
            reject('Database not initialized');
            return;
        }
        db.all('SELECT * FROM consensus_profiles ORDER BY profile_name', [], (err, rows) => {
            if (err) {
                reject(err);
            }
            else {
                // Map to expected format
                const profiles = rows.map((row) => ({
                    id: row.id,
                    name: row.profile_name,
                    generator: row.generator_model,
                    refiner: row.refiner_model,
                    validator: row.validator_model,
                    curator: row.curator_model
                }));
                resolve(profiles);
            }
        });
    });
}));
electron_1.ipcMain.handle('settings-load-models', () => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => {
        if (!db) {
            reject('Database not initialized');
            return;
        }
        // First check if openrouter_models table exists
        db.get("SELECT name FROM sqlite_master WHERE type='table' AND name='openrouter_models'", [], (err, row) => {
            if (err || !row) {
                // Table doesn't exist, return empty array
                resolve([]);
                return;
            }
            // Load all active models from database
            db.all(`SELECT internal_id, openrouter_id, name, provider_name, description,
                  context_window, pricing_input, pricing_output, is_active
           FROM openrouter_models
           WHERE is_active = 1
           ORDER BY provider_name, name`, [], (err2, rows) => {
                if (err2) {
                    reject(err2);
                }
                else {
                    // Map to format expected by frontend
                    const models = rows.map((row) => ({
                        value: row.openrouter_id,
                        label: row.name,
                        provider: row.provider_name,
                        description: row.description,
                        contextWindow: row.context_window,
                        pricingInput: row.pricing_input,
                        pricingOutput: row.pricing_output,
                        internalId: row.internal_id
                    }));
                    resolve(models);
                }
            });
        });
    });
}));
electron_1.ipcMain.handle('settings-reset', () => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve, reject) => {
        if (!db) {
            reject('Database not initialized');
            return;
        }
        // Clear configuration except essential items
        db.run('DELETE FROM configuration WHERE key NOT IN ("openrouter_api_key", "hive_license_key")', (err) => {
            if (err) {
                reject(err);
            }
            else {
                // Reset profile to default
                // Set balanced-performer as the active profile in consensus_settings
                db.run(`INSERT INTO consensus_settings (key, value, updated_at) 
           VALUES (?, ?, ?)
           ON CONFLICT(key) DO UPDATE SET
           value = excluded.value,
           updated_at = excluded.updated_at`, ['active_profile_id', 'balanced-performer', new Date().toISOString()], () => {
                    resolve(true);
                });
            }
        });
    });
}));
// Analytics data handler - fetch real consensus metrics from database
// Save conversation to database
electron_1.ipcMain.handle('save-conversation', (_, data) => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve) => {
        var _a;
        if (!db) {
            SafeLogger_1.logger.error('Database not initialized for saving conversation');
            resolve(false);
            return;
        }
        SafeLogger_1.logger.info('📝 Saving conversation to database:', {
            id: data.conversationId,
            cost: data.totalCost,
            tokens: data.totalTokens
        });
        const userId = '3034c561-e193-4968-a575-f1b165d31a5b'; // sales@hivetechs.io
        const timestamp = new Date().toISOString();
        // Insert into conversations table
        db.run(`
      INSERT INTO conversations (
        id, user_id, title, total_cost, total_tokens_input, total_tokens_output, 
        created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    `, [
            data.conversationId,
            userId,
            ((_a = data.question) === null || _a === void 0 ? void 0 : _a.substring(0, 100)) || 'Consensus Query',
            data.totalCost,
            data.inputTokens,
            data.outputTokens,
            timestamp,
            timestamp
        ], (err1) => {
            if (err1) {
                SafeLogger_1.logger.error('Error saving conversation:', err1);
                resolve(false);
                return;
            }
            // Insert into knowledge_conversations
            db.run(`
        INSERT INTO knowledge_conversations (
          conversation_id, question, final_answer, source_of_truth, created_at
        ) VALUES (?, ?, ?, ?, ?)
      `, [
                data.conversationId,
                data.question,
                data.answer,
                data.answer,
                timestamp
            ], (err2) => {
                if (err2)
                    SafeLogger_1.logger.error('Error saving to knowledge_conversations:', err2);
            });
            // Insert into conversation_usage for tracking
            db.run(`
        INSERT INTO conversation_usage (
          user_id, conversation_id, timestamp
        ) VALUES (?, ?, ?)
      `, [userId, data.conversationId, timestamp], (err3) => {
                if (err3)
                    SafeLogger_1.logger.error('Error saving to conversation_usage:', err3);
            });
            // Track model usage for each stage using the active profile
            // Get the active profile to know which models were used
            db.get(`
        SELECT cp.generator_model, cp.refiner_model, cp.validator_model, cp.curator_model 
        FROM consensus_settings cs
        JOIN consensus_profiles cp ON cs.value = cp.id
        WHERE cs.key = 'active_profile_id'
      `, [], (errProfile, profile) => {
                if (!errProfile && profile) {
                    // Insert stage outputs for tracking
                    const stages = [
                        { name: 'Generator', model: profile.generator_model },
                        { name: 'Refiner', model: profile.refiner_model },
                        { name: 'Validator', model: profile.validator_model },
                        { name: 'Curator', model: profile.curator_model }
                    ];
                    stages.forEach(stage => {
                        // Estimate tokens and cost per stage (divide by 4)
                        const stageTokens = Math.floor((data.totalTokens || 0) / 4);
                        const stageCost = (data.totalCost || 0) / 4;
                        db.run(`
              INSERT INTO stage_outputs (
                conversation_id, stage_name, model, tokens_used, cost, created_at
              ) VALUES (?, ?, ?, ?, ?, ?)
            `, [data.conversationId, stage.name, stage.model, stageTokens, stageCost, timestamp]);
                    });
                }
            });
            // Insert into performance_metrics if duration provided
            if (data.duration) {
                db.run(`
          INSERT INTO performance_metrics (
            conversation_id, timestamp, total_duration, total_cost, created_at
          ) VALUES (?, ?, ?, ?, ?)
        `, [data.conversationId, timestamp, data.duration, data.totalCost || 0, timestamp], (err4) => {
                    if (err4)
                        SafeLogger_1.logger.error('Error saving performance metrics:', err4);
                });
            }
            // Insert into cost_analytics
            db.run(`
        INSERT INTO cost_analytics (
          conversation_id, total_cost, cost_per_token, model_costs, optimization_potential, created_at
        ) VALUES (?, ?, ?, ?, ?, ?)
      `, [
                data.conversationId,
                data.totalCost,
                data.totalCost / (data.totalTokens || 1),
                JSON.stringify({ [data.model || 'consensus']: data.totalCost }),
                0,
                timestamp
            ], (err5) => {
                if (err5)
                    SafeLogger_1.logger.error('Error saving cost analytics:', err5);
            });
            SafeLogger_1.logger.info(`✅ Saved conversation ${data.conversationId} to database`);
            resolve(true);
        });
    });
}));
// Get user's daily usage count
electron_1.ipcMain.handle('get-usage-count', () => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve) => {
        if (!db) {
            resolve({ used: 0, limit: 999999, remaining: 999999 });
            return;
        }
        const userId = '3034c561-e193-4968-a575-f1b165d31a5b';
        const todayStart = new Date();
        todayStart.setHours(0, 0, 0, 0);
        db.get(`
      SELECT COUNT(*) as count 
      FROM conversation_usage 
      WHERE user_id = ? 
      AND date(timestamp, 'localtime') = date('now', 'localtime')
    `, [userId], (err, row) => {
            if (err) {
                SafeLogger_1.logger.error('Error getting usage count:', err);
                resolve({ used: 0, limit: 999999, remaining: 999999 });
                return;
            }
            const used = (row === null || row === void 0 ? void 0 : row.count) || 0;
            const limit = 999999; // Unlimited for this user
            const remaining = limit - used;
            SafeLogger_1.logger.info(`Usage count for user ${userId}: ${used} / ${limit}`);
            resolve({ used, limit, remaining });
        });
    });
}));
electron_1.ipcMain.handle('get-analytics', () => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve) => {
        if (!db) {
            SafeLogger_1.logger.error('Database not initialized for analytics');
            resolve(null);
            return;
        }
        const analyticsData = {};
        // Calculate date ranges
        const now = new Date();
        const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).toISOString();
        const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString();
        const monthAgo = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000).toISOString();
        // Get user-specific data - using the logged-in user's ID
        // This comes from the D1 validation response stored earlier
        const userId = '3034c561-e193-4968-a575-f1b165d31a5b'; // sales@hivetechs.io user ID
        // Get TODAY's queries for this user from conversation_usage table
        // Use UTC dates consistently since timestamps are stored in UTC
        db.get(`
      SELECT COUNT(*) as count 
      FROM conversation_usage 
      WHERE date(timestamp, 'localtime') = date('now', 'localtime') 
      AND user_id = ?
    `, [userId], (err1, row1) => {
            if (err1) {
                SafeLogger_1.logger.error('Error getting conversation count:', err1);
                resolve(null);
                return;
            }
            SafeLogger_1.logger.info('Analytics - Today queries for user:', row1 === null || row1 === void 0 ? void 0 : row1.count);
            analyticsData.todayQueries = (row1 === null || row1 === void 0 ? void 0 : row1.count) || 0;
            // Get all-time total queries for this user
            db.get(`
        SELECT COUNT(*) as count 
        FROM conversation_usage 
        WHERE user_id = ?
      `, [userId], (errTotal, rowTotal) => {
                SafeLogger_1.logger.info('Analytics - Total queries for user:', rowTotal === null || rowTotal === void 0 ? void 0 : rowTotal.count);
                analyticsData.totalQueries = (rowTotal === null || rowTotal === void 0 ? void 0 : rowTotal.count) || 0;
                // Get TODAY's cost and token usage - join with conversation_usage for user filtering
                // Convert UTC timestamps to localtime for date comparison
                db.get(`
          SELECT 
            SUM(c.total_cost) as total_cost,
            SUM(c.total_tokens_input) as total_input,
            SUM(c.total_tokens_output) as total_output,
            AVG(pm.total_duration / 1000.0) as avg_time
          FROM conversations c
          INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
          LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
          WHERE date(cu.timestamp, 'localtime') = date('now', 'localtime')
          AND cu.user_id = ?
        `, [userId], (err2, row2) => {
                    if (err2)
                        SafeLogger_1.logger.error('Error getting today cost data:', err2);
                    SafeLogger_1.logger.info('Analytics - Today cost data:', row2);
                    analyticsData.todayCost = (row2 === null || row2 === void 0 ? void 0 : row2.total_cost) || 0;
                    analyticsData.todayAvgResponseTime = (row2 === null || row2 === void 0 ? void 0 : row2.avg_time) || 0;
                    analyticsData.todayTokenUsage = {
                        total: ((row2 === null || row2 === void 0 ? void 0 : row2.total_input) || 0) + ((row2 === null || row2 === void 0 ? void 0 : row2.total_output) || 0),
                        input: (row2 === null || row2 === void 0 ? void 0 : row2.total_input) || 0,
                        output: (row2 === null || row2 === void 0 ? void 0 : row2.total_output) || 0
                    };
                    // Get all-time totals - join with conversation_usage for user filtering
                    db.get(`
            SELECT 
              SUM(c.total_cost) as total_cost,
              SUM(c.total_tokens_input) as total_input,
              SUM(c.total_tokens_output) as total_output,
              AVG(pm.total_duration / 1000.0) as avg_time
            FROM conversations c
            INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
            LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
            WHERE cu.user_id = ?
          `, [userId], (errAllTime, rowAllTime) => {
                        if (errAllTime)
                            SafeLogger_1.logger.error('Error getting all-time cost data:', errAllTime);
                        SafeLogger_1.logger.info('Analytics - All-time cost data:', rowAllTime);
                        analyticsData.totalCost = (rowAllTime === null || rowAllTime === void 0 ? void 0 : rowAllTime.total_cost) || 0;
                        analyticsData.avgResponseTime = (rowAllTime === null || rowAllTime === void 0 ? void 0 : rowAllTime.avg_time) || 0;
                        analyticsData.tokenUsage = {
                            total: ((rowAllTime === null || rowAllTime === void 0 ? void 0 : rowAllTime.total_input) || 0) + ((rowAllTime === null || rowAllTime === void 0 ? void 0 : rowAllTime.total_output) || 0),
                            input: (rowAllTime === null || rowAllTime === void 0 ? void 0 : rowAllTime.total_input) || 0,
                            output: (rowAllTime === null || rowAllTime === void 0 ? void 0 : rowAllTime.total_output) || 0
                        };
                        // Get recent activity - join with conversation_usage for user filtering
                        db.all(`
          SELECT 
            c.id as conversation_id,
            kc.question,
            c.total_cost as cost,
            c.total_tokens_input,
            c.total_tokens_output,
            pm.total_duration as duration,
            cu.timestamp
          FROM conversation_usage cu
          INNER JOIN conversations c ON c.id = cu.conversation_id
          LEFT JOIN knowledge_conversations kc ON c.id = kc.conversation_id
          LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
          WHERE cu.user_id = ?
          ORDER BY cu.timestamp DESC 
          LIMIT 10
        `, [userId], (err3, rows3) => {
                            if (err3)
                                SafeLogger_1.logger.error('Error getting recent activity:', err3);
                            SafeLogger_1.logger.info('Recent activity rows:', rows3 === null || rows3 === void 0 ? void 0 : rows3.slice(0, 2)); // Log first 2 rows
                            analyticsData.recentActivity = (rows3 || []).map((row) => ({
                                timestamp: row.timestamp,
                                question: row.question || 'Query',
                                model: 'consensus-pipeline',
                                cost: row.cost || 0,
                                duration: (row.duration || 0) / 1000,
                                status: 'completed',
                                tokens: (row.total_tokens_input || 0) + (row.total_tokens_output || 0),
                                conversationId: row.conversation_id
                            }));
                            // Get model usage from stage_outputs table (tracks all 4 models per conversation)
                            db.all(`
            SELECT 
              so.model,
              COUNT(*) as count,
              SUM(so.cost) as totalCost
            FROM stage_outputs so
            INNER JOIN conversation_usage cu ON so.conversation_id = cu.conversation_id
            WHERE cu.user_id = ?
            GROUP BY so.model
            ORDER BY totalCost DESC
          `, [userId], (err4, rows4) => {
                                if (err4) {
                                    SafeLogger_1.logger.error('Error getting model usage from stage_outputs:', err4);
                                    // Fallback: Get models from consensus_profiles if stage_outputs is empty
                                    db.all(`
                SELECT 
                  cp.generator_model as model,
                  COUNT(c.id) as count,
                  SUM(c.total_cost * 0.25) as totalCost
                FROM conversations c
                INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
                LEFT JOIN consensus_profiles cp ON c.profile_id = cp.id
                WHERE cu.user_id = ? AND cp.generator_model IS NOT NULL
                GROUP BY cp.generator_model
                UNION ALL
                SELECT 
                  cp.refiner_model as model,
                  COUNT(c.id) as count,
                  SUM(c.total_cost * 0.25) as totalCost
                FROM conversations c
                INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
                LEFT JOIN consensus_profiles cp ON c.profile_id = cp.id
                WHERE cu.user_id = ? AND cp.refiner_model IS NOT NULL
                GROUP BY cp.refiner_model
                UNION ALL
                SELECT 
                  cp.validator_model as model,
                  COUNT(c.id) as count,
                  SUM(c.total_cost * 0.25) as totalCost
                FROM conversations c
                INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
                LEFT JOIN consensus_profiles cp ON c.profile_id = cp.id
                WHERE cu.user_id = ? AND cp.validator_model IS NOT NULL
                GROUP BY cp.validator_model
                UNION ALL
                SELECT 
                  cp.curator_model as model,
                  COUNT(c.id) as count,
                  SUM(c.total_cost * 0.25) as totalCost
                FROM conversations c
                INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
                LEFT JOIN consensus_profiles cp ON c.profile_id = cp.id
                WHERE cu.user_id = ? AND cp.curator_model IS NOT NULL
                GROUP BY cp.curator_model
              `, [userId, userId, userId, userId], (err5, rows5) => {
                                        const modelUsage = {};
                                        const modelCosts = {};
                                        // Aggregate the model data
                                        (rows5 || []).forEach((row) => {
                                            var _a;
                                            const modelName = ((_a = row.model) === null || _a === void 0 ? void 0 : _a.split('/').pop()) || row.model; // Simplify model names
                                            if (!modelUsage[modelName]) {
                                                modelUsage[modelName] = 0;
                                                modelCosts[modelName] = 0;
                                            }
                                            modelUsage[modelName] += row.count || 0;
                                            modelCosts[modelName] += row.totalCost || 0;
                                        });
                                        analyticsData.modelUsage = modelUsage;
                                        analyticsData.costByModel = modelCosts;
                                        continueProcessing();
                                    });
                                    return;
                                }
                                const modelUsage = {};
                                const modelCosts = {};
                                // Process the results from stage_outputs
                                (rows4 || []).forEach((row) => {
                                    var _a;
                                    const modelName = ((_a = row.model) === null || _a === void 0 ? void 0 : _a.split('/').pop()) || row.model; // Simplify model names
                                    if (row.count > 0) {
                                        modelUsage[modelName] = row.count;
                                        modelCosts[modelName] = row.totalCost || 0;
                                    }
                                });
                                analyticsData.modelUsage = modelUsage;
                                analyticsData.costByModel = modelCosts;
                                continueProcessing();
                            });
                            function continueProcessing() {
                                // Calculate hourly stats for last 24 hours
                                const hourlyStats = [];
                                const now = new Date();
                                const processHour = (i) => {
                                    if (i < 0) {
                                        // All hours processed
                                        analyticsData.hourlyStats = hourlyStats;
                                        analyticsData.successRate = analyticsData.totalQueries > 0 ? 100 : 0;
                                        // Add alerts
                                        analyticsData.alerts = [{
                                                type: 'info',
                                                message: `Database contains ${analyticsData.totalQueries} consensus queries`,
                                                timestamp: new Date().toISOString()
                                            }];
                                        // Resolve with complete data
                                        SafeLogger_1.logger.info('Analytics - Complete data:', JSON.stringify(analyticsData, null, 2));
                                        resolve(analyticsData);
                                        return;
                                    }
                                    const hourStart = new Date(now.getTime() - (i + 1) * 60 * 60 * 1000);
                                    const hourEnd = new Date(now.getTime() - i * 60 * 60 * 1000);
                                    db.get(`
                SELECT 
                  COUNT(DISTINCT cu.conversation_id) as queries,
                  SUM(c.total_cost) as cost,
                  AVG(pm.total_duration / 1000.0) as avg_time
                FROM conversation_usage cu
                LEFT JOIN conversations c ON c.id = cu.conversation_id
                LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
                WHERE cu.timestamp >= ? AND cu.timestamp < ?
                AND cu.user_id = ?
              `, [hourStart.toISOString(), hourEnd.toISOString(), userId], (err5, row5) => {
                                        if (err5) {
                                            SafeLogger_1.logger.error('Error getting hourly stats:', err5);
                                            // Continue with default values even if error
                                            hourlyStats.push({
                                                hour: hourStart.getHours().toString().padStart(2, '0') + ':00',
                                                queries: 0,
                                                cost: 0,
                                                avgTime: 0
                                            });
                                            processHour(i - 1);
                                            return;
                                        }
                                        hourlyStats.push({
                                            hour: hourStart.getHours().toString().padStart(2, '0') + ':00',
                                            queries: (row5 === null || row5 === void 0 ? void 0 : row5.queries) || 0,
                                            cost: (row5 === null || row5 === void 0 ? void 0 : row5.cost) || 0,
                                            avgTime: (row5 === null || row5 === void 0 ? void 0 : row5.avg_time) || 0
                                        });
                                        // Process next hour
                                        processHour(i - 1);
                                    });
                                };
                                // Start processing hours from 23 to 0
                                processHour(23);
                            }
                        });
                    });
                });
            });
        });
    });
    // Memory Service is now managed entirely by ProcessManager
    // Use processManager.startProcess('memory-service') and processManager.stopProcess('memory-service')
    // Memory service cleanup is now handled in the unified performCleanup function
    // Store reference to main window
    electron_1.app.on('browser-window-created', (_, window) => {
        if (!mainWindow) {
            mainWindow = window;
        }
    });
}));
//# sourceMappingURL=index.js.map