/**
 * Git Push Strategy Analyzer
 * Intelligently recommends the best push strategy based on repository characteristics
 */

export interface RepositoryAnalysis {
  totalSize: string;
  sizeInMB: number;
  commitCount: number;
  hasLargeFiles: boolean;
  largestFile?: { path: string; size: string };
  hasUnpushedCommits: number;
  branchStatus: 'new' | 'existing' | 'diverged';
  isMainBranch: boolean;
  recommendation: PushStrategy;
  reasoning: string[];
  risks: string[];
  pushSize?: string;
  pushSizeMB?: number;
}

export enum PushStrategy {
  REGULAR = 'regular',
  CHUNKED = 'chunked',
  FORCE = 'force',
  FRESH_BRANCH = 'fresh-branch',
  SQUASH = 'squash',
  BUNDLE = 'bundle',
  CLEANUP_FIRST = 'cleanup-first'
}

export interface PushStrategyOption {
  strategy: PushStrategy;
  label: string;
  description: string;
  icon: string;
  recommended: boolean;
  pros: string[];
  cons: string[];
  requirements?: string[];
  estimatedTime?: string;
  command?: string;
  // AI-enhanced properties
  aiEnhanced?: boolean;
  aiReasoning?: string;
  aiExplanation?: string;
  // User-selected options
  selectedOptions?: {
    forceWithLease?: boolean;
    includeTags?: boolean;
    setUpstream?: boolean;
    dryRun?: boolean;
    commitLimit?: number;
    customCommand?: string;
    atomic?: boolean;
    signPush?: boolean;
    thinPack?: boolean;
  };
}

export class GitPushStrategyAnalyzer {
  /**
   * Analyze repository and recommend best push strategy
   */
  static analyzeRepository(stats: any, gitStatus: any): RepositoryAnalysis {
    // Parse size - Use PUSH SIZE if available, otherwise fall back to total size
    const pushSizeMB = stats.pushSizeMB || 0;
    const pushSize = stats.pushSize || '0';
    
    // Also parse total repo size for context
    const sizeMatch = stats.totalSize.match(/([\d.]+)\s*([KMGT]iB)?/);
    let repoSizeInMB = 0;
    if (sizeMatch) {
      const value = parseFloat(sizeMatch[1]);
      const unit = sizeMatch[2] || 'B';
      const multipliers: Record<string, number> = {
        'B': 0.000001,
        'KiB': 0.001,
        'MiB': 1,
        'GiB': 1024,
        'TiB': 1024 * 1024
      };
      repoSizeInMB = value * (multipliers[unit] || 1);
    }
    
    // Use push size for decisions if available, otherwise use repo size
    const sizeInMB = pushSizeMB > 0 ? pushSizeMB : repoSizeInMB;

    // Determine branch status
    let branchStatus: 'new' | 'existing' | 'diverged' = 'existing';
    if (!gitStatus.hasUpstream) {
      branchStatus = 'new';
    } else if (gitStatus.behind > 0 && gitStatus.ahead > 0) {
      branchStatus = 'diverged';
    }

    // Check if main branch
    const isMainBranch = ['main', 'master', 'develop', 'development'].includes(gitStatus.branch);
    
    // Check if this is a fresh branch we already created
    const isFreshBranch = gitStatus.branch && gitStatus.branch.includes('-fresh-');

    // Determine recommendation
    let recommendation = PushStrategy.REGULAR;
    const reasoning: string[] = [];
    const risks: string[] = [];

    // Decision tree for recommendations
    
    // Special handling for fresh branches
    if (isFreshBranch) {
      if (!gitStatus.hasUpstream) {
        recommendation = PushStrategy.REGULAR;
        reasoning.push('Fresh branch created successfully');
        reasoning.push('Push to establish upstream and enable collaboration');
        reasoning.push('After pushing, you can create a PR to merge back to main');
      } else {
        recommendation = PushStrategy.REGULAR;
        reasoning.push('Fresh branch already established');
        reasoning.push('Continue pushing changes normally');
        reasoning.push('Consider creating a Pull Request to main when ready');
      }
    } else if (sizeInMB > 10000) { // > 10GB
      if (isMainBranch) {
        recommendation = PushStrategy.CLEANUP_FIRST;
        reasoning.push('Repository exceeds 10GB - cleanup is essential');
        reasoning.push('Main branch should be kept clean and efficient');
        risks.push('Pushing this much data will likely fail');
      } else {
        recommendation = PushStrategy.FRESH_BRANCH;
        reasoning.push('Repository is extremely large (>10GB)');
        reasoning.push('Fresh branch avoids history transfer issues');
        risks.push('Current branch has too much historical data');
      }
    } else if (sizeInMB > 2000) { // > 2GB
      if (stats.commitCount > 1000) {
        recommendation = PushStrategy.CHUNKED;
        reasoning.push('Large repository with many commits');
        reasoning.push('Chunked push can handle size limits incrementally');
      } else if (branchStatus === 'new') {
        recommendation = PushStrategy.SQUASH;
        reasoning.push('New branch with large changes');
        reasoning.push('Squashing reduces push size significantly');
      } else {
        recommendation = PushStrategy.CHUNKED;
        reasoning.push('Repository approaching GitHub limits');
        reasoning.push('Incremental push most likely to succeed');
      }
    } else if (sizeInMB > 1000) { // > 1GB
      if (branchStatus === 'diverged') {
        recommendation = PushStrategy.FORCE;
        reasoning.push('Branch has diverged from remote');
        reasoning.push('Force push avoids complex merge');
        risks.push('Will overwrite remote changes');
      } else {
        recommendation = PushStrategy.REGULAR;
        reasoning.push('Size is manageable for standard push');
      }
    } else {
      recommendation = PushStrategy.REGULAR;
      reasoning.push('Repository size is within normal limits');
    }

    // Add context-specific risks
    if (isMainBranch && recommendation === PushStrategy.FORCE) {
      risks.push('⚠️ Force pushing to main branch is dangerous');
      recommendation = PushStrategy.CHUNKED; // Override to safer option
    }

    return {
      totalSize: stats.totalSize,
      sizeInMB: repoSizeInMB,  // Keep repo size for reference
      commitCount: stats.commitCount,
      hasLargeFiles: sizeInMB > 100,
      hasUnpushedCommits: gitStatus.ahead || 0,
      branchStatus,
      isMainBranch,
      recommendation,
      reasoning,
      risks,
      pushSize: pushSize || stats.pushSize,
      pushSizeMB: pushSizeMB || stats.pushSizeMB || 0
    };
  }

  /**
   * Get all available push strategies with recommendations
   */
  static getPushStrategies(analysis: RepositoryAnalysis, gitStatus?: any): PushStrategyOption[] {
    const strategies: PushStrategyOption[] = [];

    // Regular Push
    strategies.push({
      strategy: PushStrategy.REGULAR,
      label: 'Standard Push',
      description: 'Normal git push to remote repository',
      icon: '📤',
      recommended: analysis.recommendation === PushStrategy.REGULAR,
      pros: [
        'Preserves all history',
        'Standard Git workflow',
        'No data loss'
      ],
      cons: [
        'May fail for large repositories',
        'Subject to 2GB pack limits',
        'Can be slow for many commits'
      ],
      estimatedTime: analysis.sizeInMB < 100 ? '< 1 minute' : 
                    analysis.sizeInMB > 2000 ? 'Will fail - exceeds 2GB limit' :
                    `~${Math.ceil(analysis.sizeInMB / 100)} minutes`,
      command: 'git push origin branch-name'
    });

    // Chunked Push - Should be TOP recommendation for large repos
    strategies.push({
      strategy: PushStrategy.CHUNKED,
      label: 'Smart Chunked Push',
      description: 'Push commits in smaller batches to avoid size limits',
      icon: '📦',
      recommended: analysis.recommendation === PushStrategy.CHUNKED,
      pros: [
        'Handles large repositories',
        'Automatic retry with smaller batches',
        'Preserves complete history',
        'Works around 2GB limits'
      ],
      cons: [
        'Takes longer than regular push',
        'May still fail for huge commits',
        'Multiple network operations'
      ],
      requirements: ['Stable internet connection'],
      estimatedTime: `${Math.ceil(analysis.commitCount / 50) * 2}-${Math.ceil(analysis.commitCount / 10) * 2} minutes`,
      command: 'git push in batches of 50, 25, 10, 5, 1'
    });

    // Force Push (with warnings)
    if (!analysis.isMainBranch) {
      strategies.push({
        strategy: PushStrategy.FORCE,
        label: 'Force Push',
        description: 'Replace remote branch entirely with local version',
        icon: '💪',
        recommended: analysis.recommendation === PushStrategy.FORCE,
        pros: [
          'Bypasses merge conflicts',
          'Simple and direct',
          'Good for feature branches'
        ],
        cons: [
          '⚠️ Destructive - loses remote history',
          'Can break other developers\' work',
          'Not suitable for shared branches'
        ],
        requirements: ['No other developers on branch', 'Backup recommended'],
        estimatedTime: `~${Math.ceil(analysis.sizeInMB / 200)} minutes`,
        command: 'git push --force origin branch-name'
      });
    }

    // Fresh Branch - only show if not already on a fresh branch
    const isFreshBranch = analysis.branchStatus === 'new' && gitStatus.branch?.includes('-fresh-');
    if (!isFreshBranch) {
      strategies.push({
        strategy: PushStrategy.FRESH_BRANCH,
        label: 'Create Fresh Branch',
        description: 'Push to a completely new branch name',
        icon: '🌱',
        recommended: analysis.recommendation === PushStrategy.FRESH_BRANCH,
        pros: [
          'Avoids all conflict issues',
          'Clean start without baggage',
          'Preserves original branch',
          'Good for experiments'
        ],
        cons: [
          'Requires manual PR/merge',
          'Branch proliferation',
          'May confuse workflow'
        ],
        estimatedTime: `~${Math.ceil(analysis.sizeInMB / 150)} minutes`,
        command: 'git push origin HEAD:feature-fresh-[date]'
      });
    } else {
      // If on fresh branch, add helpful guidance - but respect size limits!
      strategies.push({
        strategy: PushStrategy.REGULAR,
        label: 'Push Fresh Branch to Remote',
        description: 'Establish upstream for your fresh branch to enable collaboration',
        icon: '🚀',
        recommended: !gitStatus?.hasUpstream && analysis.sizeInMB < 1500 && 
                     analysis.recommendation !== PushStrategy.CHUNKED,
        pros: [
          'Establishes remote tracking',
          'Enables pull request creation',
          'Allows team collaboration',
          'Preserves your fresh start'
        ],
        cons: analysis.sizeInMB > 1500 ? ['May fail due to repository size'] : [],
        estimatedTime: analysis.sizeInMB > 2000 ? 'Will likely fail - use chunked push' :
                      `~${Math.ceil(analysis.sizeInMB / 150)} minutes`,
        command: `git push -u origin ${gitStatus?.branch || 'branch-name'}`
      });
    }

    // Squash (for new branches or many commits)
    if (analysis.branchStatus === 'new' || analysis.hasUnpushedCommits > 10) {
      strategies.push({
        strategy: PushStrategy.SQUASH,
        label: 'Squash & Push',
        description: 'Combine all commits into one and push',
        icon: '🎯',
        recommended: analysis.recommendation === PushStrategy.SQUASH,
        pros: [
          'Dramatically reduces push size',
          'Clean commit history',
          'Bypasses per-commit limits'
        ],
        cons: [
          'Loses granular history',
          'Cannot revert individual changes',
          'Requires careful commit message'
        ],
        requirements: ['Good summary commit message'],
        estimatedTime: '< 2 minutes',
        command: 'git reset --soft origin/main && git commit && git push'
      });
    }

    // Bundle (last resort)
    if (analysis.sizeInMB > 5000) {
      strategies.push({
        strategy: PushStrategy.BUNDLE,
        label: 'Create Bundle File',
        description: 'Export repository as file for manual upload',
        icon: '📁',
        recommended: analysis.recommendation === PushStrategy.BUNDLE,
        pros: [
          'Bypasses Git protocol entirely',
          'Works for any size',
          'Can be shared via other means'
        ],
        cons: [
          'Manual process',
          'Requires web upload',
          'Not integrated with PR workflow'
        ],
        requirements: ['GitHub web access', 'File upload capability'],
        estimatedTime: 'Varies based on upload speed',
        command: 'git bundle create repo.bundle --all'
      });
    }

    // Cleanup First (for huge repos)
    if (analysis.sizeInMB > 5000) {
      strategies.push({
        strategy: PushStrategy.CLEANUP_FIRST,
        label: 'Clean History First',
        description: 'Remove large files from history before pushing',
        icon: '🧹',
        recommended: analysis.recommendation === PushStrategy.CLEANUP_FIRST,
        pros: [
          'Permanent size reduction',
          'Improves repo performance',
          'Best long-term solution'
        ],
        cons: [
          'Rewrites history',
          'Requires coordination',
          'Time consuming'
        ],
        requirements: ['BFG Repo Cleaner', 'Backup essential'],
        estimatedTime: '30-60 minutes',
        command: 'bfg --strip-blobs-bigger-than 100M'
      });
    }

    // Don't sort - keep strategies in logical order
    // The recommended strategy is already marked based on analysis

    return strategies;
  }

  /**
   * Get detailed explanation for recommendation
   */
  static getRecommendationExplanation(analysis: RepositoryAnalysis): string {
    // Check if on a fresh branch
    const branch = (analysis as any).branch;
    if (branch && branch.includes('-fresh-')) {
      if (!(analysis as any).hasUpstream) {
        return '✅ Fresh branch created successfully! Now push it to establish remote tracking. After pushing, you can create a Pull Request to merge your changes back to main.';
      } else {
        return '✅ Your fresh branch is established on remote. Continue working normally or create a Pull Request when ready to merge to main.';
      }
    }
    
    const explanations: Record<PushStrategy, string> = {
      [PushStrategy.REGULAR]: 'Your repository is within normal size limits. A standard push should work without issues.',
      [PushStrategy.CHUNKED]: 'Your repository is large but manageable. Chunked push will break it into smaller pieces to avoid size limits.',
      [PushStrategy.FORCE]: 'Your branch has diverged from remote. Force push will replace the remote branch entirely with your local version.',
      [PushStrategy.FRESH_BRANCH]: 'Your repository is extremely large. Creating a fresh branch avoids transferring the entire history.',
      [PushStrategy.SQUASH]: 'You have many unpushed commits. Squashing them into one will significantly reduce the push size.',
      [PushStrategy.BUNDLE]: 'Your repository exceeds normal limits. Creating a bundle file allows manual upload bypassing Git protocols.',
      [PushStrategy.CLEANUP_FIRST]: 'Your repository is too large to push effectively. Cleaning the history first is essential for long-term health.'
    };

    return explanations[analysis.recommendation] || 'Analyzing your repository to determine the best approach...';
  }
}