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
// Set the app name immediately
electron_1.app.setName('Hive Consensus');
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const sqlite3_1 = require("sqlite3");
const git_manager_1 = require("./git-manager");
const git_manager_v2_1 = require("./git-manager-v2");
const EnhancedGitManager_1 = require("./git/EnhancedGitManager");
const file_system_1 = require("./file-system");
const ProcessManager_1 = require("./utils/ProcessManager");
const CliToolsManager_1 = require("./utils/CliToolsManager");
const cli_tool_detector_1 = require("./utils/cli-tool-detector");
// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
    electron_1.app.quit();
}
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
        console.log('[Main] No folder path provided, GitManager will return null status');
        // Don't create a manager when no folder is open
        gitManager = null;
        return;
    }
    if (GIT_MANAGER_VERSION === 2) {
        console.log('[Main] Using EnhancedGitManager with authentication support for:', folderPath);
        gitManager = new EnhancedGitManager_1.EnhancedGitManager(folderPath);
        yield gitManager.initialize();
    }
    else if (GIT_MANAGER_VERSION === 1) {
        console.log('[Main] Using GitManagerV2 with VS Code-style implementation');
        gitManager = new git_manager_v2_1.GitManagerV2(folderPath);
    }
    else {
        console.log('[Main] Using old GitManager with simple-git');
        gitManager = new git_manager_1.GitManager(folderPath);
    }
});
// File System Manager
let fileSystemManager = null;
// Initialize File System manager
const initFileSystemManager = () => {
    fileSystemManager = new file_system_1.FileSystemManager();
};
const createWindow = () => {
    // Create the browser window.
    mainWindow = new electron_1.BrowserWindow({
        height: 600,
        width: 800,
        minWidth: 700,
        minHeight: 400,
        title: 'Hive Consensus',
        icon: path.join(__dirname, '../resources/icon.png'),
        webPreferences: {
            preload: MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY,
            nodeIntegration: false,
            contextIsolation: true,
            webSecurity: false, // Allow HTTP requests to localhost for development
        },
    });
    // and load the index.html of the app.
    mainWindow.loadURL(MAIN_WINDOW_WEBPACK_ENTRY);
    // Open the DevTools.
    // mainWindow.webContents.openDevTools(); // Disabled to prevent warning overlay
    // Register IPC handlers here after window creation
    registerGitHandlers();
    registerFileSystemHandlers();
    // Memory service handlers will be registered later
    // Create application menu
    createApplicationMenu();
};
const registerGitHandlers = () => {
    // Git IPC handlers
    electron_1.ipcMain.handle('git-status', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            // No folder is open, return null to show welcome screen
            console.log('[Main] git-status: No folder open, returning null');
            return null;
        }
        return yield gitManager.getStatus();
    }));
    electron_1.ipcMain.handle('git-branches', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            console.log('[Main] git-branches: No folder open, returning empty');
            return { all: [], branches: {}, current: null, detached: false };
        }
        return yield gitManager.getBranches();
    }));
    electron_1.ipcMain.handle('git-log', (_, options) => __awaiter(void 0, void 0, void 0, function* () {
        if (!gitManager) {
            console.log('[Main] git-log: No folder open, returning empty');
            return '';
        }
        console.log('[Main] git-log called with options:', options);
        const result = yield gitManager.getLog(options || {});
        console.log('[Main] git-log returning:', result ? result.substring(0, 100) + '...' : 'empty');
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
    electron_1.ipcMain.handle('git-push', () => __awaiter(void 0, void 0, void 0, function* () {
        console.log('[Main] git-push IPC called');
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            if (gitManager instanceof EnhancedGitManager_1.EnhancedGitManager) {
                const result = yield gitManager.push();
                console.log('[Main] git-push result:', result);
                if (!result.success) {
                    throw new Error(result.error || 'Push failed');
                }
                return result.output;
            }
            else {
                const result = yield gitManager.push();
                console.log('[Main] git-push completed successfully');
                return result;
            }
        }
        catch (error) {
            console.error('[Main] git-push failed:', error);
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
        console.log('[Main] git-sync IPC called');
        if (!gitManager) {
            throw new Error('No folder open');
        }
        try {
            if (gitManager instanceof EnhancedGitManager_1.EnhancedGitManager) {
                const result = yield gitManager.sync();
                console.log('[Main] git-sync result:', result);
                if (!result.success) {
                    throw new Error(result.error || 'Sync failed');
                }
                return result.output;
            }
            else if (gitManager instanceof git_manager_v2_1.GitManagerV2) {
                const result = yield gitManager.sync();
                console.log('[Main] git-sync completed successfully');
                return result;
            }
            else {
                // Fallback for old GitManager - do pull then push
                yield gitManager.pull();
                yield gitManager.push();
                console.log('[Main] git-sync (pull+push) completed successfully');
                return;
            }
        }
        catch (error) {
            console.error('[Main] git-sync failed:', error);
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
        console.log('[Git] Setting folder to:', folderPath || '(none)');
        // If empty string or null, clear the git manager
        if (!folderPath) {
            gitManager = null;
            console.log('[Git] Cleared Git manager - no folder open');
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
            console.error('[Git] Failed to get submodule status:', error);
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
            console.error('[Git] Failed to get submodule diff:', error);
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
            console.log('[Main] fs-get-tree called without root path, returning empty');
            return [];
        }
        console.log('[Main] fs-get-tree called with root:', rootPath);
        const result = yield fileSystemManager.getFileTree(rootPath);
        console.log('[Main] fs-get-tree returning', (result === null || result === void 0 ? void 0 : result.length) || 0, 'items');
        return result;
    }));
    electron_1.ipcMain.handle('fs-get-directory', (_, dirPath) => __awaiter(void 0, void 0, void 0, function* () {
        if (!fileSystemManager)
            initFileSystemManager();
        console.log('[Main] fs-get-directory called for:', dirPath);
        const result = yield fileSystemManager.getDirectoryContents(dirPath);
        console.log('[Main] fs-get-directory returning', (result === null || result === void 0 ? void 0 : result.length) || 0, 'items for', dirPath);
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
            console.log('[Main] Creating file:', filePath);
            yield fs.writeFile(filePath, '', 'utf8');
            console.log('[Main] File created successfully:', filePath);
            return true;
        }
        catch (error) {
            console.error('[Main] Failed to create file:', error);
            throw error;
        }
    }));
    electron_1.ipcMain.handle('fs-create-folder', (_, dirPath, folderName) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const fs = require('fs').promises;
            const path = require('path');
            const folderPath = path.join(dirPath, folderName);
            console.log('[Main] Creating folder:', folderPath);
            yield fs.mkdir(folderPath, { recursive: true });
            console.log('[Main] Folder created successfully:', folderPath);
            return true;
        }
        catch (error) {
            console.error('[Main] Failed to create folder:', error);
            throw error;
        }
    }));
    electron_1.ipcMain.handle('fs-move-file', (_, sourcePath, targetPath) => __awaiter(void 0, void 0, void 0, function* () {
        try {
            const fs = require('fs').promises;
            console.log('[Main] Moving:', sourcePath, 'to', targetPath);
            yield fs.rename(sourcePath, targetPath);
            console.log('[Main] Move successful');
            return true;
        }
        catch (error) {
            console.error('[Main] Failed to move file:', error);
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
                    label: 'Close Tab',
                    accelerator: 'CmdOrCtrl+W',
                    click: () => {
                        if (mainWindow) {
                            mainWindow.webContents.send('menu-close-tab');
                        }
                    }
                },
                { type: 'separator' },
                {
                    label: 'Quit',
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
                            console.log('[Menu] Reloading with state reset...');
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
// Memory Service management - uses ProcessManager for production-ready management
const processManager = new ProcessManager_1.ProcessManager();
let memoryServicePort = 3457;
let websocketBackendPort = 8765; // Dynamic port for WebSocket backend
// CLI Tools Manager for AI CLI integration
let cliToolsManager = null;
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
    // Register WebSocket Consensus Backend with full port flexibility
    const consensusBackendPath = path.join('/Users/veronelazio/Developer/Private/hive', 'target', 'debug', 'hive-backend-server-enhanced');
    console.log('[ProcessManager] Registering WebSocket backend at:', consensusBackendPath);
    processManager.registerProcess({
        name: 'websocket-backend',
        scriptPath: consensusBackendPath,
        args: [],
        env: {
            PORT: '8765',
            RUST_LOG: 'info',
            NODE_ENV: 'development'
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
                console.log('[Main] Memory Service ready on port:', msg.port);
                memoryServicePort = msg.port || memoryServicePort;
            }
            else if (msg.type === 'db-query') {
                handleMemoryServiceDbQuery(msg);
            }
        }
        else if (name === 'websocket-backend') {
            if (msg.type === 'ready') {
                console.log('[Main] WebSocket backend ready on port:', msg.port);
                // Store the actual port for WebSocket connections
                websocketBackendPort = msg.port;
            }
        }
    });
    // Listen for process status changes
    processManager.on('process:crashed', (name) => {
        console.error(`[Main] Process ${name} crashed`);
        mainWindow === null || mainWindow === void 0 ? void 0 : mainWindow.webContents.send('process-status', { name, status: 'crashed' });
    });
    processManager.on('process:started', (name) => {
        console.log(`[Main] Process ${name} started`);
        mainWindow === null || mainWindow === void 0 ? void 0 : mainWindow.webContents.send('process-status', { name, status: 'running' });
    });
    processManager.on('process:unhealthy', (name, error) => {
        console.error(`[Main] Process ${name} health check failed:`, error.message);
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
        console.error('[Main] Memory Service process not available for IPC');
    }
};
// Handle database queries from Memory Service
const handleMemoryServiceDbQuery = (msg) => {
    console.log('[Main] Received db-query from Memory Service:', msg.sql);
    if (!db) {
        console.error('[Main] Database not initialized');
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
        console.log('[Main] Database query result:', error ? `Error: ${error.message}` : `${(rows === null || rows === void 0 ? void 0 : rows.length) || 0} rows`);
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
    console.log('[Main] Registering Memory Service IPC handlers');
    electron_1.ipcMain.handle('memory-service-start', () => __awaiter(void 0, void 0, void 0, function* () {
        console.log('[Main] IPC: memory-service-start');
        const status = processManager.getProcessStatus('memory-service');
        if (status && status.status === 'running') {
            console.log('[Main] Memory Service already running');
            return true;
        }
        try {
            console.log('[Main] Starting Memory Service as child process...');
            // Use ts-node to run TypeScript directly from source directory
            const scriptPath = path.join(electron_1.app.getAppPath(), 'src', 'memory-service', 'index.ts');
            console.log('[Main] Memory Service script path:', scriptPath);
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
            console.error('[Main] Failed to start Memory Service:', error);
            return false;
        }
    }));
    electron_1.ipcMain.handle('memory-service-stop', () => __awaiter(void 0, void 0, void 0, function* () {
        console.log('[Main] IPC: memory-service-stop');
        return yield processManager.stopProcess('memory-service');
    }));
    electron_1.ipcMain.handle('memory-service-status', () => __awaiter(void 0, void 0, void 0, function* () {
        const status = processManager.getProcessStatus('memory-service');
        const isRunning = (status === null || status === void 0 ? void 0 : status.status) === 'running';
        console.log('[Main] IPC: memory-service-status, result:', isRunning);
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
            console.error('[Main] Failed to get memory stats:', error);
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
            console.error('[Main] Failed to get connected tools:', error);
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
            console.error('[Main] Failed to get activity stream:', error);
        }
        return [];
    }));
};
// ========== CLI TOOLS MANAGEMENT ==========
// Initialize CLI Tools Manager
const initializeCliToolsManager = () => {
    if (!db) {
        console.error('[Main] Database not initialized for CLI Tools Manager');
        return;
    }
    cliToolsManager = new CliToolsManager_1.CliToolsManager(db);
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
    console.log('[Main] CLI Tools Manager initialized');
};
// Register CLI Tools IPC handlers
const registerCliToolsHandlers = () => {
    // Get all tool statuses
    electron_1.ipcMain.handle('cli-tools-get-all-status', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return {};
        const statuses = yield cliToolsManager.getAllStatuses();
        return Object.fromEntries(statuses);
    }));
    // Get specific tool status
    electron_1.ipcMain.handle('cli-tools-check-installed', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return false;
        return yield cliToolsManager.checkInstalled(toolId);
    }));
    // Install a tool
    electron_1.ipcMain.handle('cli-tools-install', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            throw new Error('CLI Tools Manager not initialized');
        yield cliToolsManager.install(toolId);
        return { success: true };
    }));
    // Uninstall a tool
    electron_1.ipcMain.handle('cli-tools-uninstall', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            throw new Error('CLI Tools Manager not initialized');
        yield cliToolsManager.uninstall(toolId);
        return { success: true };
    }));
    // Update a tool
    electron_1.ipcMain.handle('cli-tools-update', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            throw new Error('CLI Tools Manager not initialized');
        yield cliToolsManager.update(toolId);
        return { success: true };
    }));
    // Check for updates for a single tool
    electron_1.ipcMain.handle('cli-tools-check-update', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return false;
        return yield cliToolsManager.checkForUpdates(toolId);
    }));
    // Check for updates for all tools
    electron_1.ipcMain.handle('cli-tools-check-all-updates', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return {};
        const updates = yield cliToolsManager.checkAllUpdates();
        return Object.fromEntries(updates);
    }));
    // Configure a tool (e.g., auth)
    electron_1.ipcMain.handle('cli-tools-configure', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            throw new Error('CLI Tools Manager not initialized');
        yield cliToolsManager.configureTool(toolId);
        return { success: true };
    }));
    // Cancel installation
    electron_1.ipcMain.handle('cli-tools-cancel-install', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return;
        cliToolsManager.cancelInstallation(toolId);
        return { success: true };
    }));
    // Get installation logs
    electron_1.ipcMain.handle('cli-tools-get-logs', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return [];
        return cliToolsManager.getInstallationLogs(toolId);
    }));
    // Update settings
    electron_1.ipcMain.handle('cli-tools-update-settings', (_, settings) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return;
        cliToolsManager.updateSettings(settings);
        return { success: true };
    }));
    // Get tool configuration
    electron_1.ipcMain.handle('cli-tools-get-config', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return null;
        return cliToolsManager.getTool(toolId);
    }));
    // Get all tools
    electron_1.ipcMain.handle('cli-tools-get-all', () => __awaiter(void 0, void 0, void 0, function* () {
        if (!cliToolsManager)
            return {};
        const tools = cliToolsManager.getAllTools();
        return Object.fromEntries(tools);
    }));
    // Select directory (for custom install path)
    electron_1.ipcMain.handle('select-directory', () => __awaiter(void 0, void 0, void 0, function* () {
        const result = yield electron_1.dialog.showOpenDialog({
            properties: ['openDirectory', 'createDirectory']
        });
        return result.canceled ? null : result.filePaths[0];
    }));
    // Forward progress events to renderer
    if (cliToolsManager) {
        cliToolsManager.on('install-progress', (data) => {
            if (mainWindow) {
                mainWindow.webContents.send('cli-tool-progress', data);
            }
        });
        cliToolsManager.on('update-available', (data) => {
            if (mainWindow) {
                mainWindow.webContents.send('cli-tool-update-available', data);
            }
        });
    }
};
// Register simple CLI tool detection handlers (without the complex CliToolsManager)
const registerSimpleCliToolHandlers = () => {
    console.log('[Main] Registering simple CLI tool detection handlers');
    // CLI Tool Detection Handlers
    electron_1.ipcMain.handle('cli-tool-detect', (_, toolId) => __awaiter(void 0, void 0, void 0, function* () {
        console.log(`[Main] Detecting CLI tool: ${toolId}`);
        try {
            const status = yield (0, cli_tool_detector_1.getCachedToolStatus)(toolId);
            return status;
        }
        catch (error) {
            console.error(`[Main] Error detecting CLI tool ${toolId}:`, error);
            return null;
        }
    }));
    electron_1.ipcMain.handle('cli-tools-detect-all', () => __awaiter(void 0, void 0, void 0, function* () {
        console.log('[Main] Detecting all CLI tools...');
        try {
            const tools = yield (0, cli_tool_detector_1.detectAllCliTools)();
            return tools;
        }
        catch (error) {
            console.error('[Main] Error detecting CLI tools:', error);
            return [];
        }
    }));
};
// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
electron_1.app.on('ready', () => __awaiter(void 0, void 0, void 0, function* () {
    initDatabase();
    // Initialize ProcessManager with all process configurations
    initializeProcessManager();
    // Register Memory Service handlers BEFORE creating window
    // This ensures they're available when the renderer process starts
    registerMemoryServiceHandlers();
    // Start all processes in PARALLEL for fast startup (2025 best practice)
    console.log('[Main] 🚀 Starting all managed processes in parallel...');
    // Log initial ProcessManager status
    console.log('[Main] Initial ProcessManager status:');
    processManager.logStatus();
    // Start all services simultaneously - no blocking, no waiting
    const startupPromises = [];
    // Start WebSocket Backend (highest priority - contains AI Helpers and Consensus)
    startupPromises.push(processManager.startProcess('websocket-backend')
        .then((started) => {
        if (started) {
            const backendInfo = processManager.getProcessStatus('websocket-backend');
            if (backendInfo && backendInfo.port) {
                websocketBackendPort = backendInfo.port;
            }
            console.log(`[Main] ✅ WebSocket Backend started on port ${websocketBackendPort}`);
            return { name: 'websocket-backend', success: true };
        }
        else {
            console.error('[Main] ❌ WebSocket Backend failed to start');
            return { name: 'websocket-backend', success: false };
        }
    })
        .catch((error) => {
        console.error('[Main] ❌ WebSocket Backend error:', error.message);
        return { name: 'websocket-backend', success: false };
    }));
    // Start Memory Service in parallel (non-blocking, non-critical)
    startupPromises.push(processManager.startProcess('memory-service')
        .then((started) => {
        if (started) {
            console.log('[Main] ✅ Memory Service started successfully');
            return { name: 'memory-service', success: true };
        }
        else {
            console.warn('[Main] ⚠️ Memory Service failed (non-critical)');
            return { name: 'memory-service', success: false };
        }
    })
        .catch((error) => {
        console.warn('[Main] ⚠️ Memory Service error (non-critical):', error.message);
        return { name: 'memory-service', success: false };
    }));
    // Wait for all services to complete startup attempts
    const results = yield Promise.allSettled(startupPromises);
    // Check critical services only (backend with AI Helpers is critical)
    let criticalServicesHealthy = true;
    results.forEach((result) => {
        if (result.status === 'fulfilled') {
            const { name, success } = result.value;
            if (name === 'websocket-backend' && !success) {
                criticalServicesHealthy = false;
            }
        }
    });
    // Log final status
    console.log('[Main] Final ProcessManager status after parallel startup:');
    processManager.logStatus();
    if (!criticalServicesHealthy) {
        console.error('[Main] ⚠️ Critical services (WebSocket Backend) failed to start');
    }
    else {
        console.log('[Main] ✅ Critical services started successfully');
    }
    // Initialize CLI Tools Manager (commented out to avoid WebSocket issues)
    // initializeCliToolsManager();
    // Register our simple CLI tool detection handlers
    registerSimpleCliToolHandlers();
    // Don't initialize Git manager on startup - wait until a folder is opened
    // initGitManager(); 
    createWindow();
    // Register dialog handlers (Git and FileSystem handlers are already registered in createWindow)
    registerDialogHandlers();
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
// Clean up processes on app quit
electron_1.app.on('before-quit', (event) => __awaiter(void 0, void 0, void 0, function* () {
    event.preventDefault();
    console.log('[Main] Cleaning up processes before quit...');
    yield processManager.cleanup();
    electron_1.app.exit(0);
}));
// Handle unexpected termination
process.on('SIGINT', () => __awaiter(void 0, void 0, void 0, function* () {
    console.log('[Main] Received SIGINT, cleaning up...');
    yield processManager.cleanup();
    process.exit(0);
}));
process.on('SIGTERM', () => __awaiter(void 0, void 0, void 0, function* () {
    console.log('[Main] Received SIGTERM, cleaning up...');
    yield processManager.cleanup();
    process.exit(0);
}));
// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.
// Set up IPC handlers for backend communication
electron_1.ipcMain.handle('backend-health', () => __awaiter(void 0, void 0, void 0, function* () {
    try {
        const response = yield fetch('http://localhost:8765/health');
        return yield response.json();
    }
    catch (error) {
        throw error;
    }
}));
electron_1.ipcMain.handle('backend-test', () => __awaiter(void 0, void 0, void 0, function* () {
    try {
        const response = yield fetch('http://localhost:8765/test', {
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
        const response = yield fetch('http://localhost:8765/api/consensus', {
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
        const response = yield fetch('http://127.0.0.1:8765/api/consensus/quick', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        return yield response.json();
    }
    catch (error) {
        console.error('Quick consensus error:', error);
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
                console.log(`[Main] Using dynamic WebSocket port: ${backendPort}`);
            }
            wsConnection = new WebSocket(url);
            wsConnection.on('open', () => {
                console.log('WebSocket connected in main process');
                resolve({ connected: true });
            });
            wsConnection.on('message', (data) => {
                // Forward message to renderer
                event.sender.send('websocket-message', data.toString());
            });
            wsConnection.on('error', (error) => {
                console.error('WebSocket error in main:', error);
                event.sender.send('websocket-error', error.message);
                reject(error);
            });
            wsConnection.on('close', () => {
                console.log('WebSocket closed in main process');
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
    console.log('WebSocket not ready, attempting reconnection...');
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
                    console.log('WebSocket reconnected successfully');
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
        console.error('Failed to reconnect WebSocket:', error);
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
    var _a, _b, _c, _d;
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
            console.error('Failed to test OpenRouter key:', error);
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
                        console.log('D1 validation response:', JSON.stringify(data, null, 2));
                        if (data.valid) {
                            // Parse tier information
                            const tier = data.tier || ((_a = data.user) === null || _a === void 0 ? void 0 : _a.subscription_tier) || 'free';
                            const dailyLimit = data.daily_limit || ((_b = data.limits) === null || _b === void 0 ? void 0 : _b.daily) || 10;
                            const email = data.email || ((_c = data.user) === null || _c === void 0 ? void 0 : _c.email) || '';
                            const userId = data.user_id || ((_d = data.user) === null || _d === void 0 ? void 0 : _d.id) || '';
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
                        console.error('License validation failed:', response.status, errorText);
                        result.hiveValid = false;
                        result.licenseInfo = {
                            valid: false,
                            error: `Validation failed: ${response.status}`
                        };
                    }
                }
                catch (error) {
                    console.error('Failed to validate Hive license:', error);
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
                        console.error('Failed to save active profile:', err);
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
            console.error('Database not initialized for saving conversation');
            resolve(false);
            return;
        }
        console.log('📝 Saving conversation to database:', {
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
                console.error('Error saving conversation:', err1);
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
                    console.error('Error saving to knowledge_conversations:', err2);
            });
            // Insert into conversation_usage for tracking
            db.run(`
        INSERT INTO conversation_usage (
          user_id, conversation_id, timestamp
        ) VALUES (?, ?, ?)
      `, [userId, data.conversationId, timestamp], (err3) => {
                if (err3)
                    console.error('Error saving to conversation_usage:', err3);
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
                        console.error('Error saving performance metrics:', err4);
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
                    console.error('Error saving cost analytics:', err5);
            });
            console.log(`✅ Saved conversation ${data.conversationId} to database`);
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
                console.error('Error getting usage count:', err);
                resolve({ used: 0, limit: 999999, remaining: 999999 });
                return;
            }
            const used = (row === null || row === void 0 ? void 0 : row.count) || 0;
            const limit = 999999; // Unlimited for this user
            const remaining = limit - used;
            console.log(`Usage count for user ${userId}: ${used} / ${limit}`);
            resolve({ used, limit, remaining });
        });
    });
}));
electron_1.ipcMain.handle('get-analytics', () => __awaiter(void 0, void 0, void 0, function* () {
    return new Promise((resolve) => {
        if (!db) {
            console.error('Database not initialized for analytics');
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
                console.error('Error getting conversation count:', err1);
                resolve(null);
                return;
            }
            console.log('Analytics - Today queries for user:', row1 === null || row1 === void 0 ? void 0 : row1.count);
            analyticsData.todayQueries = (row1 === null || row1 === void 0 ? void 0 : row1.count) || 0;
            // Get all-time total queries for this user
            db.get(`
        SELECT COUNT(*) as count 
        FROM conversation_usage 
        WHERE user_id = ?
      `, [userId], (errTotal, rowTotal) => {
                console.log('Analytics - Total queries for user:', rowTotal === null || rowTotal === void 0 ? void 0 : rowTotal.count);
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
                        console.error('Error getting today cost data:', err2);
                    console.log('Analytics - Today cost data:', row2);
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
                            console.error('Error getting all-time cost data:', errAllTime);
                        console.log('Analytics - All-time cost data:', rowAllTime);
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
                                console.error('Error getting recent activity:', err3);
                            console.log('Recent activity rows:', rows3 === null || rows3 === void 0 ? void 0 : rows3.slice(0, 2)); // Log first 2 rows
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
                                    console.error('Error getting model usage from stage_outputs:', err4);
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
                                        console.log('Analytics - Complete data:', JSON.stringify(analyticsData, null, 2));
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
                                            console.error('Error getting hourly stats:', err5);
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
    // Start Memory Service using ProcessManager
    const startMemoryService = () => __awaiter(void 0, void 0, void 0, function* () {
        const status = processManager.getProcessStatus('memory-service');
        if (status && status.status === 'running') {
            console.log('[Main] Memory Service already running');
            return true;
        }
        try {
            console.log('[Main] Starting Memory Service with ProcessManager...');
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
            console.error('[Main] Failed to start Memory Service:', error);
            return false;
        }
    });
    // Stop Memory Service using ProcessManager
    const stopMemoryService = () => __awaiter(void 0, void 0, void 0, function* () {
        yield processManager.stopProcess('memory-service');
    });
    // Clean up on app quit
    electron_1.app.on('before-quit', () => __awaiter(void 0, void 0, void 0, function* () {
        yield stopMemoryService();
    }));
    // Store reference to main window
    electron_1.app.on('browser-window-created', (_, window) => {
        if (!mainWindow) {
            mainWindow = window;
        }
    });
}));
//# sourceMappingURL=index.js.map