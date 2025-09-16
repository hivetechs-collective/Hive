import { GitStatus, GitBranch, GitCommit } from './git';

export interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  size?: number;
  modified?: Date;
}

declare global {
  interface Window {
    // File API
    fileAPI: {
      getFileTree(rootPath?: string): Promise<FileNode[]>;
      getDirectoryContents(dirPath: string): Promise<FileNode[]>;
      readFile(filePath: string): Promise<string>;
      writeFile(filePath: string, content: string): Promise<void>;
      watchFile(filePath: string): Promise<void>;
      unwatchFile(filePath: string): Promise<void>;
      searchFiles(rootPath: string, pattern: string): Promise<string[]>;
      getFileStats(filePath: string): Promise<any>;
      createFile(dirPath: string, fileName: string): Promise<boolean>;
      createFolder(dirPath: string, folderName: string): Promise<boolean>;
      moveFile(sourcePath: string, targetPath: string): Promise<boolean>;
      fileExists(filePath: string): Promise<boolean>;
      onFileChanged(callback: (filePath: string) => void): void;
    };
    
    // Git API (extend existing)
    gitAPI: {
      getStatus(): Promise<GitStatus>;
      getBranches(): Promise<GitBranch[]>;
      getLog(options?: { maxCount?: number; skip?: number; graph?: boolean; oneline?: boolean; limit?: number }): Promise<string | GitCommit[]>;
      getDiff(file?: string): Promise<string>;
      getStagedDiff(file?: string): Promise<string>;
      stage(files: string[]): Promise<void>;
      unstage(files: string[]): Promise<void>;
      commit(message: string): Promise<void>;
      discard(files: string[]): Promise<void>;
      clean(files: string[]): Promise<void>;
      push(): Promise<void>;
      pull(): Promise<void>;
      fetch(): Promise<void>;
      switchBranch(branchName: string): Promise<void>;
      createBranch(branchName: string): Promise<void>;
      getFileStatus(path: string): Promise<string | null>;
      initRepo(repoPath: string): Promise<any>;
      getCommitFiles(hash: string): Promise<any>;
      getFileDiff(commitHash: string, filePath: string): Promise<string>;
      setFolder(folderPath: string): Promise<any>;
      sync(): Promise<void>;
      getSubmoduleStatus(submodulePath: string): Promise<string>;
      getSubmoduleDiff(submodulePath: string): Promise<string>;
      analyzeRepository?(): Promise<any>;
      openMergeTool?(): Promise<void>;
      pushChunked(): Promise<string>;
      getRepoStats(): Promise<{
        totalSize: string;
        largestPack: string;
        commitCount: number;
        recommendation: string;
      }>;
      clone(url: string, parentDirectory: string): Promise<{ success: boolean; destination?: string; output?: string; error?: string }>;
    };
    
    // Terminal API
    terminalAPI: {
      createTerminalProcess(options: {
        terminalId: string;
        command?: string;
        args?: string[];
        cwd?: string;
        env?: Record<string, string>;
        toolId?: string;
      }): Promise<any>;
      writeToTerminal(terminalId: string, data: string): Promise<any>;
      resizeTerminal(terminalId: string, cols: number, rows: number): Promise<any>;
      killTerminalProcess(terminalId: string): Promise<any>;
      getTerminalStatus(terminalId: string): Promise<any>;
      listTerminals(): Promise<any[]>;
      onTerminalData(callback: (terminalId: string, data: string) => void): void;
      onTerminalExit(callback: (terminalId: string, code?: number) => void): void;
      onTerminalCreated(callback: (terminalInfo: any) => void): void;
      onTerminalReady(callback: (terminalId: string, url: string) => void): void;
      onTerminalError(callback: (terminalId: string, error: string) => void): void;
    };
    
    // Database API
    databaseAPI: {
      getSetting(key: string): Promise<string | null>;
      setSetting(key: string, value: string): Promise<{ success: boolean }>;
      // Session persistence
      saveSession?(folderPath: string, tabs: any[], activeTab: string | null): Promise<any>;
      loadSession?(folderPath: string): Promise<any>;
      clearSession?(folderPath: string): Promise<any>;
      // Recent folders
      addRecentFolder?(folderPath: string, tabCount: number): Promise<any>;
      getRecentFolders?(): Promise<Array<{ folder_path: string; last_opened: string; tab_count: number }>>;
      removeRecentFolder?(folderPath: string): Promise<any>;
    };
    
    // Electron API
    electronAPI: {
      getAnalytics(): Promise<any>;
      saveConversation(data: any): Promise<any>;
      getUsageCount(): Promise<any>;
      showInputDialog(title: string, defaultValue?: string): Promise<string | null>;
      showOpenDialog(options: any): Promise<any>;
      showSaveDialog(options: any): Promise<any>;
      showMessageBox(options: any): Promise<any>;
      setTitle(title: string): Promise<void>;
      onMenuOpenFolder(callback: (folderPath: string) => void): void;
      onMenuCloseFolder(callback: () => void): void;
      onMenuOpenFile(callback: (filePath: string) => void): void;
      onMenuNewFile(callback: () => void): void;
      onMenuSave(callback: () => void): void;
      onMenuSaveAs(callback: () => void): void;
      onMenuCloseTab(callback: () => void): void;
      onMenuCloseAllTabs(callback: () => void): void;
      onMenuToggleAutoSave(callback: (enabled: boolean) => void): void;
      onMenuResetState(callback: () => void): void;
      onMenuGettingStarted(callback: () => void): void;
      onMenuMemoryGuide(callback: () => void): void;
      onMenuCloneRepo(callback: () => void): void;
      onMenuInitRepo(callback: () => void): void;
      onMenuAbout(callback: () => void): void;
      
      // Help menu handlers
      onMenuHelpDocumentation(callback: () => void): void;
      onMenuShowWelcome(callback: () => void): void;
      onMenuToggleExplorer(callback: () => void): void;
      onMenuToggleGit(callback: () => void): void;
      onMenuToggleTerminal(callback: () => void): void;
      onMenuOpenMemory(callback: () => void): void;
      onMenuOpenCliTools(callback: () => void): void;
      onMenuOpenAnalytics(callback: () => void): void;
      onMenuGoToFile(callback: () => void): void;
      onMenuGoToLine(callback: () => void): void;
      
      getVersion(): Promise<string>;
      openExternal?(url: string): Promise<void>;
      refreshMenu(): Promise<any>;
      updateMenuContext(context: { autoSaveEnabled?: boolean; hasFolder?: boolean; isRepo?: boolean }): Promise<any>;
      
      // Memory Service API
      startMemoryService(): Promise<boolean>;
      stopMemoryService(): Promise<void>;
      isMemoryServiceRunning(): Promise<boolean>;
      getMemoryStats(): Promise<any>;
      getConnectedTools(): Promise<any[]>;
      getMemoryActivity(limit?: number): Promise<any[]>;
      
      // Terminal Event Forwarding (for compatibility)
      createTerminalProcess?: (options: any) => Promise<any>;
      killTerminalProcess?: (terminalId: string) => Promise<any>;
      onTerminalCreated?: (callback: (terminalInfo: any) => void) => void;
      onTerminalReady?: (callback: (terminalId: string, url: string) => void) => void;
      onTerminalExit?: (callback: (terminalId: string, code?: number) => void) => void;
      
      // AI Tool Launch Event
      onLaunchAIToolTerminal?: (callback: (data: {
        toolId: string;
        toolName: string;
        command: string;
        cwd: string;
      }) => void) => void;
    };
    
    // Consensus API
    consensusAPI: {
      interruptConsensus: () => void;
    };
    
    // Global instances
    fileExplorer: any;
    editorTabs: any;
    gitUI: any;
    scmView: any;
    
    // Global variables
    currentOpenedFolder?: string;
    
    // Global functions
    openFolder: (folderPath?: string) => Promise<void>;
    closeFolder: () => void;
    cloneRepository: () => Promise<void>;
  }
}
