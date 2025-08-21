/**
 * CLI Tool Detection Utility
 * Safely detects installed AI CLI tools without modifying system
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export interface CliToolStatus {
  id: string;
  name: string;
  installed: boolean;
  version?: string;
  path?: string;
  memoryServiceConnected?: boolean;
}

/**
 * Detect if Claude Code CLI is installed
 */
export async function detectClaudeCode(): Promise<CliToolStatus> {
  const status: CliToolStatus = {
    id: 'claude-code',
    name: 'Claude Code',
    installed: false
  };

  try {
    // Try to get version - this is the safest detection method
    const { stdout } = await execAsync('claude --version 2>/dev/null');
    
    if (stdout) {
      status.installed = true;
      console.log('[CLI Detector] Claude Code version output:', stdout.trim());
      // Extract version from output (format: "1.0.86 (Claude Code)")
      const versionMatch = stdout.match(/(\d+\.\d+\.\d+)/);
      if (versionMatch) {
        status.version = versionMatch[1];
        console.log('[CLI Detector] Detected version:', status.version);
      } else {
        console.log('[CLI Detector] Could not parse version from:', stdout);
      }
    }
  } catch (error) {
    // Command not found or other error - tool not installed
    console.log('[CLI Detector] Claude Code not found:', error);
  }

  // Try to get the executable path
  if (status.installed) {
    try {
      const { stdout: pathOutput } = await execAsync('which claude 2>/dev/null');
      if (pathOutput) {
        status.path = pathOutput.trim();
      }
    } catch {
      // Path detection failed, but tool might still be installed
    }
  }

  // Check if Memory Service connection is configured
  if (status.installed) {
    status.memoryServiceConnected = await checkMemoryServiceConfig('claude-code');
  }

  return status;
}

/**
 * Check if a CLI tool has Memory Service configured
 */
async function checkMemoryServiceConfig(toolId: string): Promise<boolean> {
  // For now, just check if Memory Service is running
  // Later we can check actual tool configuration
  try {
    const response = await fetch('http://localhost:3457/health');
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Detect all supported CLI tools
 */
export async function detectAllCliTools(): Promise<CliToolStatus[]> {
  const tools: CliToolStatus[] = [];
  
  // Start with Claude Code only
  tools.push(await detectClaudeCode());
  
  // TODO: Add other tools incrementally
  // tools.push(await detectGeminiCli());
  // tools.push(await detectQwenCode());
  // etc.
  
  return tools;
}

/**
 * Cache detection results to avoid repeated checks
 */
const detectionCache = new Map<string, { status: CliToolStatus; timestamp: number }>();
const CACHE_TTL = 30000; // 30 seconds

export async function getCachedToolStatus(toolId: string): Promise<CliToolStatus | null> {
  const cached = detectionCache.get(toolId);
  if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
    return cached.status;
  }
  
  // Cache miss or expired
  let status: CliToolStatus | null = null;
  
  if (toolId === 'claude-code') {
    status = await detectClaudeCode();
    detectionCache.set(toolId, { status, timestamp: Date.now() });
  }
  
  return status;
}