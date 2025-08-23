/**
 * CLI Tools Detector
 * Main process module for detecting installed CLI tools
 * Enterprise-grade implementation with proper error handling
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { 
  CliToolInfo, 
  CliToolStatus, 
  CliToolConfig,
  CLI_TOOLS_REGISTRY 
} from '../../shared/types/cli-tools';
import { logger } from '../../utils/SafeLogger';

const execAsync = promisify(exec);

/**
 * Detector class for CLI tools
 * Handles detection of installed tools with caching
 */
export class CliToolsDetector {
  private cache: Map<string, CliToolInfo> = new Map();
  private cacheTimeout = 30000; // 30 seconds cache
  private lastCacheUpdate: Map<string, number> = new Map();
  
  /**
   * Get enhanced PATH with common installation directories
   */
  private getEnhancedPath(): string {
    const pathAdditions = [
      '/opt/homebrew/bin',     // Homebrew on Apple Silicon
      '/usr/local/bin',        // Homebrew on Intel Mac / common Unix
      '/usr/bin',              // System binaries
      '/bin',                  // Core binaries
      '/usr/sbin',             // System admin binaries
      '/sbin',                 // Core admin binaries
      path.join(process.env.HOME || '', '.local', 'bin'), // User local binaries
      path.join(process.env.HOME || '', '.cargo', 'bin'),  // Rust/Cargo binaries
      path.join(process.env.HOME || '', 'go', 'bin'),      // Go binaries
    ];
    
    const currentPath = process.env.PATH || '';
    const allPaths = [...new Set([...pathAdditions, ...currentPath.split(path.delimiter)])];
    return allPaths.join(path.delimiter);
  }
  
  /**
   * Detect a single CLI tool
   */
  async detectTool(toolId: string, forceRefresh = false): Promise<CliToolInfo | null> {
    // Check cache first
    if (!forceRefresh && this.isCacheValid(toolId)) {
      logger.info(`[CliToolsDetector] Using cached result for ${toolId}`);
      return this.cache.get(toolId) || null;
    }
    
    const config = CLI_TOOLS_REGISTRY[toolId];
    if (!config) {
      logger.warn(`[CliToolsDetector] Unknown tool ID: ${toolId}`);
      return null;
    }
    
    logger.info(`[CliToolsDetector] Detecting ${config.name}...`);
    
    const toolInfo: CliToolInfo = {
      id: toolId,
      name: config.name,
      description: config.description,
      command: config.command,
      installed: false,
      status: CliToolStatus.CHECKING,
      lastChecked: new Date()
    };
    
    try {
      // Check if command exists
      const enhancedPath = this.getEnhancedPath();
      const env = { ...process.env, PATH: enhancedPath };
      
      const { stdout: whichOutput } = await execAsync(
        `which ${config.command}`,
        { env }
      );
      
      const toolPath = whichOutput.trim();
      if (!toolPath) {
        toolInfo.status = CliToolStatus.NOT_INSTALLED;
        this.updateCache(toolId, toolInfo);
        return toolInfo;
      }
      
      toolInfo.path = toolPath;
      toolInfo.installed = true;
      toolInfo.status = CliToolStatus.INSTALLED;
      
      // Try to get version if command is provided
      if (config.versionCommand) {
        try {
          const { stdout: versionOutput } = await execAsync(
            config.versionCommand,
            { 
              env,
              timeout: 5000 // 5 second timeout for version check
            }
          );
          
          toolInfo.version = this.extractVersion(versionOutput, config);
          logger.info(`[CliToolsDetector] ${config.name} version: ${toolInfo.version}`);
        } catch (versionError) {
          logger.warn(`[CliToolsDetector] Could not get version for ${config.name}:`, versionError);
          toolInfo.version = 'unknown';
        }
      }
      
      // Check for memory service connection (for supported tools)
      if (toolId === 'claude-code' || toolId === 'gemini-cli' || toolId === 'qwen-code' || toolId === 'openai-codex' || toolId === 'cline' || toolId === 'grok') {
        toolInfo.memoryServiceConnected = await this.checkMemoryServiceConnection(toolId);
      }
      
    } catch (error) {
      logger.info(`[CliToolsDetector] ${config.name} not found in PATH`);
      toolInfo.status = CliToolStatus.NOT_INSTALLED;
    }
    
    this.updateCache(toolId, toolInfo);
    return toolInfo;
  }
  
  /**
   * Detect all registered CLI tools
   */
  async detectAllTools(forceRefresh = false): Promise<CliToolInfo[]> {
    const toolIds = Object.keys(CLI_TOOLS_REGISTRY);
    const detectionPromises = toolIds.map(id => this.detectTool(id, forceRefresh));
    const results = await Promise.all(detectionPromises);
    return results.filter(tool => tool !== null) as CliToolInfo[];
  }
  
  /**
   * Extract version from command output
   */
  private extractVersion(output: string, config: CliToolConfig): string {
    if (!output) return 'unknown';
    
    // Clean the output
    const cleanOutput = output.trim();
    
    // Special handling for Claude Code
    if (config.id === 'claude-code') {
      // Claude Code outputs just the version number followed by (Claude Code)
      const match = cleanOutput.match(/^([\d.]+)/);
      if (match) return match[1];
    }
    
    // Special handling for Gemini CLI
    if (config.id === 'gemini-cli') {
      // Gemini CLI outputs format like "gemini-cli/0.1.18" or just "0.1.18"
      const match = cleanOutput.match(/(?:gemini-cli\/)?(\d+\.\d+\.\d+)/);
      if (match) return match[1];
    }
    
    // Use provided regex if available
    if (config.versionRegex) {
      const regex = typeof config.versionRegex === 'string' 
        ? new RegExp(config.versionRegex)
        : config.versionRegex;
      const match = cleanOutput.match(regex);
      if (match && match[1]) return match[1];
    }
    
    // Generic version extraction
    const genericMatch = cleanOutput.match(/(\d+\.\d+\.\d+(?:\.\d+)?)/);
    if (genericMatch) return genericMatch[1];
    
    // If no version found, return first line of output (truncated)
    const firstLine = cleanOutput.split('\n')[0];
    return firstLine.substring(0, 50);
  }
  
  /**
   * Check if tool is connected to memory service
   */
  private async checkMemoryServiceConnection(toolId: string): Promise<boolean> {
    try {
      let token: string | undefined;
      let endpoint: string | undefined;
      
      // Grok is unique - it uses its own MCP config file
      if (toolId === 'grok') {
        const grokMcpPath = path.join(os.homedir(), '.grok', 'mcp-config.json');
        if (fs.existsSync(grokMcpPath)) {
          try {
            const grokMcp = JSON.parse(fs.readFileSync(grokMcpPath, 'utf-8'));
            const memoryServer = grokMcp.servers?.['hive-memory-service'];
            if (memoryServer?.env) {
              token = memoryServer.env.MEMORY_SERVICE_TOKEN;
              endpoint = memoryServer.env.MEMORY_SERVICE_ENDPOINT;
            }
          } catch (e) {
            logger.debug(`[CliToolsDetector] Failed to parse Grok MCP config:`, e);
          }
        }
        
        if (!token) {
          // Fallback to checking cli-tools-config.json for Grok
          const configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');
          if (fs.existsSync(configPath)) {
            const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
            token = config[toolId]?.memoryService?.token;
            endpoint = config[toolId]?.memoryService?.endpoint;
          }
        }
      } else {
        // Other tools use the standard cli-tools-config.json
        const configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');
        if (!fs.existsSync(configPath)) {
          logger.debug(`[CliToolsDetector] No config file found for memory service check`);
          return false;
        }
        
        const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
        const toolConfig = config[toolId];
        token = toolConfig?.memoryService?.token;
        endpoint = toolConfig?.memoryService?.endpoint;
      }
      
      if (!token) {
        logger.debug(`[CliToolsDetector] No memory service token found for ${toolId}`);
        return false;
      }
      
      // Check if the token is valid by querying the Memory Service
      const memoryServicePort = endpoint?.match(/:(\d+)/)?.[1] || '3457';
      
      // Use node's http module instead of fetch for compatibility
      const http = require('http');
      
      return new Promise((resolve) => {
        const options = {
          hostname: 'localhost',
          port: memoryServicePort,
          path: '/api/v1/memory/stats',
          method: 'GET',
          headers: {
            'Authorization': `Bearer ${token}`,
            'X-Client-Name': toolId
          },
          timeout: 2000
        };
        
        const req = http.request(options, (res: any) => {
          // If we get any response (even 401), the service is running
          // We just care if the service is accessible
          resolve(res.statusCode === 200 || res.statusCode === 401);
        });
        
        req.on('error', () => {
          logger.debug(`[CliToolsDetector] Memory service not accessible for ${toolId}`);
          resolve(false);
        });
        
        req.on('timeout', () => {
          req.destroy();
          resolve(false);
        });
        
        req.end();
      });
    } catch (error) {
      logger.debug(`[CliToolsDetector] Failed to check memory service connection for ${toolId}:`, error);
      return false;
    }
  }
  
  /**
   * Check if cache is valid for a tool
   */
  private isCacheValid(toolId: string): boolean {
    const lastUpdate = this.lastCacheUpdate.get(toolId);
    if (!lastUpdate) return false;
    
    const now = Date.now();
    return (now - lastUpdate) < this.cacheTimeout;
  }
  
  /**
   * Update cache for a tool
   */
  private updateCache(toolId: string, toolInfo: CliToolInfo): void {
    this.cache.set(toolId, toolInfo);
    this.lastCacheUpdate.set(toolId, Date.now());
  }
  
  /**
   * Clear cache for a specific tool or all tools
   */
  clearCache(toolId?: string): void {
    if (toolId) {
      this.cache.delete(toolId);
      this.lastCacheUpdate.delete(toolId);
    } else {
      this.cache.clear();
      this.lastCacheUpdate.clear();
    }
  }
  
  /**
   * Get cached tool info without detection
   */
  getCachedTool(toolId: string): CliToolInfo | null {
    return this.cache.get(toolId) || null;
  }
}

// Export singleton instance
export const cliToolsDetector = new CliToolsDetector();