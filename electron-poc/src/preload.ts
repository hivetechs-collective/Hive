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
  async healthCheck(): Promise<any> {
    return ipcRenderer.invoke('backend-health');
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
