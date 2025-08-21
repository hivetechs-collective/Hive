// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import { contextBridge, ipcRenderer } from 'electron';

// Expose backend API to renderer
contextBridge.exposeInMainWorld('backendAPI', {
  async testConnection(): Promise<any> {
    return ipcRenderer.invoke('backend-test');
  },
  async runConsensus(query: string): Promise<any> {
    return ipcRenderer.invoke('backend-consensus', query);
  },
  async runQuickConsensus(data: {query: string, profile?: string}): Promise<any> {
    return ipcRenderer.invoke('backend-consensus-quick', data);
  },
  async healthCheck(): Promise<any> {
    return ipcRenderer.invoke('backend-health');
  },
  async getBackendPort(): Promise<number> {
    return ipcRenderer.invoke('websocket-backend-port');
  }
});

// WebSocket proxy API
contextBridge.exposeInMainWorld('websocketAPI', {
  async connect(url: string): Promise<any> {
    return ipcRenderer.invoke('websocket-connect', url);
  },
  async send(message: string): Promise<any> {
    return ipcRenderer.invoke('websocket-send', message);
  },
  async close(): Promise<any> {
    return ipcRenderer.invoke('websocket-close');
  },
  onMessage(callback: (data: string) => void) {
    ipcRenderer.on('websocket-message', (_, data) => callback(data));
  },
  onError(callback: (error: string) => void) {
    ipcRenderer.on('websocket-error', (_, error) => callback(error));
  },
  onClose(callback: () => void) {
    ipcRenderer.on('websocket-closed', callback);
  }
});

// Settings API
contextBridge.exposeInMainWorld('settingsAPI', {
  loadSettings: () => ipcRenderer.invoke('settings-load'),
  testKeys: (keys: any) => ipcRenderer.invoke('settings-test-keys', keys),
  saveKeys: (keys: any) => ipcRenderer.invoke('settings-save-keys', keys),
  saveProfile: (profile: any) => ipcRenderer.invoke('settings-save-profile', profile),
  saveAllSettings: (settings: any) => ipcRenderer.invoke('settings-save-all', settings),
  resetSettings: () => ipcRenderer.invoke('settings-reset'),
  loadProfiles: () => ipcRenderer.invoke('settings-load-profiles'),
  loadModels: () => ipcRenderer.invoke('settings-load-models'),
});

// Analytics API
contextBridge.exposeInMainWorld('electronAPI', {
  getAnalytics: () => ipcRenderer.invoke('get-analytics'),
  saveConversation: (data: any) => ipcRenderer.invoke('save-conversation', data),
  getUsageCount: () => ipcRenderer.invoke('get-usage-count'),
  showInputDialog: (title: string, defaultValue?: string) => ipcRenderer.invoke('show-input-dialog', title, defaultValue),
  
  // Dialog API
  showOpenDialog: (options: any) => ipcRenderer.invoke('show-open-dialog', options),
  showSaveDialog: (options: any) => ipcRenderer.invoke('show-save-dialog', options),
  showMessageBox: (options: any) => ipcRenderer.invoke('show-message-box', options),
  setTitle: (title: string) => ipcRenderer.invoke('set-title', title),
  
  // Memory Service API
  startMemoryService: () => ipcRenderer.invoke('memory-service-start'),
  stopMemoryService: () => ipcRenderer.invoke('memory-service-stop'),
  isMemoryServiceRunning: () => ipcRenderer.invoke('memory-service-status'),
  getMemoryStats: () => ipcRenderer.invoke('memory-service-stats'),
  getConnectedTools: () => ipcRenderer.invoke('memory-service-tools'),
  getMemoryActivity: (limit?: number) => ipcRenderer.invoke('memory-service-activity', limit),
  
  // CLI Tool Management
  detectCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-detect', toolId),
  detectAllCliTools: () => ipcRenderer.invoke('cli-tools-detect-all'),
  installCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-install', toolId),
  updateCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-update', toolId),
  configureCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-configure', toolId),
  launchCliTool: (toolId: string, projectPath: string) => ipcRenderer.invoke('cli-tool-launch', toolId, projectPath),
  checkCliToolUpdates: () => ipcRenderer.invoke('cli-tools-check-updates'),
  
  // CLI Tool Events
  onCliToolInstallProgress: (callback: (progress: any) => void) => {
    ipcRenderer.on('cli-tool-install-progress', (_, progress) => callback(progress));
  },
  onCliToolUpdateProgress: (callback: (progress: any) => void) => {
    ipcRenderer.on('cli-tool-update-progress', (_, progress) => callback(progress));
  },
  
  // Menu event listeners
  onMenuOpenFolder: (callback: (folderPath: string) => void) => {
    ipcRenderer.on('menu-open-folder', (_, folderPath) => callback(folderPath));
  },
  onMenuCloseFolder: (callback: () => void) => {
    ipcRenderer.on('menu-close-folder', callback);
  },
  onMenuOpenFile: (callback: (filePath: string) => void) => {
    ipcRenderer.on('menu-open-file', (_, filePath) => callback(filePath));
  },
  onMenuNewFile: (callback: () => void) => {
    ipcRenderer.on('menu-new-file', callback);
  },
  onMenuSave: (callback: () => void) => {
    ipcRenderer.on('menu-save', callback);
  },
  onMenuSaveAs: (callback: () => void) => {
    ipcRenderer.on('menu-save-as', callback);
  },
  onMenuCloseTab: (callback: () => void) => {
    ipcRenderer.on('menu-close-tab', callback);
  },
  onMenuResetState: (callback: () => void) => {
    ipcRenderer.on('menu-reset-state', callback);
  }
});

// Git API
contextBridge.exposeInMainWorld('gitAPI', {
  getStatus: () => ipcRenderer.invoke('git-status'),
  getBranches: () => ipcRenderer.invoke('git-branches'),
  getLog: (options?: { maxCount?: number; graph?: boolean; oneline?: boolean; limit?: number }) => ipcRenderer.invoke('git-log', options),
  getDiff: (file?: string) => ipcRenderer.invoke('git-diff', file),
  getStagedDiff: (file?: string) => ipcRenderer.invoke('git-staged-diff', file),
  stage: (files: string[]) => ipcRenderer.invoke('git-stage', files),
  unstage: (files: string[]) => ipcRenderer.invoke('git-unstage', files),
  commit: (message: string) => ipcRenderer.invoke('git-commit', message),
  discard: (files: string[]) => ipcRenderer.invoke('git-discard', files),
  push: () => ipcRenderer.invoke('git-push'),
  pull: () => ipcRenderer.invoke('git-pull'),
  sync: () => ipcRenderer.invoke('git-sync'),
  fetch: () => ipcRenderer.invoke('git-fetch'),
  switchBranch: (branchName: string) => ipcRenderer.invoke('git-switch-branch', branchName),
  createBranch: (branchName: string) => ipcRenderer.invoke('git-create-branch', branchName),
  getFileStatus: (path: string) => ipcRenderer.invoke('git-file-status', path),
  initRepo: (repoPath: string) => ipcRenderer.invoke('git-init', repoPath),
  getCommitFiles: (hash: string) => ipcRenderer.invoke('git-commit-files', hash),
  getFileDiff: (commitHash: string, filePath: string) => ipcRenderer.invoke('git-file-diff', commitHash, filePath),
  setFolder: (folderPath: string) => ipcRenderer.invoke('git-set-folder', folderPath),
  getSubmoduleStatus: (submodulePath: string) => ipcRenderer.invoke('git-submodule-status', submodulePath),
  getSubmoduleDiff: (submodulePath: string) => ipcRenderer.invoke('git-submodule-diff', submodulePath)
});

// Helper to safely invoke IPC calls and prevent Event objects from being thrown
const safeInvoke = async (channel: string, ...args: any[]) => {
  try {
    const result = await ipcRenderer.invoke(channel, ...args);
    return result;
  } catch (error) {
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
};

// File System API
contextBridge.exposeInMainWorld('fileAPI', {
  getFileTree: (rootPath?: string) => safeInvoke('fs-get-tree', rootPath),
  getDirectoryContents: (dirPath: string) => safeInvoke('fs-get-directory', dirPath),
  readFile: (filePath: string) => safeInvoke('fs-read-file', filePath),
  writeFile: (filePath: string, content: string) => safeInvoke('fs-write-file', filePath, content),
  watchFile: (filePath: string) => safeInvoke('fs-watch-file', filePath),
  unwatchFile: (filePath: string) => safeInvoke('fs-unwatch-file', filePath),
  searchFiles: (rootPath: string, pattern: string) => safeInvoke('fs-search', rootPath, pattern),
  getFileStats: (filePath: string) => safeInvoke('fs-stats', filePath),
  createFile: (dirPath: string, fileName: string) => safeInvoke('fs-create-file', dirPath, fileName),
  createFolder: (dirPath: string, folderName: string) => safeInvoke('fs-create-folder', dirPath, folderName),
  moveFile: (sourcePath: string, targetPath: string) => safeInvoke('fs-move-file', sourcePath, targetPath),
  fileExists: (filePath: string) => safeInvoke('fs-file-exists', filePath),
  onFileChanged: (callback: (filePath: string) => void) => {
    ipcRenderer.on('file-changed', (_, filePath) => callback(filePath));
  }
});
