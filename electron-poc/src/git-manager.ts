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
      // No folder open - definitively not a repo
      this.isRepo = false;
      // Don't create a git instance when no folder is open
      this.git = null as any;
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
    repoPath?: string;
  }> {
    // Return empty status if no folder is open or not a repo
    if (!this.repoPath || !this.isRepo || !this.git) {
      return {
        files: [],
        branch: '',
        ahead: 0,
        behind: 0,
        isRepo: false,
        repoPath: this.repoPath
      };
    }

    try {
      // Fetch first to get accurate ahead/behind counts
      try {
        await this.git.fetch();
      } catch (fetchError) {
        console.log('Fetch failed (may be offline):', fetchError);
      }
      
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
        ahead: status.ahead || 0,
        behind: status.behind || 0,
        isRepo: true,
        repoPath: this.repoPath
      };
    } catch (error) {
      console.error('Git status error:', error);
      throw error;
    }
  }

  async getBranches(): Promise<GitBranch[]> {
    if (!this.repoPath || !this.isRepo || !this.git) return [];

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
    if (!this.repoPath || !this.isRepo || !this.git) {
      console.log('[GitManager] Not a repo or no folder open, returning empty log');
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
      // Remove lock file if it exists
      const lockPath = path.join(this.repoPath, '.git', 'index.lock');
      if (fs.existsSync(lockPath)) {
        fs.unlinkSync(lockPath);
      }
      
      await this.git.add(files);
    } catch (error) {
      console.error('Git stage error:', error);
      throw error;
    }
  }

  async unstage(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    try {
      // Remove lock file if it exists
      const lockPath = path.join(this.repoPath, '.git', 'index.lock');
      if (fs.existsSync(lockPath)) {
        fs.unlinkSync(lockPath);
      }
      
      await this.git.reset(['HEAD', ...files]);
    } catch (error) {
      console.error('Git unstage error:', error);
      throw error;
    }
  }

  async commit(message: string): Promise<void> {
    if (!this.isRepo) return;

    try {
      // Remove lock file if it exists
      const lockPath = path.join(this.repoPath, '.git', 'index.lock');
      if (fs.existsSync(lockPath)) {
        fs.unlinkSync(lockPath);
      }
      
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
      // Get current branch status first
      const status = await this.git.status();
      const currentBranch = status.current;
      
      if (!currentBranch) {
        throw new Error('No current branch');
      }

      // Check if branch has upstream
      const branches = await this.git.branch(['-vv']);
      const currentBranchInfo = branches.branches[currentBranch];
      const hasUpstream = currentBranchInfo && (currentBranchInfo as any).tracking;

      if (!hasUpstream) {
        console.log(`No upstream for ${currentBranch}, setting upstream...`);
        // Push with --set-upstream
        await this.git.push(['--set-upstream', 'origin', currentBranch]);
        console.log('Successfully pushed with upstream set');
      } else {
        // Regular push
        await this.git.push();
        console.log('Successfully pushed');
      }
    } catch (error: any) {
      console.error('Git push error:', error);
      throw error;
    }
  }

  async pull(): Promise<void> {
    if (!this.isRepo) return;

    try {
      // Get current branch status
      const status = await this.git.status();
      const currentBranch = status.current;
      
      if (!currentBranch) {
        throw new Error('No current branch');
      }

      // Check if branch has upstream
      const branches = await this.git.branch(['-vv']);
      const currentBranchInfo = branches.branches[currentBranch];
      const hasUpstream = currentBranchInfo && (currentBranchInfo as any).tracking;

      if (!hasUpstream) {
        console.log(`No upstream for ${currentBranch}, setting upstream first...`);
        // Set upstream to track origin/branch
        await this.git.branch(['--set-upstream-to', `origin/${currentBranch}`, currentBranch]);
        console.log('Upstream set, now pulling...');
      }
      
      // Now pull
      await this.git.pull();
      console.log('Successfully pulled');
    } catch (error: any) {
      // If pull fails because remote branch doesn't exist, that's okay
      if (error.message && error.message.includes('no such ref was fetched')) {
        console.log('Remote branch does not exist yet - nothing to pull');
      } else {
        console.error('Git pull error:', error);
        throw error;
      }
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