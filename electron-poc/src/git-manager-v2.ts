import { GitExecutor, GitError, GitErrorCode } from './git-executor';
import { GitOperationQueue } from './git-operation-queue';
import * as path from 'path';
import * as fs from 'fs';

export interface GitFileStatus {
  path: string;
  index: string;  // X in XY format - staged status
  working: string; // Y in XY format - working tree status
  renamed?: string;
}

export interface GitStatus {
  files: GitFileStatus[];
  branch: string;
  ahead: number;
  behind: number;
  isRepo: boolean;
  repoPath?: string;
  hasUpstream?: boolean;
  upstream?: string;
}

export interface GitBranch {
  name: string;
  current: boolean;
  commit: string;
  upstream?: string;
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

export class GitManagerV2 {
  private executor: GitExecutor;
  private queue: GitOperationQueue;
  private repoPath: string;
  private isRepo: boolean = false;
  private cachedStatus: GitStatus | null = null;
  private statusCacheTime: number = 0;
  private readonly CACHE_DURATION = 500; // 500ms cache for status

  constructor(repoPath?: string) {
    this.repoPath = repoPath || '';
    this.queue = new GitOperationQueue();
    
    if (this.repoPath) {
      this.executor = new GitExecutor(this.repoPath);
      this.checkIfRepo();
      this.configureGit();
    } else {
      this.isRepo = false;
      this.executor = null as any;
    }
  }

  private async checkIfRepo(): Promise<void> {
    try {
      await this.executor.exec(['rev-parse', '--git-dir']);
      this.isRepo = true;
      console.log('[GitManagerV2] Repository detected at:', this.repoPath);
    } catch (error) {
      this.isRepo = false;
      console.log('[GitManagerV2] Not a Git repository:', this.repoPath);
    }
  }

  private async configureGit(): Promise<void> {
    try {
      // Enable automatic upstream for new branches (Git 2.37+)
      await this.executor.exec(['config', 'push.autoSetupRemote', 'true']);
      console.log('[GitManagerV2] Configured push.autoSetupRemote');
      
      // Ensure credential helper is set for macOS
      if (process.platform === 'darwin') {
        await this.executor.exec(['config', 'credential.helper', 'osxkeychain']);
        console.log('[GitManagerV2] Configured macOS credential helper');
      }
    } catch (error) {
      console.log('[GitManagerV2] Could not configure Git settings:', error);
    }
  }

  async getStatus(): Promise<GitStatus> {
    // Return cached status if still fresh
    if (this.cachedStatus && Date.now() - this.statusCacheTime < this.CACHE_DURATION) {
      return this.cachedStatus;
    }

    // Return empty status if no folder is open or not a repo
    if (!this.repoPath || !this.isRepo || !this.executor) {
      return {
        files: [],
        branch: '',
        ahead: 0,
        behind: 0,
        isRepo: false,
        repoPath: this.repoPath,
      };
    }

    // Use priority execution for status (doesn't wait in queue)
    return await this.queue.executePriority(async () => {
      try {
        console.log('[GitManagerV2] Getting status...');
        
        // Fetch first to get accurate ahead/behind counts (but don't block on it)
        this.executor.fetch({ prune: true }).catch(err => 
          console.log('[GitManagerV2] Background fetch failed (may be offline):', err)
        );

        const status = await this.executor.status();
        
        const result: GitStatus = {
          files: status.files || [],
          branch: status.branch || '',
          ahead: status.ahead || 0,
          behind: status.behind || 0,
          isRepo: true,
          repoPath: this.repoPath,
          hasUpstream: !!status.upstream,
          upstream: status.upstream,
        };

        // If no upstream but we have local commits, check if we have commits to push
        if (!result.hasUpstream && result.branch) {
          try {
            // Count commits ahead of origin/master (or origin/main)
            let baseRef = 'origin/master';
            try {
              // Check if origin/master exists
              await this.executor.exec(['rev-parse', '--verify', 'origin/master']);
            } catch {
              // Try origin/main instead
              try {
                await this.executor.exec(['rev-parse', '--verify', 'origin/main']);
                baseRef = 'origin/main';
              } catch {
                // No base branch found, can't count ahead
                console.log(`[GitManagerV2] No base branch found for comparison`);
                return result;
              }
            }
            
            // Count commits ahead of base branch
            const logResult = await this.executor.exec(['log', '--oneline', `${baseRef}..HEAD`]);
            const commits = logResult.stdout.trim().split('\n').filter(l => l);
            result.ahead = commits.length;
            console.log(`[GitManagerV2] Branch ${result.branch} has ${result.ahead} commits ahead of ${baseRef}`);
          } catch (error) {
            console.log('[GitManagerV2] Could not count unpushed commits:', error);
          }
        }

        // Cache the result
        this.cachedStatus = result;
        this.statusCacheTime = Date.now();

        return result;
      } catch (error) {
        console.error('[GitManagerV2] Failed to get status:', error);
        return {
          files: [],
          branch: '',
          ahead: 0,
          behind: 0,
          isRepo: false,
          repoPath: this.repoPath,
        };
      }
    });
  }

  async stage(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('stage', async () => {
      try {
        // Clear any lock file first
        this.clearLockFile();
        
        await this.executor.add(files);
        console.log('[GitManagerV2] Staged files:', files);
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Failed to stage files:', error);
        throw error;
      }
    });
  }

  async unstage(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('unstage', async () => {
      try {
        // Clear any lock file first
        this.clearLockFile();
        
        await this.executor.reset(files);
        console.log('[GitManagerV2] Unstaged files:', files);
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Failed to unstage files:', error);
        throw error;
      }
    });
  }

  async commit(message: string): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('commit', async () => {
      try {
        // Clear any lock file first
        this.clearLockFile();
        
        await this.executor.commit(message);
        console.log('[GitManagerV2] Committed with message:', message);
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Failed to commit:', error);
        throw error;
      }
    });
  }

  async push(): Promise<void> {
    if (!this.isRepo) {
      console.log('[GitManagerV2] Not a repo, cannot push');
      return;
    }

    return this.queue.enqueue('push', async () => {
      try {
        console.log('[GitManagerV2] Starting push operation...');
        
        // Get current branch directly without using queue (we're already in queue!)
        const statusResult = await this.executor.status();
        const branch = statusResult.branch;
        
        if (!branch) {
          throw new Error('No current branch');
        }

        console.log(`[GitManagerV2] Pushing branch: ${branch}, upstream: ${statusResult.upstream}, ahead: ${statusResult.ahead}`);

        // Determine if we need to set upstream
        const needsUpstream = !statusResult.upstream;
        
        if (needsUpstream) {
          console.log(`[GitManagerV2] Setting upstream for ${branch}`);
          await this.executor.push({
            remote: 'origin',
            branch: branch,
            setUpstream: true,
          });
          console.log('[GitManagerV2] Successfully pushed with upstream set');
        } else {
          console.log('[GitManagerV2] Performing regular push');
          await this.executor.push({
            remote: 'origin',
            branch: branch,
          });
          console.log('[GitManagerV2] Successfully pushed');
        }
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error: any) {
        console.error('[GitManagerV2] Push failed:', error);
        
        // Handle specific error codes
        if (error instanceof GitError) {
          switch (error.data.gitErrorCode) {
            case GitErrorCode.AuthenticationFailed:
              throw new Error('Git authentication failed. Please check your credentials.');
            case GitErrorCode.PushRejected:
              throw new Error('Push was rejected. You may need to pull first.');
            case GitErrorCode.RemoteConnectionError:
              throw new Error('Could not connect to remote repository.');
            case GitErrorCode.NoUpstreamBranch:
              // This shouldn't happen as we handle it, but just in case
              throw new Error('No upstream branch configured.');
            default:
              throw new Error(error.data.message || 'Push failed');
          }
        }
        
        throw error;
      }
    });
  }

  async pull(): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('pull', async () => {
      try {
        console.log('[GitManagerV2] Starting pull operation...');
        
        // Get current branch directly without using queue (we're already in queue!)
        const statusResult = await this.executor.status();
        const branch = statusResult.branch;
        
        if (!branch) {
          throw new Error('No current branch');
        }

        // Check if we have an upstream
        if (!statusResult.upstream) {
          console.log(`[GitManagerV2] No upstream for ${branch}, attempting to set...`);
          
          // Try to set upstream to origin/branch
          try {
            await this.executor.exec(['branch', '--set-upstream-to', `origin/${branch}`, branch]);
            console.log('[GitManagerV2] Upstream set, now pulling...');
          } catch (error) {
            console.log('[GitManagerV2] Could not set upstream (remote branch may not exist)');
            throw new Error('No remote branch to pull from');
          }
        }

        // Perform the pull
        await this.executor.pull({
          remote: 'origin',
          branch: branch,
        });
        
        console.log('[GitManagerV2] Successfully pulled');
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error: any) {
        console.error('[GitManagerV2] Pull failed:', error);
        
        // Handle specific error codes
        if (error instanceof GitError) {
          switch (error.data.gitErrorCode) {
            case GitErrorCode.AuthenticationFailed:
              throw new Error('Git authentication failed. Please check your credentials.');
            case GitErrorCode.Conflict:
              throw new Error('Merge conflicts detected. Please resolve conflicts manually.');
            case GitErrorCode.DirtyWorkTree:
              throw new Error('You have uncommitted changes. Please commit or stash them first.');
            case GitErrorCode.RemoteConnectionError:
              throw new Error('Could not connect to remote repository.');
            default:
              throw new Error(error.data.message || 'Pull failed');
          }
        }
        
        throw error;
      }
    });
  }

  async sync(): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('sync', async () => {
      try {
        console.log('[GitManagerV2] Starting sync operation (pull then push)...');
        
        // Get current branch directly
        const statusResult = await this.executor.status();
        const branch = statusResult.branch;
        
        if (!branch) {
          throw new Error('No current branch');
        }
        
        // First pull
        console.log('[GitManagerV2] Sync: Pulling...');
        await this.executor.pull({
          remote: 'origin',
          branch: branch,
        });
        
        // Then push
        console.log('[GitManagerV2] Sync: Pushing...');
        const needsUpstream = !statusResult.upstream;
        if (needsUpstream) {
          await this.executor.push({
            remote: 'origin',
            branch: branch,
            setUpstream: true,
          });
        } else {
          await this.executor.push({
            remote: 'origin',
            branch: branch,
          });
        }
        
        console.log('[GitManagerV2] Successfully synced');
      } catch (error) {
        console.error('[GitManagerV2] Sync failed:', error);
        throw error;
      }
    });
  }

  async fetch(): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('fetch', async () => {
      try {
        await this.executor.fetch({ all: true, prune: true });
        console.log('[GitManagerV2] Successfully fetched');
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Fetch failed:', error);
        throw error;
      }
    });
  }

  async getBranches(): Promise<GitBranch[]> {
    if (!this.isRepo) return [];

    try {
      const branches = await this.executor.branch({ all: true });
      return branches;
    } catch (error) {
      console.error('[GitManagerV2] Failed to get branches:', error);
      return [];
    }
  }

  async switchBranch(branchName: string): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('checkout', async () => {
      try {
        await this.executor.checkout(branchName);
        console.log('[GitManagerV2] Switched to branch:', branchName);
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Failed to switch branch:', error);
        throw error;
      }
    });
  }

  async createBranch(branchName: string): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('branch', async () => {
      try {
        await this.executor.exec(['checkout', '-b', branchName]);
        console.log('[GitManagerV2] Created and switched to branch:', branchName);
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Failed to create branch:', error);
        throw error;
      }
    });
  }

  async getLog(options: { maxCount?: number; graph?: boolean; oneline?: boolean; limit?: number } = {}): Promise<string> {
    if (!this.isRepo) return '';

    try {
      // Pass all options including graph
      return await this.executor.log(options);
    } catch (error) {
      console.error('[GitManagerV2] Failed to get log:', error);
      return '';
    }
  }

  async getDiff(file?: string): Promise<string> {
    if (!this.isRepo) return '';

    try {
      const args = ['diff'];
      if (file) {
        args.push('--', file);
      }
      const result = await this.executor.exec(args);
      return result.stdout;
    } catch (error) {
      console.error('[GitManagerV2] Failed to get diff:', error);
      return '';
    }
  }

  async getStagedDiff(file?: string): Promise<string> {
    if (!this.isRepo) return '';

    try {
      const args = ['diff', '--cached'];
      if (file) {
        args.push('--', file);
      }
      const result = await this.executor.exec(args);
      return result.stdout;
    } catch (error) {
      console.error('[GitManagerV2] Failed to get staged diff:', error);
      return '';
    }
  }

  async discard(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('checkout', async () => {
      try {
        await this.executor.exec(['checkout', '--', ...files]);
        console.log('[GitManagerV2] Discarded changes in files:', files);
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Failed to discard changes:', error);
        throw error;
      }
    });
  }

  async clean(files: string[]): Promise<void> {
    if (!this.isRepo) return;

    return this.queue.enqueue('clean', async () => {
      try {
        // Use git clean -f to remove untracked files
        // We specify each file explicitly for safety
        for (const file of files) {
          await this.executor.exec(['clean', '-f', '--', file]);
        }
        console.log('[GitManagerV2] Cleaned untracked files:', files);
        
        // Invalidate status cache
        this.cachedStatus = null;
      } catch (error) {
        console.error('[GitManagerV2] Failed to clean files:', error);
        throw error;
      }
    });
  }

  async initRepo(): Promise<void> {
    try {
      await this.executor.exec(['init']);
      this.isRepo = true;
      console.log('[GitManagerV2] Initialized repository');
    } catch (error) {
      console.error('[GitManagerV2] Failed to initialize repository:', error);
      throw error;
    }
  }

  async getCommitFiles(hash: string): Promise<string[]> {
    if (!this.isRepo) return [];

    try {
      const result = await this.executor.exec(['show', '--name-only', '--pretty=format:', hash]);
      return result.stdout.trim().split('\n').filter(f => f);
    } catch (error) {
      console.error('[GitManagerV2] Failed to get commit files:', error);
      return [];
    }
  }

  async getFileDiff(commitHash: string, filePath: string): Promise<string> {
    if (!this.isRepo) return '';

    try {
      const result = await this.executor.exec(['show', `${commitHash}:${filePath}`]);
      return result.stdout;
    } catch (error) {
      console.error('[GitManagerV2] Failed to get file diff:', error);
      return '';
    }
  }

  private clearLockFile(): void {
    if (!this.repoPath) return;
    
    const lockPath = path.join(this.repoPath, '.git', 'index.lock');
    if (fs.existsSync(lockPath)) {
      try {
        fs.unlinkSync(lockPath);
        console.log('[GitManagerV2] Cleared lock file');
      } catch (error) {
        console.error('[GitManagerV2] Failed to clear lock file:', error);
      }
    }
  }
}