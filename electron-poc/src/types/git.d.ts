export interface GitFileStatus {
  path: string;
  index: string;  // X in XY format - staged status
  working: string; // Y in XY format - working tree status
  working_dir?: string; // Alternative property used by simple-git library
  renamed?: string;
}

export interface GitBranch {
  name: string;
  current: boolean;
  commit: string;
  remote?: string;
  ahead?: number;
  behind?: number;
}

export interface GitCommit {
  hash: string;
  author: string;
  date: Date;
  message: string;
  refs?: string;
}

export interface GitStatus {
  files: GitFileStatus[];
  branch: string;
  ahead: number;
  behind: number;
  isRepo: boolean;
  repoPath?: string;
  hasUpstream?: boolean;
}

declare global {
  interface Window {
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
      sync(): Promise<void>;
      fetch(): Promise<void>;
      switchBranch(branchName: string): Promise<void>;
      createBranch(branchName: string): Promise<void>;
      initRepo(repoPath: string): Promise<any>;
      getCommitFiles(hash: string): Promise<{ files: any[] }>;
      getFileDiff(commitHash: string, filePath: string): Promise<string>;
      setFolder(folderPath: string): Promise<{ success: boolean }>;
    };
  }
}