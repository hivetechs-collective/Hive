"use strict";
// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
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
// Expose backend API to renderer
electron_1.contextBridge.exposeInMainWorld('backendAPI', {
    testConnection() {
        return __awaiter(this, void 0, void 0, function* () {
            return electron_1.ipcRenderer.invoke('backend-test');
        });
    },
    runConsensus(query) {
        return __awaiter(this, void 0, void 0, function* () {
            return electron_1.ipcRenderer.invoke('backend-consensus', query);
        });
    },
    runQuickConsensus(data) {
        return __awaiter(this, void 0, void 0, function* () {
            return electron_1.ipcRenderer.invoke('backend-consensus-quick', data);
        });
    },
    healthCheck() {
        return __awaiter(this, void 0, void 0, function* () {
            return electron_1.ipcRenderer.invoke('backend-health');
        });
    }
});
// WebSocket proxy API
electron_1.contextBridge.exposeInMainWorld('websocketAPI', {
    connect(url) {
        return __awaiter(this, void 0, void 0, function* () {
            return electron_1.ipcRenderer.invoke('websocket-connect', url);
        });
    },
    send(message) {
        return __awaiter(this, void 0, void 0, function* () {
            return electron_1.ipcRenderer.invoke('websocket-send', message);
        });
    },
    close() {
        return __awaiter(this, void 0, void 0, function* () {
            return electron_1.ipcRenderer.invoke('websocket-close');
        });
    },
    onMessage(callback) {
        electron_1.ipcRenderer.on('websocket-message', (_, data) => callback(data));
    },
    onError(callback) {
        electron_1.ipcRenderer.on('websocket-error', (_, error) => callback(error));
    },
    onClose(callback) {
        electron_1.ipcRenderer.on('websocket-closed', callback);
    }
});
// Settings API
electron_1.contextBridge.exposeInMainWorld('settingsAPI', {
    loadSettings: () => electron_1.ipcRenderer.invoke('settings-load'),
    testKeys: (keys) => electron_1.ipcRenderer.invoke('settings-test-keys', keys),
    saveKeys: (keys) => electron_1.ipcRenderer.invoke('settings-save-keys', keys),
    saveProfile: (profile) => electron_1.ipcRenderer.invoke('settings-save-profile', profile),
    saveAllSettings: (settings) => electron_1.ipcRenderer.invoke('settings-save-all', settings),
    resetSettings: () => electron_1.ipcRenderer.invoke('settings-reset'),
    loadProfiles: () => electron_1.ipcRenderer.invoke('settings-load-profiles'),
    loadModels: () => electron_1.ipcRenderer.invoke('settings-load-models'),
});
// Analytics API
electron_1.contextBridge.exposeInMainWorld('electronAPI', {
    getAnalytics: () => electron_1.ipcRenderer.invoke('get-analytics'),
    saveConversation: (data) => electron_1.ipcRenderer.invoke('save-conversation', data),
    getUsageCount: () => electron_1.ipcRenderer.invoke('get-usage-count'),
    showInputDialog: (title, defaultValue) => electron_1.ipcRenderer.invoke('show-input-dialog', title, defaultValue),
    // Dialog API
    showOpenDialog: (options) => electron_1.ipcRenderer.invoke('show-open-dialog', options),
    showSaveDialog: (options) => electron_1.ipcRenderer.invoke('show-save-dialog', options),
    showMessageBox: (options) => electron_1.ipcRenderer.invoke('show-message-box', options),
    setTitle: (title) => electron_1.ipcRenderer.invoke('set-title', title),
    // Memory Service API
    startMemoryService: () => electron_1.ipcRenderer.invoke('memory-service-start'),
    stopMemoryService: () => electron_1.ipcRenderer.invoke('memory-service-stop'),
    isMemoryServiceRunning: () => electron_1.ipcRenderer.invoke('memory-service-status'),
    getMemoryStats: () => electron_1.ipcRenderer.invoke('memory-service-stats'),
    getConnectedTools: () => electron_1.ipcRenderer.invoke('memory-service-tools'),
    getMemoryActivity: (limit) => electron_1.ipcRenderer.invoke('memory-service-activity', limit),
    // CLI Tool Detection
    detectCliTool: (toolId) => electron_1.ipcRenderer.invoke('cli-tool-detect', toolId),
    detectAllCliTools: () => electron_1.ipcRenderer.invoke('cli-tools-detect-all'),
    // Menu event listeners
    onMenuOpenFolder: (callback) => {
        electron_1.ipcRenderer.on('menu-open-folder', (_, folderPath) => callback(folderPath));
    },
    onMenuCloseFolder: (callback) => {
        electron_1.ipcRenderer.on('menu-close-folder', callback);
    },
    onMenuOpenFile: (callback) => {
        electron_1.ipcRenderer.on('menu-open-file', (_, filePath) => callback(filePath));
    },
    onMenuNewFile: (callback) => {
        electron_1.ipcRenderer.on('menu-new-file', callback);
    },
    onMenuSave: (callback) => {
        electron_1.ipcRenderer.on('menu-save', callback);
    },
    onMenuSaveAs: (callback) => {
        electron_1.ipcRenderer.on('menu-save-as', callback);
    },
    onMenuCloseTab: (callback) => {
        electron_1.ipcRenderer.on('menu-close-tab', callback);
    },
    onMenuResetState: (callback) => {
        electron_1.ipcRenderer.on('menu-reset-state', callback);
    }
});
// Git API
electron_1.contextBridge.exposeInMainWorld('gitAPI', {
    getStatus: () => electron_1.ipcRenderer.invoke('git-status'),
    getBranches: () => electron_1.ipcRenderer.invoke('git-branches'),
    getLog: (options) => electron_1.ipcRenderer.invoke('git-log', options),
    getDiff: (file) => electron_1.ipcRenderer.invoke('git-diff', file),
    getStagedDiff: (file) => electron_1.ipcRenderer.invoke('git-staged-diff', file),
    stage: (files) => electron_1.ipcRenderer.invoke('git-stage', files),
    unstage: (files) => electron_1.ipcRenderer.invoke('git-unstage', files),
    commit: (message) => electron_1.ipcRenderer.invoke('git-commit', message),
    discard: (files) => electron_1.ipcRenderer.invoke('git-discard', files),
    push: () => electron_1.ipcRenderer.invoke('git-push'),
    pull: () => electron_1.ipcRenderer.invoke('git-pull'),
    sync: () => electron_1.ipcRenderer.invoke('git-sync'),
    fetch: () => electron_1.ipcRenderer.invoke('git-fetch'),
    switchBranch: (branchName) => electron_1.ipcRenderer.invoke('git-switch-branch', branchName),
    createBranch: (branchName) => electron_1.ipcRenderer.invoke('git-create-branch', branchName),
    getFileStatus: (path) => electron_1.ipcRenderer.invoke('git-file-status', path),
    initRepo: (repoPath) => electron_1.ipcRenderer.invoke('git-init', repoPath),
    getCommitFiles: (hash) => electron_1.ipcRenderer.invoke('git-commit-files', hash),
    getFileDiff: (commitHash, filePath) => electron_1.ipcRenderer.invoke('git-file-diff', commitHash, filePath),
    setFolder: (folderPath) => electron_1.ipcRenderer.invoke('git-set-folder', folderPath),
    getSubmoduleStatus: (submodulePath) => electron_1.ipcRenderer.invoke('git-submodule-status', submodulePath),
    getSubmoduleDiff: (submodulePath) => electron_1.ipcRenderer.invoke('git-submodule-diff', submodulePath)
});
// Helper to safely invoke IPC calls and prevent Event objects from being thrown
const safeInvoke = (channel, ...args) => __awaiter(void 0, void 0, void 0, function* () {
    try {
        const result = yield electron_1.ipcRenderer.invoke(channel, ...args);
        return result;
    }
    catch (error) {
        // If error is an Event object, convert it to a proper error
        if (error instanceof Event) {
            console.error('[SafeInvoke] Caught Event object as error, converting...');
            throw new Error('IPC call failed: Event object thrown');
        }
        // If error looks like [object Event], convert it
        if (error && typeof error === 'object' && error.toString && error.toString().includes('[object Event]')) {
            console.error('[SafeInvoke] Caught [object Event] string, converting...');
            throw new Error('IPC call failed: Event-like object thrown');
        }
        throw error;
    }
});
// File System API
electron_1.contextBridge.exposeInMainWorld('fileAPI', {
    getFileTree: (rootPath) => safeInvoke('fs-get-tree', rootPath),
    getDirectoryContents: (dirPath) => safeInvoke('fs-get-directory', dirPath),
    readFile: (filePath) => safeInvoke('fs-read-file', filePath),
    writeFile: (filePath, content) => safeInvoke('fs-write-file', filePath, content),
    watchFile: (filePath) => safeInvoke('fs-watch-file', filePath),
    unwatchFile: (filePath) => safeInvoke('fs-unwatch-file', filePath),
    searchFiles: (rootPath, pattern) => safeInvoke('fs-search', rootPath, pattern),
    getFileStats: (filePath) => safeInvoke('fs-stats', filePath),
    createFile: (dirPath, fileName) => safeInvoke('fs-create-file', dirPath, fileName),
    createFolder: (dirPath, folderName) => safeInvoke('fs-create-folder', dirPath, folderName),
    moveFile: (sourcePath, targetPath) => safeInvoke('fs-move-file', sourcePath, targetPath),
    fileExists: (filePath) => safeInvoke('fs-file-exists', filePath),
    onFileChanged: (callback) => {
        electron_1.ipcRenderer.on('file-changed', (_, filePath) => callback(filePath));
    }
});
//# sourceMappingURL=preload.js.map