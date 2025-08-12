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
  getUsageCount: () => ipcRenderer.invoke('get-usage-count')
});

// Git API
contextBridge.exposeInMainWorld('gitAPI', {
  getStatus: () => ipcRenderer.invoke('git-status'),
  getBranches: () => ipcRenderer.invoke('git-branches'),
  getLog: (limit?: number) => ipcRenderer.invoke('git-log', limit),
  getDiff: (file?: string) => ipcRenderer.invoke('git-diff', file),
  getStagedDiff: (file?: string) => ipcRenderer.invoke('git-staged-diff', file),
  stage: (files: string[]) => ipcRenderer.invoke('git-stage', files),
  unstage: (files: string[]) => ipcRenderer.invoke('git-unstage', files),
  commit: (message: string) => ipcRenderer.invoke('git-commit', message),
  discard: (files: string[]) => ipcRenderer.invoke('git-discard', files),
  push: () => ipcRenderer.invoke('git-push'),
  pull: () => ipcRenderer.invoke('git-pull'),
  fetch: () => ipcRenderer.invoke('git-fetch'),
  switchBranch: (branchName: string) => ipcRenderer.invoke('git-switch-branch', branchName),
  createBranch: (branchName: string) => ipcRenderer.invoke('git-create-branch', branchName),
  getFileStatus: (path: string) => ipcRenderer.invoke('git-file-status', path)
});

// File System API
contextBridge.exposeInMainWorld('fileAPI', {
  getFileTree: (rootPath?: string) => ipcRenderer.invoke('fs-get-tree', rootPath),
  getDirectoryContents: (dirPath: string) => ipcRenderer.invoke('fs-get-directory', dirPath),
  readFile: (filePath: string) => ipcRenderer.invoke('fs-read-file', filePath),
  writeFile: (filePath: string, content: string) => ipcRenderer.invoke('fs-write-file', filePath, content),
  watchFile: (filePath: string) => ipcRenderer.invoke('fs-watch-file', filePath),
  unwatchFile: (filePath: string) => ipcRenderer.invoke('fs-unwatch-file', filePath),
  searchFiles: (rootPath: string, pattern: string) => ipcRenderer.invoke('fs-search', rootPath, pattern),
  getFileStats: (filePath: string) => ipcRenderer.invoke('fs-stats', filePath),
  onFileChanged: (callback: (filePath: string) => void) => {
    ipcRenderer.on('file-changed', (_, filePath) => callback(filePath));
  }
});
