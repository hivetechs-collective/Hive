/**
 * Git Chunked Push Strategy
 * Handles large repository pushes by splitting them into manageable chunks
 */

// Note: This module should be used from the main process only
// The renderer will call these functions through IPC
import { notifications } from './notification';

// We'll use window.gitAPI to execute git commands instead
const execAsync = async (command: string, options?: any): Promise<{ stdout: string; stderr?: string }> => {
  // This will be called through IPC in the actual implementation
  throw new Error('Git commands must be executed through the main process');
};

export interface ChunkedPushOptions {
  remote?: string;
  branch?: string;
  chunkSize?: number; // Number of commits per chunk
  maxPackSize?: number; // Max size in bytes per push (default 1.5GB to stay under 2GB)
}

export interface PushProgress {
  totalCommits: number;
  pushedCommits: number;
  currentChunk: number;
  totalChunks: number;
  status: 'preparing' | 'pushing' | 'completed' | 'failed';
  error?: string;
}

export class GitChunkedPush {
  private repoPath: string;
  private onProgress?: (progress: PushProgress) => void;
  
  constructor(repoPath: string, onProgress?: (progress: PushProgress) => void) {
    this.repoPath = repoPath;
    this.onProgress = onProgress;
  }
  
  /**
   * Push commits in chunks to avoid GitHub's 2GB limit
   */
  async pushInChunks(options: ChunkedPushOptions = {}): Promise<void> {
    const {
      remote = 'origin',
      branch = await this.getCurrentBranch(),
      chunkSize = 100, // Push 100 commits at a time by default
      maxPackSize = 1.5 * 1024 * 1024 * 1024 // 1.5GB default
    } = options;
    
    try {
      // Get list of unpushed commits
      const unpushedCommits = await this.getUnpushedCommits(remote, branch);
      
      if (unpushedCommits.length === 0) {
        this.updateProgress({
          totalCommits: 0,
          pushedCommits: 0,
          currentChunk: 0,
          totalChunks: 0,
          status: 'completed'
        });
        return;
      }
      
      // Calculate chunks
      const chunks = this.createChunks(unpushedCommits, chunkSize);
      const totalChunks = chunks.length;
      
      console.log(`[ChunkedPush] Pushing ${unpushedCommits.length} commits in ${totalChunks} chunks`);
      
      // Push each chunk
      let pushedCommits = 0;
      for (let i = 0; i < chunks.length; i++) {
        const chunk = chunks[i];
        const chunkNumber = i + 1;
        
        this.updateProgress({
          totalCommits: unpushedCommits.length,
          pushedCommits,
          currentChunk: chunkNumber,
          totalChunks,
          status: 'pushing'
        });
        
        // Check chunk size before pushing
        const chunkSizeBytes = await this.estimateChunkSize(chunk[0], chunk[chunk.length - 1]);
        
        if (chunkSizeBytes > maxPackSize) {
          // If chunk is too large, split it further
          console.log(`[ChunkedPush] Chunk ${chunkNumber} is too large (${this.formatBytes(chunkSizeBytes)}), splitting...`);
          const smallerChunks = await this.splitBySize(chunk, maxPackSize);
          
          for (const smallChunk of smallerChunks) {
            await this.pushChunk(remote, branch, smallChunk[smallChunk.length - 1]);
            pushedCommits += smallChunk.length;
            
            this.updateProgress({
              totalCommits: unpushedCommits.length,
              pushedCommits,
              currentChunk: chunkNumber,
              totalChunks,
              status: 'pushing'
            });
          }
        } else {
          // Push the chunk
          await this.pushChunk(remote, branch, chunk[chunk.length - 1]);
          pushedCommits += chunk.length;
        }
      }
      
      this.updateProgress({
        totalCommits: unpushedCommits.length,
        pushedCommits: unpushedCommits.length,
        currentChunk: totalChunks,
        totalChunks,
        status: 'completed'
      });
      
    } catch (error: any) {
      this.updateProgress({
        totalCommits: 0,
        pushedCommits: 0,
        currentChunk: 0,
        totalChunks: 0,
        status: 'failed',
        error: error.message
      });
      throw error;
    }
  }
  
  /**
   * Push using incremental strategy (push older commits first)
   */
  async pushIncremental(options: ChunkedPushOptions = {}): Promise<void> {
    const {
      remote = 'origin',
      branch = await this.getCurrentBranch()
    } = options;
    
    try {
      // Get the common ancestor with remote
      const { stdout: mergeBase } = await execAsync(
        `git merge-base ${remote}/${branch} ${branch}`,
        { cwd: this.repoPath }
      );
      const baseCommit = mergeBase.trim();
      
      // Get all commits between base and current
      const { stdout: commitList } = await execAsync(
        `git rev-list ${baseCommit}..${branch}`,
        { cwd: this.repoPath }
      );
      const commits = commitList.trim().split('\n').filter(Boolean).reverse(); // Oldest first
      
      if (commits.length === 0) {
        console.log('[ChunkedPush] No commits to push');
        return;
      }
      
      console.log(`[ChunkedPush] Pushing ${commits.length} commits incrementally`);
      
      // Push commits in increments
      const increment = Math.min(50, Math.ceil(commits.length / 10)); // Push in 10 batches or 50 commits at a time
      
      for (let i = 0; i < commits.length; i += increment) {
        const endIndex = Math.min(i + increment, commits.length);
        const targetCommit = commits[endIndex - 1];
        
        console.log(`[ChunkedPush] Pushing commits ${i + 1} to ${endIndex} of ${commits.length}`);
        
        await this.pushChunk(remote, branch, targetCommit);
        
        this.updateProgress({
          totalCommits: commits.length,
          pushedCommits: endIndex,
          currentChunk: Math.ceil(endIndex / increment),
          totalChunks: Math.ceil(commits.length / increment),
          status: 'pushing'
        });
      }
      
      this.updateProgress({
        totalCommits: commits.length,
        pushedCommits: commits.length,
        currentChunk: Math.ceil(commits.length / increment),
        totalChunks: Math.ceil(commits.length / increment),
        status: 'completed'
      });
      
    } catch (error: any) {
      console.error('[ChunkedPush] Incremental push failed:', error);
      throw error;
    }
  }
  
  /**
   * Alternative: Create a shallow push (useful for CI/CD)
   */
  async pushShallow(depth: number = 1): Promise<void> {
    try {
      const branch = await this.getCurrentBranch();
      
      // Create a shallow clone and push
      const { stdout } = await execAsync(
        `git push --depth=${depth} origin ${branch}`,
        { cwd: this.repoPath }
      );
      
      console.log('[ChunkedPush] Shallow push completed:', stdout);
    } catch (error: any) {
      console.error('[ChunkedPush] Shallow push failed:', error);
      throw error;
    }
  }
  
  /**
   * Get current branch name
   */
  private async getCurrentBranch(): Promise<string> {
    const { stdout } = await execAsync(
      'git rev-parse --abbrev-ref HEAD',
      { cwd: this.repoPath }
    );
    return stdout.trim();
  }
  
  /**
   * Get list of unpushed commits
   */
  private async getUnpushedCommits(remote: string, branch: string): Promise<string[]> {
    try {
      const { stdout } = await execAsync(
        `git rev-list ${remote}/${branch}..${branch} 2>/dev/null || git rev-list ${branch}`,
        { cwd: this.repoPath }
      );
      return stdout.trim().split('\n').filter(Boolean).reverse(); // Return in chronological order
    } catch (error) {
      // If remote branch doesn't exist, return all commits
      const { stdout } = await execAsync(
        `git rev-list ${branch}`,
        { cwd: this.repoPath }
      );
      return stdout.trim().split('\n').filter(Boolean).reverse();
    }
  }
  
  /**
   * Create chunks of commits
   */
  private createChunks(commits: string[], chunkSize: number): string[][] {
    const chunks: string[][] = [];
    for (let i = 0; i < commits.length; i += chunkSize) {
      chunks.push(commits.slice(i, i + chunkSize));
    }
    return chunks;
  }
  
  /**
   * Estimate the size of a chunk of commits
   */
  private async estimateChunkSize(fromCommit: string, toCommit: string): Promise<number> {
    try {
      const { stdout } = await execAsync(
        `git rev-list --objects ${fromCommit}..${toCommit} | git cat-file --batch-check='%(objectsize)' | awk '{sum+=$1} END {print sum}'`,
        { cwd: this.repoPath, maxBuffer: 10 * 1024 * 1024 }
      );
      return parseInt(stdout.trim()) || 0;
    } catch (error) {
      console.error('[ChunkedPush] Failed to estimate chunk size:', error);
      return 0;
    }
  }
  
  /**
   * Split commits by size limit
   */
  private async splitBySize(commits: string[], maxSize: number): Promise<string[][]> {
    const chunks: string[][] = [];
    let currentChunk: string[] = [];
    let currentSize = 0;
    
    for (const commit of commits) {
      const commitSize = await this.getCommitSize(commit);
      
      if (currentSize + commitSize > maxSize && currentChunk.length > 0) {
        chunks.push(currentChunk);
        currentChunk = [commit];
        currentSize = commitSize;
      } else {
        currentChunk.push(commit);
        currentSize += commitSize;
      }
    }
    
    if (currentChunk.length > 0) {
      chunks.push(currentChunk);
    }
    
    return chunks;
  }
  
  /**
   * Get size of a single commit
   */
  private async getCommitSize(commit: string): Promise<number> {
    try {
      const { stdout } = await execAsync(
        `git cat-file -s ${commit}`,
        { cwd: this.repoPath }
      );
      return parseInt(stdout.trim()) || 0;
    } catch (error) {
      return 0;
    }
  }
  
  /**
   * Push a specific chunk of commits
   */
  private async pushChunk(remote: string, branch: string, upToCommit: string): Promise<void> {
    try {
      console.log(`[ChunkedPush] Pushing up to commit ${upToCommit.substring(0, 7)}`);
      
      const { stdout, stderr } = await execAsync(
        `git push ${remote} ${upToCommit}:${branch}`,
        { cwd: this.repoPath, maxBuffer: 50 * 1024 * 1024 }
      );
      
      if (stderr && !stderr.includes('Everything up-to-date')) {
        console.log('[ChunkedPush] Push output:', stderr);
      }
      
      console.log('[ChunkedPush] Chunk pushed successfully');
    } catch (error: any) {
      console.error('[ChunkedPush] Failed to push chunk:', error);
      throw error;
    }
  }
  
  /**
   * Update progress callback
   */
  private updateProgress(progress: PushProgress): void {
    if (this.onProgress) {
      this.onProgress(progress);
    }
  }
  
  /**
   * Format bytes to human-readable string
   */
  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
  }
  
  /**
   * Smart push that automatically selects the best strategy
   */
  async smartPush(options: ChunkedPushOptions = {}): Promise<void> {
    const notificationId = notifications.show({
      title: 'Analyzing Repository',
      message: 'Determining best push strategy...',
      type: 'loading',
      duration: 0
    });
    
    try {
      // Check repository size
      const { stdout: packSize } = await execAsync(
        'git count-objects -vH | grep "pack-size" | cut -d: -f2',
        { cwd: this.repoPath }
      );
      
      const sizeMatch = packSize.match(/([\d.]+)\s*([KMGT]i?B)?/);
      let sizeInBytes = 0;
      
      if (sizeMatch) {
        const value = parseFloat(sizeMatch[1]);
        const unit = sizeMatch[2] || 'B';
        const multipliers: { [key: string]: number } = {
          'B': 1,
          'KiB': 1024,
          'MiB': 1024 * 1024,
          'GiB': 1024 * 1024 * 1024
        };
        sizeInBytes = value * (multipliers[unit] || 1);
      }
      
      // Choose strategy based on size
      if (sizeInBytes > 1.8 * 1024 * 1024 * 1024) {
        // Over 1.8GB - use chunked push
        notifications.update(notificationId, {
          title: 'Large Repository Detected',
          message: 'Using chunked push strategy to avoid size limits...',
          type: 'info',
          duration: 3000
        });
        
        await this.pushInChunks(options);
        
      } else if (sizeInBytes > 500 * 1024 * 1024) {
        // 500MB - 1.8GB - use incremental push
        notifications.update(notificationId, {
          title: 'Medium Repository',
          message: 'Using incremental push strategy...',
          type: 'info',
          duration: 3000
        });
        
        await this.pushIncremental(options);
        
      } else {
        // Under 500MB - regular push
        notifications.update(notificationId, {
          title: 'Pushing Changes',
          message: 'Repository size is within limits, using standard push...',
          type: 'info',
          duration: 3000
        });
        
        const branch = await this.getCurrentBranch();
        await execAsync(`git push origin ${branch}`, { cwd: this.repoPath });
      }
      
      notifications.update(notificationId, {
        title: 'Push Successful',
        message: 'All changes have been pushed to remote',
        type: 'success',
        duration: 3000
      });
      
    } catch (error: any) {
      notifications.update(notificationId, {
        title: 'Push Failed',
        message: error.message,
        type: 'error',
        duration: 5000
      });
      throw error;
    }
  }
}