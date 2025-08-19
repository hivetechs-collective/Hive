/**
 * Enhanced Git Manager
 * Integrates the authentication system with Git operations
 * Enterprise-class Git management based on VS Code's architecture
 */

import * as simpleGit from 'simple-git';
import { GitAuthenticationManager } from './authentication/GitAuthenticationManager';
import { spawn } from 'child_process';
import * as path from 'path';

export interface GitOperationResult {
  success: boolean;
  output?: string;
  error?: string;
}

export class EnhancedGitManager {
  private git: simpleGit.SimpleGit;
  private static authManager: GitAuthenticationManager | null = null;
  private repoPath: string;
  private isInitialized = false;
  
  constructor(repoPath: string) {
    this.repoPath = repoPath;
    this.git = simpleGit.simpleGit(repoPath);
    
    // Use singleton authentication manager to avoid duplicate IPC handlers
    if (!EnhancedGitManager.authManager) {
      EnhancedGitManager.authManager = new GitAuthenticationManager({
        enableCache: true,
        cacheDuration: 300000, // 5 minutes
        useSystemCredentialManager: true,
        enableOAuth: true,
      });
    }
    
    console.log('[EnhancedGitManager] Initialized for:', repoPath);
  }
  
  /**
   * Initialize the Git manager
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) return;
    
    console.log('[EnhancedGitManager] Initializing...');
    
    // Initialize authentication manager
    await EnhancedGitManager.authManager!.initialize();
    
    // Configure Git settings
    await this.configureGit();
    
    this.isInitialized = true;
    console.log('[EnhancedGitManager] Initialization complete');
  }
  
  /**
   * Configure Git settings
   */
  private async configureGit(): Promise<void> {
    try {
      // Set push.autoSetupRemote for easier branch pushing
      await this.git.addConfig('push.autoSetupRemote', 'true');
      
      // Ensure credential helper is not interfering
      // We'll handle credentials ourselves
      await this.git.addConfig('credential.helper', '');
      
      console.log('[EnhancedGitManager] Git configured');
    } catch (error) {
      console.error('[EnhancedGitManager] Error configuring Git:', error);
    }
  }
  
  /**
   * Get repository status
   */
  async getStatus(): Promise<any> {
    try {
      // First check if we're actually in a Git repository
      const isRepo = await this.git.checkIsRepo();
      
      if (!isRepo) {
        // Not in a Git repository, return null to show welcome screen
        console.log('[EnhancedGitManager] Not in a Git repository');
        return null;
      }
      
      const status = await this.git.status();
      // Convert to plain object to avoid serialization issues in IPC
      return {
        isRepo: true, // We confirmed we're in a repo
        current: status.current,
        tracking: status.tracking,
        ahead: status.ahead,
        behind: status.behind,
        files: status.files,
        staged: status.staged,
        renamed: status.renamed,
        deleted: status.deleted,
        modified: status.modified,
        created: status.created,
        conflicted: status.conflicted,
        isClean: status.isClean(),
        branch: status.current || 'master',
        hasUpstream: !!status.tracking
      };
    } catch (error) {
      console.error('[EnhancedGitManager] Error getting status:', error);
      // If there's an error, we're probably not in a repo
      return null;
    }
  }
  
  /**
   * Stage files
   */
  async stage(files: string[]): Promise<void> {
    await this.git.add(files);
  }
  
  /**
   * Unstage files
   */
  async unstage(files: string[]): Promise<void> {
    await this.git.reset(['HEAD', ...files]);
  }
  
  /**
   * Commit changes
   */
  async commit(message: string): Promise<void> {
    await this.git.commit(message);
  }
  
  /**
   * Push changes with authentication support
   */
  async push(): Promise<GitOperationResult> {
    console.log('[EnhancedGitManager] Starting push operation...');
    
    try {
      // Ensure we're initialized
      await this.initialize();
      
      // Get current branch
      const status = await this.git.status();
      const branch = status.current;
      
      if (!branch) {
        return { 
          success: false, 
          error: 'No current branch' 
        };
      }
      
      console.log(`[EnhancedGitManager] Pushing branch: ${branch}`);
      
      // Check if branch has upstream
      const branches = await this.git.branch(['-vv']);
      const currentBranchInfo = branches.branches[branch];
      const hasUpstream = currentBranchInfo && (currentBranchInfo as any).tracking;
      
      // Prepare push arguments
      const args = hasUpstream 
        ? ['push', 'origin', branch]
        : ['push', '--set-upstream', 'origin', branch];
      
      console.log(`[EnhancedGitManager] Push command: git ${args.join(' ')}`);
      
      // Execute with authentication
      const result = await EnhancedGitManager.authManager!.executeGitCommand(args, this.repoPath);
      
      if (result.code === 0) {
        console.log('[EnhancedGitManager] Push successful');
        return { 
          success: true, 
          output: result.stdout || result.stderr 
        };
      } else {
        console.error('[EnhancedGitManager] Push failed:', result.stderr);
        return { 
          success: false, 
          error: result.stderr || 'Push failed' 
        };
      }
    } catch (error: any) {
      console.error('[EnhancedGitManager] Push error:', error);
      return { 
        success: false, 
        error: error.message || 'Unknown error' 
      };
    }
  }
  
  /**
   * Pull changes with authentication support
   */
  async pull(): Promise<GitOperationResult> {
    console.log('[EnhancedGitManager] Starting pull operation...');
    
    try {
      // Ensure we're initialized
      await this.initialize();
      
      // Get current branch
      const status = await this.git.status();
      const branch = status.current;
      
      if (!branch) {
        return { 
          success: false, 
          error: 'No current branch' 
        };
      }
      
      console.log(`[EnhancedGitManager] Pulling branch: ${branch}`);
      
      // Execute with authentication
      const result = await EnhancedGitManager.authManager!.executeGitCommand(
        ['pull', 'origin', branch],
        this.repoPath
      );
      
      if (result.code === 0) {
        console.log('[EnhancedGitManager] Pull successful');
        return { 
          success: true, 
          output: result.stdout || result.stderr 
        };
      } else {
        console.error('[EnhancedGitManager] Pull failed:', result.stderr);
        return { 
          success: false, 
          error: result.stderr || 'Pull failed' 
        };
      }
    } catch (error: any) {
      console.error('[EnhancedGitManager] Pull error:', error);
      return { 
        success: false, 
        error: error.message || 'Unknown error' 
      };
    }
  }
  
  /**
   * Sync (pull then push) with authentication support
   */
  async sync(): Promise<GitOperationResult> {
    console.log('[EnhancedGitManager] Starting sync operation...');
    
    // First pull
    const pullResult = await this.pull();
    if (!pullResult.success) {
      return pullResult;
    }
    
    // Then push
    const pushResult = await this.push();
    return pushResult;
  }
  
  /**
   * Fetch remote changes
   */
  async fetch(): Promise<GitOperationResult> {
    console.log('[EnhancedGitManager] Starting fetch operation...');
    
    try {
      // Ensure we're initialized
      await this.initialize();
      
      // Execute with authentication
      const result = await EnhancedGitManager.authManager!.executeGitCommand(
        ['fetch', '--all', '--prune'],
        this.repoPath
      );
      
      if (result.code === 0) {
        console.log('[EnhancedGitManager] Fetch successful');
        return { 
          success: true, 
          output: result.stdout || result.stderr 
        };
      } else {
        console.error('[EnhancedGitManager] Fetch failed:', result.stderr);
        return { 
          success: false, 
          error: result.stderr || 'Fetch failed' 
        };
      }
    } catch (error: any) {
      console.error('[EnhancedGitManager] Fetch error:', error);
      return { 
        success: false, 
        error: error.message || 'Unknown error' 
      };
    }
  }
  
  /**
   * Clone a repository with authentication support
   */
  async clone(url: string, destination: string): Promise<GitOperationResult> {
    console.log(`[EnhancedGitManager] Cloning ${url} to ${destination}`);
    
    try {
      // Ensure we're initialized
      await this.initialize();
      
      // Execute with authentication
      const result = await EnhancedGitManager.authManager!.executeGitCommand(
        ['clone', url, destination],
        path.dirname(destination)
      );
      
      if (result.code === 0) {
        console.log('[EnhancedGitManager] Clone successful');
        return { 
          success: true, 
          output: result.stdout || result.stderr 
        };
      } else {
        console.error('[EnhancedGitManager] Clone failed:', result.stderr);
        return { 
          success: false, 
          error: result.stderr || 'Clone failed' 
        };
      }
    } catch (error: any) {
      console.error('[EnhancedGitManager] Clone error:', error);
      return { 
        success: false, 
        error: error.message || 'Unknown error' 
      };
    }
  }
  
  /**
   * Get commit log
   */
  async getLog(options: { maxCount?: number; graph?: boolean } = {}): Promise<string> {
    const args = ['log'];
    
    if (options.graph) {
      args.push('--graph');
    }
    
    if (options.maxCount) {
      args.push(`-${options.maxCount}`);
    }
    
    // Use markers to help parsing
    args.push('--pretty=format:COMMIT_START|%H|%an|%ae|%ad|%s|COMMIT_END');
    
    const result = await this.git.raw(args);
    return result;
  }
  
  /**
   * Get branches
   */
  async getBranches(): Promise<any> {
    const branches = await this.git.branch();
    // Convert to plain object to avoid serialization issues
    return {
      all: branches.all,
      branches: branches.branches,
      current: branches.current,
      detached: branches.detached
    };
  }
  
  /**
   * Switch branch
   */
  async switchBranch(branchName: string): Promise<void> {
    await this.git.checkout(branchName);
  }
  
  /**
   * Create new branch
   */
  async createBranch(branchName: string): Promise<void> {
    await this.git.checkoutLocalBranch(branchName);
  }
  
  /**
   * Get diff
   */
  async getDiff(file?: string): Promise<string> {
    if (file) {
      return await this.git.diff(['--', file]);
    }
    return await this.git.diff();
  }
  
  /**
   * Get staged diff
   */
  async getStagedDiff(file?: string): Promise<string> {
    if (file) {
      return await this.git.diff(['--cached', '--', file]);
    }
    return await this.git.diff(['--cached']);
  }
  
  /**
   * Discard changes
   */
  async discard(files: string[]): Promise<void> {
    await this.git.checkout(['--', ...files]);
  }
  
  /**
   * Initialize Git repository
   */
  async initRepo(): Promise<void> {
    await this.git.init();
  }
  
  /**
   * Get files changed in a commit
   */
  async getCommitFiles(hash: string): Promise<string[]> {
    const result = await this.git.raw(['diff-tree', '--no-commit-id', '--name-only', '-r', hash]);
    return result.split('\n').filter(file => file.length > 0);
  }
  
  /**
   * Get diff for a specific file in a commit
   */
  async getFileDiff(commitHash: string, filePath: string): Promise<string> {
    // Get the diff of the file between the commit and its parent
    return await this.git.raw(['diff', `${commitHash}~1..${commitHash}`, '--', filePath]);
  }
  
  /**
   * Get remote info
   */
  async getRemoteInfo(): Promise<{ ahead: number; behind: number }> {
    try {
      const status = await this.git.status();
      return {
        ahead: status.ahead || 0,
        behind: status.behind || 0
      };
    } catch {
      return { ahead: 0, behind: 0 };
    }
  }
  
  /**
   * Clean up resources
   */
  dispose(): void {
    // Don't dispose the singleton auth manager
    // It will be reused across folder changes
  }
}