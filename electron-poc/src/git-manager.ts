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
    // Use provided path or no path (for when no folder is open)
    this.repoPath = repoPath || '';
    if (this.repoPath) {
      this.git = simpleGit(this.repoPath);
      this.checkIfRepo();
    } else {
      // No folder open - not a repo
      this.isRepo = false;
      this.git = simpleGit();
    }
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

  async getLog(options: { maxCount?: number; graph?: boolean; oneline?: boolean; limit?: number } = {}): Promise<string> {
    if (!this.isRepo) {
      console.log('[GitManager] Not a repo, returning empty log');
      return '';
    }

    try {
      // Use raw git command for more control over format
      const args = ['log'];
      
      const maxCount = options.maxCount || options.limit || 50;
      console.log('[GitManager] Using maxCount:', maxCount);
      args.push(`-${maxCount}`);
      
      // For now, skip graph decorations to simplify parsing
      // if (options.graph) {
      //   args.push('--graph');
      // }
      
      if (options.oneline) {
        args.push('--oneline');
      } else {
        // Use a simpler format with newlines between commits
        args.push('--pretty=format:COMMIT_START|%H|%an|%ae|%ad|%s|COMMIT_END%n');
      }
      
      console.log('[GitManager] Git log args:', args);
      const result = await this.git.raw(args);
      console.log('[GitManager] Git log result length:', result ? result.length : 0);
      return result || '';
    } catch (error) {
      console.error('[GitManager] Git log error:', error);
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
  
  async getCommitFiles(hash: string): Promise<{ files: any[] }> {
    if (!this.isRepo) return { files: [] };
    
    try {
      // Get the list of files changed in this commit
      const result = await this.git.raw(['show', '--name-status', '--format=', hash]);
      const lines = result.split('\n').filter(line => line.trim());
      
      const files = lines.map(line => {
        const parts = line.split('\t');
        if (parts.length >= 2) {
          return {
            status: parts[0],
            path: parts[1],
            additions: 0,
            deletions: 0
          };
        }
        return null;
      }).filter(f => f);
      
      return { files };
    } catch (error) {
      console.error('Failed to get commit files:', error);
      return { files: [] };
    }
  }

  async getFileDiff(commitHash: string, filePath: string): Promise<string> {
    if (!this.isRepo) return '';
    
    try {
      // Get the diff for a specific file in a commit
      // Use proper diff format with unified context
      const result = await this.git.raw(['diff', `${commitHash}^..${commitHash}`, '--', filePath]);
      
      // If the file was added in this commit (no parent), show the full file as added
      if (!result || result.trim() === '') {
        const fileContent = await this.git.raw(['show', `${commitHash}:${filePath}`]);
        if (fileContent) {
          // Format as an addition diff
          const lines = fileContent.split('\n');
          const diff = `diff --git a/${filePath} b/${filePath}
new file mode 100644
index 0000000..0000000
--- /dev/null
+++ b/${filePath}
${lines.map(line => '+' + line).join('\n')}`;
          return diff;
        }
      }
      
      return result || '';
    } catch (error) {
      // Try alternative method for first commit or added files
      try {
        const fileContent = await this.git.raw(['show', `${commitHash}:${filePath}`]);
        if (fileContent) {
          const lines = fileContent.split('\n');
          const diff = `diff --git a/${filePath} b/${filePath}
new file mode 100644
index 0000000..0000000
--- /dev/null
+++ b/${filePath}
${lines.map(line => '+' + line).join('\n')}`;
          return diff;
        }
      } catch (innerError) {
        console.error('Failed to get file diff:', innerError);
      }
      return '';
    }
  }
}