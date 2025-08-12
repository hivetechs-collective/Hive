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
      onFileChanged(callback: (filePath: string) => void): void;
    };
    
    // Git API (extend existing)
    gitAPI: {
      getStatus(): Promise<GitStatus>;
      getBranches(): Promise<GitBranch[]>;
      getLog(limit?: number): Promise<GitCommit[]>;
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
    };
    
    // Global instances
    fileExplorer: any;
    editorTabs: any;
    gitUI: any;
  }
}