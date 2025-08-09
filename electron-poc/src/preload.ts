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
