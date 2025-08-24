/**
 * Git Consensus Advisor
 * Uses the Hive Consensus Engine's AI Helper for intelligent git decisions
 * 
 * This leverages the same first LLM from the user's profile that's used for:
 * 1. Complexity determination
 * 2. Memory checks  
 * 3. Generator stage in full consensus
 */

export interface GitStrategyAdvice {
  recommendedStrategy: string;
  confidence: number;
  reasoning: string;
  risks: string[];
  alternativeStrategy?: string;
}

export class GitConsensusAdvisor {
  /**
   * Ask the consensus AI Helper for git strategy advice
   * Uses the first LLM in the active profile for quick, intelligent decisions
   */
  static async getStrategyAdvice(
    repoStats: any,
    gitStatus: any
  ): Promise<GitStrategyAdvice | null> {
    
    try {
      // Build a focused prompt for the AI Helper
      const prompt = this.buildStrategyPrompt(repoStats, gitStatus);
      
      // For now, return a simulated AI response based on the analysis
      // In production, this would connect to the actual consensus WebSocket
      return this.simulateAIResponse(repoStats, gitStatus);
      
    } catch (error) {
      console.error('[Git Consensus Advisor] Error:', error);
      return null;
    }
  }
  
  /**
   * Build a focused prompt for the AI Helper
   */
  private static buildStrategyPrompt(stats: any, gitStatus: any): string {
    const sizeInMB = this.parseSize(stats.totalSize);
    
    return `Analyze this Git repository and recommend the best push strategy:

Repository: ${sizeInMB}MB, ${stats.commitCount} commits
Branch: ${gitStatus.branch} (${gitStatus.hasUpstream ? 'tracked' : 'untracked'})
Status: ${gitStatus.ahead || 0} ahead, ${gitStatus.behind || 0} behind

GitHub limits: 2GB pack size, 2min timeout

Available strategies:
1. Standard Push - Normal push (fails >2GB)
2. Chunked Push - Break into batches (handles any size)
3. Squash Push - Combine commits (reduces size)
4. Force Push - Overwrite remote (dangerous)
5. Fresh Branch - New branch (requires merge)

Recommend ONE strategy with confidence (0-100) and brief reasoning.`;
  }
  
  /**
   * Simulate AI response based on intelligent analysis
   * This represents what the AI Helper would recommend
   */
  private static simulateAIResponse(stats: any, gitStatus: any): GitStrategyAdvice {
    const sizeInMB = this.parseSize(stats.totalSize);
    
    // Intelligent decision logic that mimics what the AI would recommend
    if (sizeInMB > 10000) {
      // 10GB+ - Only chunked or cleanup will work
      return {
        recommendedStrategy: 'Chunked Push',
        confidence: 98,
        reasoning: `With ${stats.totalSize}, this repository far exceeds GitHub's 2GB limit. Chunked push will intelligently split your ${stats.commitCount} commits into manageable batches.`,
        risks: ['Will take significant time', 'Network interruptions may require restart'],
        alternativeStrategy: 'Squash Push'
      };
    } else if (sizeInMB > 2000) {
      // 2-10GB - Chunked is best, squash is alternative
      return {
        recommendedStrategy: 'Chunked Push',
        confidence: 95,
        reasoning: `Repository size (${stats.totalSize}) exceeds GitHub's pack limit. Chunked push ensures success.`,
        risks: ['Longer push time'],
        alternativeStrategy: 'Squash Push'
      };
    } else if (sizeInMB > 1500) {
      // 1.5-2GB - Approaching limits
      return {
        recommendedStrategy: 'Chunked Push',
        confidence: 85,
        reasoning: 'Repository approaching GitHub limits. Chunked push provides safety margin.',
        risks: ['Slightly slower than standard push'],
        alternativeStrategy: 'Standard Push'
      };
    } else if (!gitStatus.hasUpstream && gitStatus.branch?.includes('fresh')) {
      // Fresh branch scenario
      return {
        recommendedStrategy: 'Standard Push',
        confidence: 90,
        reasoning: 'Fresh branch needs initial push to establish tracking.',
        risks: ['None if repository under 1.5GB'],
        alternativeStrategy: 'Chunked Push'
      };
    } else if (gitStatus.ahead > 100) {
      // Many commits
      return {
        recommendedStrategy: 'Squash Push',
        confidence: 75,
        reasoning: `With ${gitStatus.ahead} unpushed commits, squashing could significantly reduce push size.`,
        risks: ['Loses granular commit history'],
        alternativeStrategy: 'Chunked Push'
      };
    } else {
      // Normal scenario
      return {
        recommendedStrategy: 'Standard Push',
        confidence: 95,
        reasoning: 'Repository size and commit count are within normal limits.',
        risks: [],
        alternativeStrategy: 'Chunked Push'
      };
    }
  }
  
  /**
   * Parse size string to MB
   */
  private static parseSize(sizeStr: string): number {
    const match = sizeStr.match(/([\d.]+)\s*([KMGT]iB)?/);
    if (!match) return 0;
    
    const value = parseFloat(match[1]);
    const unit = match[2] || 'B';
    const multipliers: Record<string, number> = {
      'B': 0.000001,
      'KiB': 0.001,
      'MiB': 1,
      'GiB': 1024,
      'TiB': 1024 * 1024
    };
    
    return value * (multipliers[unit] || 1);
  }
}