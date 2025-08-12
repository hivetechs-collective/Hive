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
  getUsageCount: () => ipcRenderer.invoke('get-usage-count'),
  
  // LazyGit API
  startLazyGit: () => ipcRenderer.invoke('lazygit-start'),
  stopLazyGit: () => ipcRenderer.invoke('lazygit-stop'),
  writeLazyGit: (data: string) => ipcRenderer.invoke('lazygit-write', data),
  resizeLazyGit: (cols: number, rows: number) => ipcRenderer.invoke('lazygit-resize', cols, rows),
  onLazyGitData: (callback: (data: string) => void) => {
    ipcRenderer.on('lazygit-data', (_, data) => callback(data));
  },
  onLazyGitExit: (callback: (code: number) => void) => {
    ipcRenderer.on('lazygit-exit', (_, code) => callback(code));
  }
});
