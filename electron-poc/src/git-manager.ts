import simpleGit, { SimpleGit, StatusResult, LogResult, DiffResult } from 'simple-git';
import * as path from 'path';
import * as fs from 'fs';

export interface GitFileStatus {
  path: string;
  index: string;  // X in XY format - staged status
  working: string; // Y in XY format - working tree status
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

export class GitManager {
  private git: SimpleGit;
  private repoPath: string;
  private isRepo: boolean = false;

  constructor(repoPath?: string) {
    // Always use the parent hive directory for Git operations
    this.repoPath = repoPath || '/Users/veronelazio/Developer/Private/hive';
    this.git = simpleGit(this.repoPath);
    this.checkIfRepo();
  }

  private async checkIfRepo(): Promise<void> {
    try {
      const isRepo = await this.git.checkIsRepo();
      this.isRepo = isRepo;
    } catch (error) {
      this.isRepo = false;
    }
  }

  async getStatus(): Promise<{
    files: GitFileStatus[];
    branch: string;
    ahead: number;
    behind: number;
    isRepo: boolean;
  }> {
    if (!this.isRepo) {
      return {
        files: [],
        branch: '',
        ahead: 0,
        behind: 0,
        isRepo: false
      };
    }

    try {
      const status = await this.git.status();
      
      const files: GitFileStatus[] = [];
      
      // Process all file statuses
      status.files.forEach(file => {
        // Filter out submodules (dioxus-fork, src/hive_ui) - these are deprecated
        // Only include files from electron-poc directory
        if (file.path.startsWith('electron-poc/') || 
            (!file.path.includes('dioxus-fork') && !file.path.includes('src/hive_ui'))) {
          files.push({
            path: file.path,
            index: file.index || ' ',
            working: file.working_dir || ' ',
            renamed: (file as any).rename || undefined
          });
        }
      });

      return {
        files,
        branch: status.current || 'master',
        ahead: status.ahead,
        behind: status.behind,
        isRepo: true
      };
    } catch (error) {
      console.error('Git status error:', error);
      throw error;
    }
  }

  async getBranches(): Promise<GitBranch[]> {
    if (!this.isRepo) return [];

    try {
      const summary = await this.git.branchLocal();
      const branches: GitBranch[] = [];

      for (const [name, branch] of Object.entries(summary.branches)) {
        branches.push({
          name,
          current: branch.current,
          commit: branch.commit,
          remote: (branch as any).tracking || undefined
        });
      }

      return branches;
    } catch (error) {
      console.error('Git branches error:', error);
      return [];
    }
  }

  async getLog(limit: number = 50): Promise<GitCommit[]> {
    if (!this.isRepo) return [];

    try {
      const log = await this.git.log({ maxCount: limit });
      
      return log.all.map(commit => ({
        hash: commit.hash,
        author: commit.author_name,
        date: new Date(commit.date),
        message: commit.message,
        refs: commit.refs
      }));
    } catch (error) {
      console.error('Git log error:', error);
      return [];
    }
  }

  async getDiff(file?: string): Promise<string> {
    if (!this.isRepo) return '';

    try {
      let diff: string;
      if (file) {
        diff = await this.git.diff(['--', file]);
      } else {
        diff = await this.git.diff();
      }
      return diff;
    } catch (error) {
      console.error('Git diff error:', error);
      return '';
    }
  }

  async getStagedDiff(file?: string): Promise<string> {
    if (!this.isRepo) return '';

    try {
      let diff: string;
      if (file) {
        diff = await this.git.diff(['--cached', '--', file]);
      } else {
        diff = await this.git.diff(['--cached']);
      }
      return diff;
    } catch (error) {
      console.error('Git staged diff error:', error);
      return '';
    }
  }

  async stage(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.add(files);
    } catch (error) {
      console.error('Git stage error:', error);
      throw error;
    }
  }

  async unstage(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.reset(['HEAD', ...files]);
    } catch (error) {
      console.error('Git unstage error:', error);
      throw error;
    }
  }

  async commit(message: string): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.commit(message);
    } catch (error) {
      console.error('Git commit error:', error);
      throw error;
    }
  }

  async discard(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.checkout(['--', ...files]);
    } catch (error) {
      console.error('Git discard error:', error);
      throw error;
    }
  }

  async push(): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.push();
    } catch (error) {
      console.error('Git push error:', error);
      throw error;
    }
  }

  async pull(): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.pull();
    } catch (error) {
      console.error('Git pull error:', error);
      throw error;
    }
  }

  async fetch(): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.fetch();
    } catch (error) {
      console.error('Git fetch error:', error);
      throw error;
    }
  }

  async switchBranch(branchName: string): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.checkout(branchName);
    } catch (error) {
      console.error('Git switch branch error:', error);
      throw error;
    }
  }

  async createBranch(branchName: string): Promise<void> {
    if (!this.isRepo) return;

    try {
      await this.git.checkoutLocalBranch(branchName);
    } catch (error) {
      console.error('Git create branch error:', error);
      throw error;
    }
  }

  getRepoPath(): string {
    return this.repoPath;
  }

  getIsRepo(): boolean {
    return this.isRepo;
  }
  
  async initRepo(): Promise<void> {
    try {
      await this.git.init();
      this.isRepo = true;
      console.log('Git repository initialized at:', this.repoPath);
    } catch (error) {
      console.error('Failed to initialize Git repository:', error);
      throw error;
    }
  }
}