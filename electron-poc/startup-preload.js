const { contextBridge, ipcRenderer } = require('electron');

// Expose protected methods for startup screen
contextBridge.exposeInMainWorld('electronAPI', {
    // Listen for startup progress updates
    onStartupProgress: (callback) => {
        ipcRenderer.on('startup-progress', callback);
    },
    
    // Listen for startup errors
    onStartupError: (callback) => {
        ipcRenderer.on('startup-error', callback);
    },
    
    // Clean up listeners
    removeAllListeners: () => {
        ipcRenderer.removeAllListeners('startup-progress');
        ipcRenderer.removeAllListeners('startup-error');
    }
});