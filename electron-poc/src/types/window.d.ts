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
      getLog(options?: { maxCount?: number; graph?: boolean; oneline?: boolean; limit?: number }): Promise<string | GitCommit[]>;
      getDiff(file?: string): Promise<string>;
      getStagedDiff(file?: string): Promise<string>;
      stage(files: string[]): Promise<void>;
      unstage(files: string[]): Promise<void>;
      commit(message: string): Promise<void>;
      discard(files: string[]): Promise<void>;
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
      onMenuNewFile(callback: () => void): void;
      onMenuSaveFile(callback: () => void): void;
      
      // Memory Service API
      startMemoryService(): Promise<boolean>;
      stopMemoryService(): Promise<void>;
      isMemoryServiceRunning(): Promise<boolean>;
      getMemoryStats(): Promise<any>;
      getConnectedTools(): Promise<any[]>;
      getMemoryActivity(limit?: number): Promise<any[]>;
    };
    
    // Global instances
    fileExplorer: any;
    editorTabs: any;
    gitUI: any;
    scmView: any;
    
    // Global functions
    openFolder: () => Promise<void>;
    cloneRepository: () => Promise<void>;
  }
}