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
  runQuickConsensus: (data: {query: string, profile?: string}) => {
    console.log('[Preload] runQuickConsensus called with:', data);
    return ipcRenderer.invoke('backend-consensus-quick', data);
  },
  async healthCheck(): Promise<any> {
    return ipcRenderer.invoke('backend-health');
  },
  async getBackendPort(): Promise<number> {
    return ipcRenderer.invoke('websocket-backend-port');
  },
  async testClaudeDebug(message: string): Promise<any> {
    return ipcRenderer.invoke('test-claude-debug', message);
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

// Consensus Events API - SIMPLIFIED
contextBridge.exposeInMainWorld('consensusAPI', {
  // Single visual update handler
  onVisualUpdate(callback: (data: any) => void) {
    ipcRenderer.on('visual-update', (_, data) => callback(data));
  },
  // Keep consensus complete for final response
  onConsensusComplete(callback: (data: any) => void) {
    ipcRenderer.on('consensus-complete', (_, data) => callback(data));
  },
  // New iterative consensus events
  onRoundUpdate(callback: (data: any) => void) {
    ipcRenderer.on('consensus-round-update', (_, data) => callback(data));
  },
  onStageUpdate(callback: (data: any) => void) {
    ipcRenderer.on('consensus-stage-update', (_, data) => callback(data));
  },
  onConsensusVoteUpdate(callback: (data: any) => void) {
    ipcRenderer.on('consensus-vote-update', (_, data) => callback(data));
  },
  onConsensusStatus(callback: (data: any) => void) {
    ipcRenderer.on('consensus-status', (_, data) => callback(data));
  },
  // Generic listener for any channel (for our direct commands)
  on(channel: string, callback: (data: any) => void) {
    ipcRenderer.on(channel, (_, data) => callback(data));
  },
  removeAllListeners() {
    ipcRenderer.removeAllListeners('visual-update');
    ipcRenderer.removeAllListeners('consensus-complete');
    ipcRenderer.removeAllListeners('consensus-round-update');
    ipcRenderer.removeAllListeners('consensus-stage-update');
    ipcRenderer.removeAllListeners('consensus-vote-update');
    ipcRenderer.removeAllListeners('consensus-status');
    ipcRenderer.removeAllListeners('updateStageStatus');
    ipcRenderer.removeAllListeners('updateStageProgress');
    ipcRenderer.removeAllListeners('updateModelDisplay');
    ipcRenderer.removeAllListeners('neuralConsciousness.updatePhase');
  },
  
  // Interrupt consensus functionality
  interruptConsensus: () => ipcRenderer.invoke('interrupt-consensus')
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
  getProfile: (profileName: string) => ipcRenderer.invoke('settings-get-profile', profileName),
  updateProfileMaxRounds: (profileName: string, maxRounds: number) => ipcRenderer.invoke('settings-update-profile-max-rounds', profileName, maxRounds),
});

// Analytics API
contextBridge.exposeInMainWorld('electronAPI', {
  // Proven analytics IPC handler backed by the unified DB (period-aware)
  getAnalytics: (period?: '24h' | '7d' | '30d') => ipcRenderer.invoke('get-analytics', period),
  saveConversation: (data: any) => ipcRenderer.invoke('save-conversation', data),
  getUsageCount: () => ipcRenderer.invoke('get-usage-count'),
  showInputDialog: (title: string, defaultValue?: string) => ipcRenderer.invoke('show-input-dialog', title, defaultValue),
  
  // Dialog API
  showOpenDialog: (options: any) => ipcRenderer.invoke('show-open-dialog', options),
  showSaveDialog: (options: any) => ipcRenderer.invoke('show-save-dialog', options),
  showMessageBox: (options: any) => ipcRenderer.invoke('show-message-box', options),
  setTitle: (title: string) => ipcRenderer.invoke('set-title', title),
  openExternal: (url: string) => ipcRenderer.invoke('open-external', url),
  
  // Memory Service API
  startMemoryService: () => ipcRenderer.invoke('memory-service-start'),
  stopMemoryService: () => ipcRenderer.invoke('memory-service-stop'),
  isMemoryServiceRunning: () => ipcRenderer.invoke('memory-service-status'),
  getMemoryServicePort: () => ipcRenderer.invoke('memory-service-port'),
  getMemoryStats: () => ipcRenderer.invoke('memory-service-stats'),
  getConnectedTools: () => ipcRenderer.invoke('memory-service-tools'),
  getMemoryActivity: (limit?: number) => ipcRenderer.invoke('memory-service-activity', limit),
  
  // CLI Tool Management
  detectCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-detect', toolId),
  detectAllCliTools: () => ipcRenderer.invoke('cli-tools-detect-all'),
  installCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-install', toolId),
  updateCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-update', toolId),
  uninstallCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-uninstall', toolId),
  configureCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-configure', toolId),
  launchCliTool: (toolId: string, projectPath?: string) => ipcRenderer.invoke('cli-tool-launch', toolId, projectPath),
  checkCliToolUpdates: () => ipcRenderer.invoke('cli-tools-check-updates'),
  
  // CLI Tool Events
  onCliToolInstallProgress: (callback: (progress: any) => void) => {
    ipcRenderer.on('cli-tool-install-progress', (_, progress) => callback(progress));
  },
  onCliToolUpdateProgress: (callback: (progress: any) => void) => {
    ipcRenderer.on('cli-tool-update-progress', (_, progress) => callback(progress));
  },
  onLaunchAIToolTerminal: (callback: (data: any) => void) => {
    ipcRenderer.on('launch-ai-tool-terminal', (_, data) => callback(data));
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
  onMenuCloseAllTabs: (callback: () => void) => {
    ipcRenderer.on('menu-close-all-tabs', callback);
  },
  onMenuToggleAutoSave: (callback: (enabled: boolean) => void) => {
    ipcRenderer.on('menu-toggle-auto-save', (_, enabled) => callback(enabled));
  },
  onMenuResetState: (callback: () => void) => {
    ipcRenderer.on('menu-reset-state', callback);
  },
  onMenuGettingStarted: (callback: () => void) => {
    ipcRenderer.on('menu-getting-started', callback);
  },
  onMenuMemoryGuide: (callback: () => void) => {
    ipcRenderer.on('menu-memory-guide', callback);
  },
  onMenuCloneRepo: (callback: () => void) => {
    ipcRenderer.on('menu-clone-repo', callback);
  },
  onMenuInitRepo: (callback: () => void) => {
    ipcRenderer.on('menu-init-repo', callback);
  },
  onMenuHelpDocumentation: (callback: () => void) => {
    ipcRenderer.on('menu-help-documentation', callback);
  },
  onMenuAbout: (callback: () => void) => {
    ipcRenderer.on('menu-about', callback);
  },
  onMenuShowWelcome: (callback: () => void) => {
    ipcRenderer.on('menu-show-welcome', callback);
  },
  onMenuToggleExplorer: (callback: () => void) => {
    ipcRenderer.on('menu-toggle-explorer', callback);
  },
  onMenuToggleGit: (callback: () => void) => {
    ipcRenderer.on('menu-toggle-git', callback);
  },
  onMenuToggleTerminal: (callback: () => void) => {
    ipcRenderer.on('menu-toggle-terminal', callback);
  },
  onMenuOpenMemory: (callback: () => void) => {
    ipcRenderer.on('menu-open-memory', callback);
  },
  onMenuOpenCliTools: (callback: () => void) => {
    ipcRenderer.on('menu-open-cli-tools', callback);
  },
  onMenuOpenAnalytics: (callback: () => void) => {
    ipcRenderer.on('menu-open-analytics', callback);
  },
  onMenuGoToFile: (callback: () => void) => {
    ipcRenderer.on('menu-go-to-file', callback);
  },
  onMenuGoToLine: (callback: () => void) => {
    ipcRenderer.on('menu-go-to-line', callback);
  },
  getVersion: () => ipcRenderer.invoke('get-app-version'),
  refreshMenu: () => ipcRenderer.invoke('menu-refresh'),
  updateMenuContext: (context: { autoSaveEnabled?: boolean; hasFolder?: boolean; isRepo?: boolean }) =>
    ipcRenderer.invoke('menu-update-context', context),
  // Backup helpers
  listBackups: () => ipcRenderer.invoke('list-backups'),
  deleteBackup: (filePath: string) => ipcRenderer.invoke('delete-backup', filePath),
  revealInFolder: (filePath: string) => ipcRenderer.invoke('reveal-in-folder', filePath),
  openPath: (targetPath: string) => ipcRenderer.invoke('open-path', targetPath)
});

// Git API
contextBridge.exposeInMainWorld('gitAPI', {
  getStatus: () => ipcRenderer.invoke('git-status'),
  getBranches: () => ipcRenderer.invoke('git-branches'),
  getLog: (options?: { maxCount?: number; skip?: number; graph?: boolean; oneline?: boolean; limit?: number }) => ipcRenderer.invoke('git-log', options),
  getDiff: (file?: string) => ipcRenderer.invoke('git-diff', file),
  getStagedDiff: (file?: string) => ipcRenderer.invoke('git-staged-diff', file),
  stage: (files: string[]) => ipcRenderer.invoke('git-stage', files),
  unstage: (files: string[]) => ipcRenderer.invoke('git-unstage', files),
  commit: (message: string) => ipcRenderer.invoke('git-commit', message),
  discard: (files: string[]) => ipcRenderer.invoke('git-discard', files),
  clean: (files: string[]) => ipcRenderer.invoke('git-clean', files),
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
  getSubmoduleDiff: (submodulePath: string) => ipcRenderer.invoke('git-submodule-diff', submodulePath),
  pushChunked: () => ipcRenderer.invoke('git-push-chunked'),
  getRepoStats: () => ipcRenderer.invoke('git-repo-stats'),
  // New push options
  pushWithOptions: (options: any) => ipcRenderer.invoke('git-push-with-options', options),
  pushForceWithLease: () => ipcRenderer.invoke('git-push-force-lease'),
  pushCustom: (command: string) => ipcRenderer.invoke('git-push-custom', command),
  pushDryRun: (options?: any) => ipcRenderer.invoke('git-push-dry-run', options)
  ,
  // Clone repository into a parent directory
  clone: (url: string, parentDirectory: string) => ipcRenderer.invoke('git-clone', url, parentDirectory)
});

// Maintenance API
contextBridge.exposeInMainWorld('maintenanceAPI', {
  modelsSyncNow: () => ipcRenderer.invoke('models-sync-now'),
  profilesMigrateV2: () => ipcRenderer.invoke('profiles-migrate-v2'),
  usageSyncNow: () => ipcRenderer.invoke('usage-sync-now'),
  profilesRebindActive: (profileIdOrName: string) => ipcRenderer.invoke('profiles-rebind-active', profileIdOrName)
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

// Database API
contextBridge.exposeInMainWorld('databaseAPI', {
  getSetting: (key: string) => ipcRenderer.invoke('db-get-setting', key),
  setSetting: (key: string, value: string) => ipcRenderer.invoke('db-set-setting', key, value),
  
  // Session persistence
  saveSession: (folderPath: string, tabs: any[], activeTab: string | null) => 
    ipcRenderer.invoke('db-save-session', folderPath, tabs, activeTab),
  loadSession: (folderPath: string) => 
    ipcRenderer.invoke('db-load-session', folderPath),
  clearSession: (folderPath: string) => 
    ipcRenderer.invoke('db-clear-session', folderPath),
  
  // Recent folders
  addRecentFolder: (folderPath: string, tabCount: number) => 
    ipcRenderer.invoke('db-add-recent-folder', folderPath, tabCount),
  getRecentFolders: () => 
    ipcRenderer.invoke('db-get-recent-folders'),
  removeRecentFolder: (folderPath: string) => 
    ipcRenderer.invoke('db-remove-recent-folder', folderPath)
  ,
  clearRecentFolders: () => 
    ipcRenderer.invoke('db-clear-recent-folders'),
  // Welcome analytics and DB maintenance
  logWelcomeAction: (action: string) => ipcRenderer.invoke('db-welcome-analytics-log', action),
  compact: () => ipcRenderer.invoke('db-compact'),
  integrityCheck: () => ipcRenderer.invoke('db-integrity-check')
  ,
  backup: (destPath: string) => ipcRenderer.invoke('db-backup', destPath),
  restore: (srcPath: string) => ipcRenderer.invoke('db-restore', srcPath)
});

// Terminal API
// Compatibility API for components expecting window.api
contextBridge.exposeInMainWorld('api', {
  invoke: (channel: string, ...args: any[]) => ipcRenderer.invoke(channel, ...args),
  receive: (channel: string, func: (...args: any[]) => void) => {
    ipcRenderer.on(channel, (event, ...args) => func(...args));
  },
  removeListener: (channel: string, func: (...args: any[]) => void) => {
    ipcRenderer.removeListener(channel, func);
  }
});

// SUPER SIMPLE TEST
contextBridge.exposeInMainWorld('testAPI', {
  test: () => {
    console.log('[Preload] testAPI.test called');
    return 'TEST WORKS';
  },
  testAsync: async () => {
    console.log('[Preload] testAPI.testAsync called');
    try {
      const result = await ipcRenderer.invoke('backend-consensus-quick', {
        query: 'Direct test from preload',
        profile: 'Free Also'
      });
      console.log('[Preload] Got result:', result);
      return result;
    } catch (error) {
      console.log('[Preload] Error:', error);
      throw error;
    }
  },
  
  // Interrupt consensus functionality
  interruptConsensus: () => ipcRenderer.invoke('interrupt-consensus')
});

contextBridge.exposeInMainWorld('terminalAPI', {
  // Create a new terminal process
  createTerminalProcess: (options: {
    terminalId: string;
    command?: string;
    args?: string[];
    cwd?: string;
    env?: Record<string, string>;
    toolId?: string;
  }) => safeInvoke('create-terminal-process', options),
  
  // Write data to terminal
  writeToTerminal: (terminalId: string, data: string) => 
    safeInvoke('write-to-terminal', terminalId, data),
  
  // Resize terminal
  resizeTerminal: (terminalId: string, cols: number, rows: number) =>
    safeInvoke('resize-terminal', terminalId, cols, rows),
  
  // Kill terminal process
  killTerminalProcess: (terminalId: string) =>
    safeInvoke('kill-terminal-process', terminalId),
  
  // Get terminal status
  getTerminalStatus: (terminalId: string) =>
    safeInvoke('get-terminal-status', terminalId),
  
  // List all terminals
  listTerminals: () => safeInvoke('list-terminals'),
  
  // Listen for terminal output
  onTerminalData: (callback: (terminalId: string, data: string) => void) => {
    ipcRenderer.on('terminal-data', (_, terminalId, data) => callback(terminalId, data));
  },
  
  // Listen for terminal exit
  onTerminalExit: (callback: (terminalId: string, code?: number) => void) => {
    ipcRenderer.on('terminal-exit', (_, terminalId, code) => callback(terminalId, code));
  },
  
  // Listen for terminal created event
  onTerminalCreated: (callback: (terminalInfo: any) => void) => {
    ipcRenderer.on('terminal-created', (_, terminalInfo) => callback(terminalInfo));
  },
  
  // Listen for terminal ready event
  onTerminalReady: (callback: (terminalId: string, url: string) => void) => {
    ipcRenderer.on('terminal-ready', (_, terminalId, url) => callback(terminalId, url));
  },
  
  // Listen for terminal error event
  onTerminalError: (callback: (terminalId: string, error: string) => void) => {
    ipcRenderer.on('terminal-error', (_, terminalId, error) => callback(terminalId, error));
  }
});
