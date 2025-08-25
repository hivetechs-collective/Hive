/**
 * Git Chunked Push for Main Process
 * Handles large repository pushes by splitting them into manageable chunks
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export class GitChunkedPushMain {
  /**
   * Try a simple incremental push strategy
   * Push commits in smaller batches to avoid the 2GB limit
   */
  static async pushInBatches(repoPath: string, batchSize: number = 50): Promise<{ success: boolean; message: string }> {
    try {
      console.log(`[ChunkedPush] Starting chunked push for repository: ${repoPath}`);
      console.log(`[ChunkedPush] Initial batch size: ${batchSize}`);
      
      // Get current branch
      const { stdout: branch } = await execAsync('git rev-parse --abbrev-ref HEAD', { cwd: repoPath });
      const currentBranch = branch.trim();
      console.log(`[ChunkedPush] Current branch: ${currentBranch}`);
      
      // Check if remote branch exists
      let remoteBranch = '';
      try {
        const { stdout } = await execAsync(`git rev-parse --abbrev-ref ${currentBranch}@{upstream}`, { cwd: repoPath });
        remoteBranch = stdout.trim();
      } catch {
        // No upstream, will push all commits
        console.log('[ChunkedPush] No upstream branch, will create it');
      }
      
      // Get unpushed commits
      let commits: string[] = [];
      if (remoteBranch) {
        const { stdout } = await execAsync(
          `git rev-list ${remoteBranch}..${currentBranch}`,
          { cwd: repoPath }
        );
        commits = stdout.trim().split('\n').filter(Boolean).reverse(); // Oldest first
      } else {
        // Get all commits if no upstream
        const { stdout } = await execAsync(
          `git rev-list ${currentBranch} --max-count=1000`,
          { cwd: repoPath }
        );
        commits = stdout.trim().split('\n').filter(Boolean).reverse();
      }
      
      if (commits.length === 0) {
        return { success: true, message: 'No commits to push' };
      }
      
      console.log(`[ChunkedPush] Found ${commits.length} commits to push`);
      
      // Push in batches
      let currentBatchSize = batchSize;
      let pushedCount = 0;
      let i = 0;
      
      while (i < commits.length) {
        const batchEnd = Math.min(i + currentBatchSize, commits.length);
        const targetCommit = commits[batchEnd - 1];
        const batchNumber = Math.floor(pushedCount / batchSize) + 1;
        const totalBatches = Math.ceil(commits.length / currentBatchSize);
        
        console.log(`[ChunkedPush] Pushing batch ${batchNumber}/${totalBatches} (commits ${i + 1}-${batchEnd}, batch size: ${currentBatchSize})`);
        
        try {
          // Try to push up to this commit
          // For new branches, we need to use the full ref path
          const pushCommand = remoteBranch 
            ? `git push origin ${targetCommit}:${currentBranch}`
            : `git push -u origin ${targetCommit}:refs/heads/${currentBranch}`;
            
          await execAsync(pushCommand, { 
            cwd: repoPath,
            timeout: 600000, // 10 minute timeout per batch for large repos
            maxBuffer: 50 * 1024 * 1024 // 50MB buffer for large output
          });
          
          pushedCount = batchEnd;
          i = batchEnd; // Move to next batch
          console.log(`[ChunkedPush] Successfully pushed ${pushedCount}/${commits.length} commits`);
          
          // Update upstream after first successful push
          if (!remoteBranch && pushedCount > 0) {
            remoteBranch = `origin/${currentBranch}`;
          }
        } catch (error: any) {
          const errorMsg = error.message || error.toString();
          
          // Check if it's still a size issue or a general push failure
          if (errorMsg.includes('pack exceeds maximum allowed size') || 
              errorMsg.includes('fatal: the remote end hung up unexpectedly') ||
              errorMsg.includes('error: failed to push')) {
            
            // Try with smaller batch
            if (currentBatchSize > 1) {
              currentBatchSize = Math.max(1, Math.floor(currentBatchSize / 2));
              console.log(`[ChunkedPush] Push failed, reducing batch size to ${currentBatchSize} and retrying`);
              // Don't increment i, retry the same position with smaller batch
              continue;
            } else {
              // Even single commits are too large
              console.log('[ChunkedPush] Even single commits exceed size limit');
              
              // Try to skip this problematic commit and continue
              if (i + 1 < commits.length) {
                console.log(`[ChunkedPush] Skipping problematic commit ${targetCommit} and continuing`);
                i++; // Skip this commit
                currentBatchSize = 1; // Reset to single commits
                continue;
              } else {
                return { 
                  success: pushedCount > 0, 
                  message: pushedCount > 0 
                    ? `Partially successful: Pushed ${pushedCount}/${commits.length} commits. Some commits are too large to push.`
                    : `Failed: Repository has commits that exceed size limits. Consider using Git LFS for large files.`
                };
              }
            }
          }
          
          // For other errors, throw them
          throw error;
        }
      }
      
      return { 
        success: true, 
        message: `Successfully pushed all ${commits.length} commits` 
      };
      
    } catch (error: any) {
      console.error('[ChunkedPush] Error:', error);
      return { 
        success: false, 
        message: `Chunked push failed: ${error.message || 'Unknown error'}` 
      };
    }
  }
  
  /**
   * Alternative: Force push with lease to specific commit
   */
  static async forcePushToCommit(repoPath: string, commitHash: string): Promise<{ success: boolean; message: string }> {
    try {
      const { stdout: branch } = await execAsync('git rev-parse --abbrev-ref HEAD', { cwd: repoPath });
      const currentBranch = branch.trim();
      
      await execAsync(
        `git push --force-with-lease origin ${commitHash}:${currentBranch}`,
        { cwd: repoPath, timeout: 120000 }
      );
      
      return { success: true, message: `Pushed up to commit ${commitHash}` };
    } catch (error: any) {
      return { success: false, message: error.message };
    }
  }
  
  /**
   * Get repository statistics for decision making
   */
  static async getRepoStats(repoPath: string, gitStatus?: any): Promise<{ 
    totalSize: string;
    largestPack: string;
    commitCount: number;
    recommendation: string;
    pushSize?: string;
    pushSizeMB?: number;
    commitsAhead?: number;
  }> {
    try {
      // Get pack size
      const { stdout: packInfo } = await execAsync(
        'git count-objects -vH',
        { cwd: repoPath }
      );
      
      let totalSize = '0';
      let largestPack = '0';
      
      const sizeMatch = packInfo.match(/size-pack: (.+)/);
      if (sizeMatch) {
        totalSize = sizeMatch[1];
      }
      
      // Get commit count
      const { stdout: commitCount } = await execAsync(
        'git rev-list --all --count',
        { cwd: repoPath }
      );
      
      const count = parseInt(commitCount.trim());
      
      // Calculate actual push size if we have git status
      let pushSize = '0';
      let pushSizeMB = 0;
      let commitsAhead = gitStatus?.ahead || 0;
      
      if (gitStatus && gitStatus.ahead > 0) {
        try {
          // Get the size of objects that would be pushed
          // This simulates what would be sent in a push
          // First try with origin/branch, if that fails try without
          let pushObjects = '';
          try {
            const result = await execAsync(
              `git rev-list --objects origin/${gitStatus.branch}..HEAD 2>/dev/null | git cat-file --batch-check='%(objectsize) %(objectname)' | awk '{sum+=$1} END {print sum}'`,
              { cwd: repoPath, shell: true }
            );
            pushObjects = result.stdout;
          } catch (e) {
            console.log('[GitChunkedPushMain] First push size calculation failed, trying alternative method');
            // Try counting just the commits we're ahead
            const result = await execAsync(
              `git rev-list HEAD~${gitStatus.ahead}..HEAD --objects | git cat-file --batch-check='%(objectsize) %(objectname)' | awk '{sum+=$1} END {print sum}'`,
              { cwd: repoPath, shell: true }
            );
            pushObjects = result.stdout;
          }
          
          const pushBytes = parseInt(pushObjects.trim()) || 0;
          pushSizeMB = pushBytes / (1024 * 1024);
          
          // Format push size for display
          if (pushSizeMB < 1) {
            pushSize = `${(pushBytes / 1024).toFixed(2)} KiB`;
          } else if (pushSizeMB < 1024) {
            pushSize = `${pushSizeMB.toFixed(2)} MiB`;
          } else {
            pushSize = `${(pushSizeMB / 1024).toFixed(2)} GiB`;
          }
          
          console.log(`[GitChunkedPushMain] Push size calculated: ${pushSize} (${pushSizeMB.toFixed(2)} MB) for ${commitsAhead} commits`);
          console.log(`[GitChunkedPushMain] Push bytes: ${pushBytes}, Repository size: ${totalSize}`);
        } catch (e) {
          console.log('[GitChunkedPushMain] Could not calculate push size:', e);
          // Fallback: estimate based on commit count
          // Average commit size is roughly 5-10KB
          pushSizeMB = (commitsAhead * 0.01); // ~10KB per commit
          pushSize = `~${pushSizeMB.toFixed(2)} MiB (estimated)`;
        }
      }
      
      // Parse total repo size for comparison
      const sizeInMB = parseFloat(totalSize.replace(/[^\d.]/g, ''));
      const unit = totalSize.match(/[KMGT]iB/)?.[0] || 'B';
      const multiplier = unit === 'GiB' ? 1024 : unit === 'MiB' ? 1 : 0.001;
      const actualSizeMB = sizeInMB * multiplier;
      
      // Base recommendation on PUSH SIZE, not repo size
      let recommendation = '';
      if (commitsAhead === 0) {
        recommendation = 'No commits to push. Use custom command for special operations.';
      } else if (pushSizeMB > 100) {
        recommendation = `Push size: ${pushSize}. GitHub limit is 100MB. Use chunked push.`;
      } else if (pushSizeMB > 50) {
        recommendation = `Push size: ${pushSize}. Consider using chunked push for safety.`;
      } else {
        recommendation = `Push size: ${pushSize}. Standard push should work fine.`;
      }
      
      return {
        totalSize,
        largestPack,
        commitCount: count,
        recommendation,
        pushSize,
        pushSizeMB,
        commitsAhead
      };
    } catch (error: any) {
      return {
        totalSize: 'Unknown',
        largestPack: 'Unknown', 
        commitCount: 0,
        recommendation: 'Could not analyze repository',
        pushSize: 'Unknown',
        pushSizeMB: 0,
        commitsAhead: 0
      };
    }
  }
}