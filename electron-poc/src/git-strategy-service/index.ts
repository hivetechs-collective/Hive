/**
 * Git Strategy Service
 * Dedicated child process for AI-powered git strategy recommendations
 * 
 * KEY PRINCIPLES:
 * - Complete isolation from Consensus Engine
 * - Single source of truth: SQLite database
 * - NO FALLBACK: Real AI or nothing
 * - Dynamic port allocation via ProcessManager
 */

import express from 'express';
import Database from 'better-sqlite3';
import fetch from 'node-fetch';
import path from 'path';
import os from 'os';

interface GitStrategyRequest {
  repoStats: {
    totalSize: string;
    commitCount: number;
    hasUnpushedCommits: boolean;
    lastPushDate?: Date;
  };
  gitStatus: {
    ahead: number;
    behind: number;
    branch: string;
    hasUpstream: boolean;
  };
}

interface GitStrategyResponse {
  success: boolean;
  advice?: {
    recommendedStrategy: string;
    confidence: number;
    reasoning: string;
    risks: string[];
    alternativeStrategy?: string;
    llmUsed: string;
  };
  error?: string;
}

export class GitStrategyService {
  private app: express.Application;
  private db: Database.Database | null = null;
  private port: number;
  
  constructor() {
    this.app = express();
    this.app.use(express.json());
    
    // Get port from environment variable set by ProcessManager
    this.port = parseInt(process.env.PORT || '4567', 10);
    
    this.setupRoutes();
    this.initDatabase();
  }
  
  private initDatabase() {
    try {
      // Connect to the unified SQLite database
      const dbPath = path.join(os.homedir(), '.hive', 'hive.db');
      this.db = new Database(dbPath, { readonly: true });
      console.log('[Git Strategy Service] Connected to database');
    } catch (error) {
      console.error('[Git Strategy Service] Database connection failed:', error);
      // Service continues but will return errors
    }
  }
  
  private setupRoutes() {
    this.app.post('/analyze', async (req, res) => {
      try {
        const request: GitStrategyRequest = req.body;
        const response = await this.analyzeStrategy(request);
        res.json(response);
      } catch (error) {
        console.error('[Git Strategy Service] Analysis error:', error);
        res.json({
          success: false,
          error: 'Analysis failed'
        } as GitStrategyResponse);
      }
    });
    
    this.app.get('/health', (req, res) => {
      res.json({ 
        status: 'healthy',
        port: this.port,
        database: this.db ? 'connected' : 'disconnected'
      });
    });
  }
  
  private async analyzeStrategy(request: GitStrategyRequest): Promise<GitStrategyResponse> {
    // NO FALLBACK - if any step fails, return error
    
    if (!this.db) {
      return {
        success: false,
        error: 'Database not available'
      };
    }
    
    // Get active consensus profile from database
    const profileRow = this.db.prepare(
      "SELECT generator_model FROM consensus_profiles WHERE is_default = 1"
    ).get() as { generator_model: string } | undefined;
    
    if (!profileRow) {
      return {
        success: false,
        error: 'No active consensus profile found'
      };
    }
    
    // Use generator model for AI Helper functionality
    const firstModel = profileRow.generator_model;
    
    // Get OpenRouter API key from database (not environment)
    const apiKeyRow = this.db.prepare(
      "SELECT value FROM configuration WHERE key = 'openrouter_api_key'"
    ).get() as { value: string } | undefined;
    
    if (!apiKeyRow || !apiKeyRow.value) {
      return {
        success: false,
        error: 'OpenRouter API key not configured in database'
      };
    }
    
    const apiKey = apiKeyRow.value;
    
    // Build prompt
    const prompt = this.buildPrompt(request);
    
    // Make OpenRouter API call
    try {
      const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey}`,
          'Content-Type': 'application/json',
          'HTTP-Referer': 'https://hivetechs.io',
          'X-Title': 'Hive Git Strategy Advisor'
        },
        body: JSON.stringify({
          model: firstModel,
          messages: [
            {
              role: 'system',
              content: 'You are a Git strategy advisor. Analyze repository information and recommend the best push strategy. Respond with JSON only.'
            },
            {
              role: 'user',
              content: prompt
            }
          ],
          temperature: 0.3,
          max_tokens: 500,
          response_format: { type: 'json_object' }
        })
      });
      
      if (!response.ok) {
        const error = await response.text();
        console.error('[Git Strategy Service] OpenRouter error:', error);
        return {
          success: false,
          error: 'AI service unavailable'
        };
      }
      
      const data = await response.json() as any;
      const content = data.choices?.[0]?.message?.content;
      
      if (!content) {
        return {
          success: false,
          error: 'Invalid AI response'
        };
      }
      
      // Parse AI response
      const advice = JSON.parse(content);
      
      return {
        success: true,
        advice: {
          recommendedStrategy: advice.strategy || 'Standard Push',
          confidence: advice.confidence || 50,
          reasoning: advice.reasoning || 'Based on repository analysis',
          risks: advice.risks || [],
          alternativeStrategy: advice.alternative,
          llmUsed: firstModel
        }
      };
      
    } catch (error) {
      console.error('[Git Strategy Service] API call failed:', error);
      return {
        success: false,
        error: 'AI request failed'
      };
    }
  }
  
  private buildPrompt(request: GitStrategyRequest): string {
    const { repoStats, gitStatus } = request;
    
    return `Analyze this Git repository and recommend the best push strategy.

Repository Stats:
- Size: ${repoStats.totalSize}
- Commits: ${repoStats.commitCount}
- Has unpushed: ${repoStats.hasUnpushedCommits}
- Branch: ${gitStatus.branch}
- Ahead: ${gitStatus.ahead} commits
- Behind: ${gitStatus.behind} commits
- Has upstream: ${gitStatus.hasUpstream}

GitHub Limits:
- Max pack size: 2GB
- Timeout: 2 minutes

Available Strategies:
1. "Standard Push" - Normal git push (best for <2GB)
2. "Chunked Push" - Split into batches (handles any size)
3. "Squash Push" - Combine commits first (reduces size)
4. "Force Push" - Overwrite remote (dangerous but effective)
5. "Fresh Branch" - Create new branch (requires later merge)

Respond with JSON containing:
{
  "strategy": "<recommended strategy name>",
  "confidence": <0-100>,
  "reasoning": "<brief explanation>",
  "risks": ["<risk1>", "<risk2>"],
  "alternative": "<alternative strategy or null>"
}`;
  }
  
  start() {
    this.app.listen(this.port, () => {
      console.log(`[Git Strategy Service] Running on port ${this.port}`);
      
      // Notify parent process we're ready
      if (process.send) {
        process.send({ type: 'ready', port: this.port });
      }
    });
  }
}

// Start the service
const service = new GitStrategyService();
service.start();

// Handle graceful shutdown
process.on('SIGTERM', () => {
  console.log('[Git Strategy Service] Shutting down');
  process.exit(0);
});

process.on('SIGINT', () => {
  console.log('[Git Strategy Service] Shutting down');
  process.exit(0);
});