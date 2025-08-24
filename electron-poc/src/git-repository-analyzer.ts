/**
 * Git Repository Analyzer
 * Analyzes repository for size issues and provides optimization recommendations
 */

// Note: This module should be used from the main process only
// The renderer will call these functions through IPC

// Use browser-compatible alternatives
const fs = {
  existsSync: (path: string) => false, // Stub
  promises: {
    stat: async (path: string) => ({ size: 0 }),
    readdir: async (path: string) => [] as string[]
  }
};
const path = {
  join: (...args: string[]) => args.join('/')
};

// We'll use window.gitAPI to execute git commands instead
const execAsync = async (command: string, options?: any): Promise<{ stdout: string; stderr?: string }> => {
  // This will be called through IPC in the actual implementation
  throw new Error('Git commands must be executed through the main process');
};

export interface RepositoryAnalysis {
  totalSize: number;
  largeFiles: LargeFile[];
  recommendations: string[];
  gitObjectsSize: number;
  workingTreeSize: number;
  lfsTrackedSize: number;
  statistics: RepositoryStatistics;
}

export interface LargeFile {
  path: string;
  size: number;
  inHistory: boolean;
  inLFS: boolean;
}

export interface RepositoryStatistics {
  fileCount: number;
  commitCount: number;
  branchCount: number;
  largestPack: number;
  uncompressedSize: number;
  compressedSize: number;
}

export class GitRepositoryAnalyzer {
  private repoPath: string;
  
  constructor(repoPath: string) {
    this.repoPath = repoPath;
  }
  
  /**
   * Perform complete repository analysis
   */
  async analyze(): Promise<RepositoryAnalysis> {
    const [
      largeFiles,
      statistics,
      gitObjectsSize,
      workingTreeSize,
      lfsTrackedSize
    ] = await Promise.all([
      this.findLargeFiles(),
      this.getRepositoryStatistics(),
      this.getGitObjectsSize(),
      this.getWorkingTreeSize(),
      this.getLFSTrackedSize()
    ]);
    
    const totalSize = gitObjectsSize + workingTreeSize;
    const recommendations = this.generateRecommendations(largeFiles, statistics, totalSize);
    
    return {
      totalSize,
      largeFiles,
      recommendations,
      gitObjectsSize,
      workingTreeSize,
      lfsTrackedSize,
      statistics
    };
  }
  
  /**
   * Find large files in repository
   */
  private async findLargeFiles(threshold = 50 * 1024 * 1024): Promise<LargeFile[]> {
    const largeFiles: LargeFile[] = [];
    
    try {
      // Find large files in working tree
      const { stdout: workingFiles } = await execAsync(
        `find . -type f -size +${Math.floor(threshold / 1024)}k -not -path "./.git/*" 2>/dev/null | head -100`,
        { cwd: this.repoPath, maxBuffer: 10 * 1024 * 1024 }
      );
      
      for (const file of workingFiles.split('\n').filter(Boolean)) {
        const filePath = file.replace(/^\.\//, '');
        const stats = await fs.promises.stat(path.join(this.repoPath, filePath)).catch(() => null);
        if (stats) {
          const isLFS = await this.isLFSTracked(filePath);
          largeFiles.push({
            path: filePath,
            size: stats.size,
            inHistory: false,
            inLFS: isLFS
          });
        }
      }
      
      // Find large files in git history
      const { stdout: historyFiles } = await execAsync(
        `git rev-list --objects --all | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | awk '/^blob/ {print $3,$4}' | sort -n -r | head -20`,
        { cwd: this.repoPath, maxBuffer: 10 * 1024 * 1024 }
      );
      
      for (const line of historyFiles.split('\n').filter(Boolean)) {
        const [sizeStr, ...pathParts] = line.split(' ');
        const size = parseInt(sizeStr);
        const filePath = pathParts.join(' ');
        
        if (size > threshold && filePath) {
          const existing = largeFiles.find(f => f.path === filePath);
          if (existing) {
            existing.inHistory = true;
          } else {
            largeFiles.push({
              path: filePath,
              size,
              inHistory: true,
              inLFS: false
            });
          }
        }
      }
    } catch (error) {
      console.error('Error finding large files:', error);
    }
    
    return largeFiles.sort((a, b) => b.size - a.size).slice(0, 20);
  }
  
  /**
   * Get repository statistics
   */
  private async getRepositoryStatistics(): Promise<RepositoryStatistics> {
    const stats: RepositoryStatistics = {
      fileCount: 0,
      commitCount: 0,
      branchCount: 0,
      largestPack: 0,
      uncompressedSize: 0,
      compressedSize: 0
    };
    
    try {
      // Get file count
      const { stdout: fileCount } = await execAsync(
        'git ls-files | wc -l',
        { cwd: this.repoPath }
      );
      stats.fileCount = parseInt(fileCount.trim());
      
      // Get commit count
      const { stdout: commitCount } = await execAsync(
        'git rev-list --all --count',
        { cwd: this.repoPath }
      );
      stats.commitCount = parseInt(commitCount.trim());
      
      // Get branch count
      const { stdout: branchCount } = await execAsync(
        'git branch -a | wc -l',
        { cwd: this.repoPath }
      );
      stats.branchCount = parseInt(branchCount.trim());
      
      // Get pack file sizes
      const packDir = path.join(this.repoPath, '.git', 'objects', 'pack');
      if (fs.existsSync(packDir)) {
        const packFiles = await fs.promises.readdir(packDir);
        for (const file of packFiles) {
          if (file.endsWith('.pack')) {
            const fileStat = await fs.promises.stat(path.join(packDir, file));
            if (fileStat.size > stats.largestPack) {
              stats.largestPack = fileStat.size;
            }
            stats.compressedSize += fileStat.size;
          }
        }
      }
      
      // Get size-pack info
      const { stdout: sizePack } = await execAsync(
        'git count-objects -vH',
        { cwd: this.repoPath }
      );
      
      const sizeMatch = sizePack.match(/size-pack: (.+)/);
      if (sizeMatch) {
        stats.compressedSize = this.parseSize(sizeMatch[1]);
      }
    } catch (error) {
      console.error('Error getting repository statistics:', error);
    }
    
    return stats;
  }
  
  /**
   * Get size of .git/objects directory
   */
  private async getGitObjectsSize(): Promise<number> {
    try {
      const { stdout } = await execAsync(
        'du -sb .git/objects 2>/dev/null',
        { cwd: this.repoPath }
      );
      const size = parseInt(stdout.split('\t')[0]);
      return isNaN(size) ? 0 : size;
    } catch {
      return 0;
    }
  }
  
  /**
   * Get size of working tree (excluding .git)
   */
  private async getWorkingTreeSize(): Promise<number> {
    try {
      const { stdout } = await execAsync(
        'du -sb . --exclude=.git 2>/dev/null',
        { cwd: this.repoPath }
      );
      const size = parseInt(stdout.split('\t')[0]);
      return isNaN(size) ? 0 : size;
    } catch {
      return 0;
    }
  }
  
  /**
   * Get size of LFS tracked files
   */
  private async getLFSTrackedSize(): Promise<number> {
    try {
      const { stdout } = await execAsync(
        'git lfs ls-files --size 2>/dev/null',
        { cwd: this.repoPath }
      );
      
      let totalSize = 0;
      for (const line of stdout.split('\n').filter(Boolean)) {
        const match = line.match(/\((\d+) B?\)/);
        if (match) {
          totalSize += parseInt(match[1]);
        }
      }
      return totalSize;
    } catch {
      return 0;
    }
  }
  
  /**
   * Check if a file is tracked by Git LFS
   */
  private async isLFSTracked(filePath: string): Promise<boolean> {
    try {
      const { stdout } = await execAsync(
        `git check-attr filter "${filePath}"`,
        { cwd: this.repoPath }
      );
      return stdout.includes('lfs');
    } catch {
      return false;
    }
  }
  
  /**
   * Generate recommendations based on analysis
   */
  private generateRecommendations(
    largeFiles: LargeFile[],
    statistics: RepositoryStatistics,
    totalSize: number
  ): string[] {
    const recommendations: string[] = [];
    
    // Check for GitHub size limit
    if (statistics.largestPack > 2 * 1024 * 1024 * 1024) {
      recommendations.push('⚠️ Repository exceeds GitHub\'s 2GB pack size limit. Split commits or use Git LFS.');
    }
    
    // Check for large files not in LFS
    const largeFiesNotInLFS = largeFiles.filter(f => !f.inLFS && f.size > 100 * 1024 * 1024);
    if (largeFiesNotInLFS.length > 0) {
      recommendations.push(`📦 ${largeFiesNotInLFS.length} files over 100MB should be moved to Git LFS.`);
    }
    
    // Check for files in history that should be removed
    const largeHistoryFiles = largeFiles.filter(f => f.inHistory && !f.inLFS);
    if (largeHistoryFiles.length > 0) {
      recommendations.push(`🗑️ ${largeHistoryFiles.length} large files in history could be removed with BFG Repo-Cleaner.`);
    }
    
    // Check if repository needs garbage collection
    if (statistics.compressedSize > 500 * 1024 * 1024) {
      recommendations.push('🧹 Run `git gc --aggressive` to optimize repository size.');
    }
    
    // Check for too many branches
    if (statistics.branchCount > 100) {
      recommendations.push(`🌿 ${statistics.branchCount} branches found. Consider pruning old branches.`);
    }
    
    // General size warning
    if (totalSize > 1024 * 1024 * 1024) {
      const sizeGB = (totalSize / (1024 * 1024 * 1024)).toFixed(2);
      recommendations.push(`💾 Repository size is ${sizeGB}GB. Consider splitting into smaller repositories.`);
    }
    
    return recommendations;
  }
  
  /**
   * Parse size string (e.g., "1.5 MiB") to bytes
   */
  private parseSize(sizeStr: string): number {
    const match = sizeStr.match(/^([\d.]+)\s*([KMGT]i?B)?$/i);
    if (!match) return 0;
    
    const value = parseFloat(match[1]);
    const unit = match[2]?.toUpperCase() || 'B';
    
    const multipliers: { [key: string]: number } = {
      'B': 1,
      'KB': 1024,
      'KIB': 1024,
      'MB': 1024 * 1024,
      'MIB': 1024 * 1024,
      'GB': 1024 * 1024 * 1024,
      'GIB': 1024 * 1024 * 1024,
      'TB': 1024 * 1024 * 1024 * 1024,
      'TIB': 1024 * 1024 * 1024 * 1024
    };
    
    return value * (multipliers[unit] || 1);
  }
  
  /**
   * Format bytes to human-readable string
   */
  static formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, i);
    
    return `${value.toFixed(2)} ${units[i]}`;
  }
}