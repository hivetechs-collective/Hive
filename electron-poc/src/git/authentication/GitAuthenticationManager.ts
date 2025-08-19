/**
 * Git Authentication Manager
 * Central manager for all Git authentication operations
 * Based on VS Code's proven architecture
 */

import { spawn, ChildProcess } from 'child_process';
import { ipcMain, BrowserWindow } from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import * as crypto from 'crypto';
import { 
  Credential, 
  CredentialRequest, 
  CredentialProvider, 
  GitAuthOptions, 
  GitEnvironment,
  AuthenticationResult 
} from './types';

export class GitAuthenticationManager {
  private credentialProviders: Map<string, CredentialProvider> = new Map();
  private credentialCache: Map<string, Credential> = new Map();
  private readonly options: Required<GitAuthOptions>;
  private askpassPath: string;
  private sshAskpassPath: string;
  private askpassMainPath: string;
  private pipePath: string;
  private isInitialized = false;
  private static ipcHandlersRegistered = false;
  
  constructor(options: GitAuthOptions = {}) {
    this.options = {
      enableCache: options.enableCache ?? true,
      cacheDuration: options.cacheDuration ?? 60000, // 60 seconds
      useSystemCredentialManager: options.useSystemCredentialManager ?? true,
      enableOAuth: options.enableOAuth ?? true,
    };
    
    // Set up paths
    const tempDir = os.tmpdir();
    const sessionId = crypto.randomBytes(8).toString('hex');
    
    this.pipePath = path.join(tempDir, `git-askpass-${sessionId}.pipe`);
    this.askpassPath = path.join(__dirname, 'askpass.sh');
    this.sshAskpassPath = path.join(__dirname, 'ssh-askpass.sh');
    this.askpassMainPath = path.join(__dirname, 'askpass-main.js');
  }
  
  /**
   * Initialize the authentication system
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) return;
    
    console.log('[GitAuth] Initializing authentication system...');
    
    // Create askpass scripts
    await this.createAskpassScripts();
    
    // Set up IPC handlers
    this.setupIpcHandlers();
    
    // Register default credential providers
    await this.registerDefaultProviders();
    
    this.isInitialized = true;
    console.log('[GitAuth] Authentication system initialized');
  }
  
  /**
   * Create the askpass scripts that Git will use
   */
  private async createAskpassScripts(): Promise<void> {
    // Create askpass.sh for HTTPS
    const askpassScript = `#!/bin/bash
# Git askpass script for HTTPS authentication
PIPE=$(mktemp -u)
mkfifo "$PIPE"
export ELECTRON_RUN_AS_NODE="1"
export VSCODE_GIT_ASKPASS_PIPE="$PIPE"
export VSCODE_GIT_ASKPASS_TYPE="https"
"${process.execPath}" "${this.askpassMainPath}" "$@"
cat "$PIPE"
rm "$PIPE"
`;
    
    // Create ssh-askpass.sh for SSH
    const sshAskpassScript = `#!/bin/bash
# SSH askpass script
PIPE=$(mktemp -u)
mkfifo "$PIPE"
export ELECTRON_RUN_AS_NODE="1"
export VSCODE_GIT_ASKPASS_PIPE="$PIPE"
export VSCODE_GIT_ASKPASS_TYPE="ssh"
"${process.execPath}" "${this.askpassMainPath}" "$@"
cat "$PIPE"
rm "$PIPE"
`;
    
    // Write scripts
    await fs.promises.writeFile(this.askpassPath, askpassScript, { mode: 0o755 });
    await fs.promises.writeFile(this.sshAskpassPath, sshAskpassScript, { mode: 0o755 });
    
    // Create askpass-main.js
    await this.createAskpassMain();
  }
  
  /**
   * Create the main askpass handler script
   */
  private async createAskpassMain(): Promise<void> {
    const askpassMainScript = `
const fs = require('fs');
const { ipcRenderer } = require('electron');

// Parse command line arguments
const args = process.argv.slice(2);
const prompt = args[0] || '';

// Get pipe path from environment
const pipePath = process.env.VSCODE_GIT_ASKPASS_PIPE;
const askpassType = process.env.VSCODE_GIT_ASKPASS_TYPE;

if (!pipePath) {
  console.error('No pipe path specified');
  process.exit(1);
}

// Parse the prompt to determine what's being requested
let request = {
  type: 'unknown',
  prompt: prompt,
  askpassType: askpassType
};

if (prompt.includes('Username')) {
  request.type = 'username';
  const match = prompt.match(/Username for '([^']+)':/);
  if (match) request.host = match[1];
} else if (prompt.includes('Password')) {
  request.type = 'password';
  const match = prompt.match(/Password for '([^']+)':/);
  if (match) {
    const url = match[1];
    const urlMatch = url.match(/https?:\\/\\/([^@]+)@(.+)/);
    if (urlMatch) {
      request.username = urlMatch[1];
      request.host = 'https://' + urlMatch[2];
    } else {
      request.host = url;
    }
  }
} else if (prompt.includes('passphrase')) {
  request.type = 'ssh-passphrase';
  const match = prompt.match(/Enter passphrase for key '([^']+)':/);
  if (match) request.keyPath = match[1];
}

// Send request to main process and wait for response
process.parentPort.once('message', (response) => {
  // Write response to pipe
  fs.writeFileSync(pipePath, response || '');
  process.exit(0);
});

// Send request
process.parentPort.postMessage(request);
`;
    
    await fs.promises.writeFile(this.askpassMainPath, askpassMainScript);
  }
  
  /**
   * Set up IPC handlers for credential requests
   */
  private setupIpcHandlers(): void {
    // Only register handlers once globally
    if (GitAuthenticationManager.ipcHandlersRegistered) {
      console.log('[GitAuth] IPC handlers already registered, skipping...');
      return;
    }
    
    // Handle askpass requests from Git processes
    ipcMain.handle('git-auth-request', async (event, request: CredentialRequest) => {
      console.log('[GitAuth] Received credential request:', request.type);
      return await this.handleCredentialRequest(request);
    });
    
    // Handle credential storage requests
    ipcMain.handle('git-auth-store', async (event, host: string, credential: Credential) => {
      this.storeCredential(host, credential);
      return { success: true };
    });
    
    // Handle credential removal requests
    ipcMain.handle('git-auth-remove', async (event, host: string) => {
      this.removeCredential(host);
      return { success: true };
    });
    
    GitAuthenticationManager.ipcHandlersRegistered = true;
  }
  
  /**
   * Register default credential providers
   */
  private async registerDefaultProviders(): Promise<void> {
    // System credential manager provider
    if (this.options.useSystemCredentialManager) {
      const { SystemCredentialProvider } = await import('./providers/SystemCredentialProvider');
      this.registerProvider(new SystemCredentialProvider());
    }
    
    // GitHub OAuth provider
    if (this.options.enableOAuth) {
      const { GitHubOAuthProvider } = await import('./providers/GitHubOAuthProvider');
      this.registerProvider(new GitHubOAuthProvider());
    }
    
    // Generic credential provider (prompts user)
    const { GenericCredentialProvider } = await import('./providers/GenericCredentialProvider');
    this.registerProvider(new GenericCredentialProvider());
  }
  
  /**
   * Register a credential provider
   */
  registerProvider(provider: CredentialProvider): void {
    console.log(`[GitAuth] Registering credential provider: ${provider.name}`);
    this.credentialProviders.set(provider.id, provider);
  }
  
  /**
   * Handle a credential request
   */
  private async handleCredentialRequest(request: CredentialRequest): Promise<AuthenticationResult> {
    // Check cache first
    if (this.options.enableCache && request.host) {
      const cached = this.getCredentialFromCache(request.host);
      if (cached) {
        console.log('[GitAuth] Using cached credential');
        return { success: true, credential: cached };
      }
    }
    
    // Try each provider
    for (const provider of this.credentialProviders.values()) {
      if (provider.canHandle(request)) {
        console.log(`[GitAuth] Trying provider: ${provider.name}`);
        
        try {
          const credential = await provider.getCredentials(request);
          if (credential) {
            // Cache the credential
            if (this.options.enableCache && request.host) {
              this.storeCredential(request.host, credential);
            }
            
            return { success: true, credential };
          }
        } catch (error) {
          console.error(`[GitAuth] Provider ${provider.name} failed:`, error);
        }
      }
    }
    
    return { 
      success: false, 
      error: 'No credential provider could handle this request' 
    };
  }
  
  /**
   * Get credential from cache
   */
  private getCredentialFromCache(host: string): Credential | null {
    const cached = this.credentialCache.get(host);
    if (cached && Date.now() - cached.timestamp < this.options.cacheDuration) {
      return cached;
    }
    
    // Remove expired credential
    if (cached) {
      this.credentialCache.delete(host);
    }
    
    return null;
  }
  
  /**
   * Store credential in cache
   */
  private storeCredential(host: string, credential: Credential): void {
    credential.timestamp = Date.now();
    this.credentialCache.set(host, credential);
  }
  
  /**
   * Remove credential from cache
   */
  private removeCredential(host: string): void {
    this.credentialCache.delete(host);
  }
  
  /**
   * Get environment variables for Git operations
   */
  getGitEnvironment(): GitEnvironment {
    return {
      GIT_ASKPASS: this.askpassPath,
      SSH_ASKPASS: this.sshAskpassPath,
      GIT_TERMINAL_PROMPT: '0', // Disable terminal prompts
      SSH_ASKPASS_REQUIRE: 'force',
      ELECTRON_RUN_AS_NODE: '1',
      VSCODE_GIT_ASKPASS_NODE: process.execPath,
      VSCODE_GIT_ASKPASS_MAIN: this.askpassMainPath,
      VSCODE_GIT_ASKPASS_PIPE: this.pipePath,
      ...process.env,
    };
  }
  
  /**
   * Execute a Git command with authentication support
   */
  async executeGitCommand(
    args: string[], 
    cwd: string
  ): Promise<{ stdout: string; stderr: string; code: number }> {
    return new Promise((resolve, reject) => {
      const env = this.getGitEnvironment();
      
      console.log('[GitAuth] Executing git command with auth:', args.join(' '));
      
      const gitProcess = spawn('git', args, {
        cwd,
        env,
        windowsHide: true,
      });
      
      let stdout = '';
      let stderr = '';
      
      gitProcess.stdout.on('data', (data) => {
        stdout += data.toString();
      });
      
      gitProcess.stderr.on('data', (data) => {
        stderr += data.toString();
      });
      
      gitProcess.on('error', (error) => {
        console.error('[GitAuth] Git process error:', error);
        reject(error);
      });
      
      gitProcess.on('close', (code) => {
        resolve({ stdout, stderr, code: code || 0 });
      });
    });
  }
  
  /**
   * Clean up resources
   */
  dispose(): void {
    // Clean up cache
    this.credentialCache.clear();
    
    // Clean up temp files
    try {
      if (fs.existsSync(this.pipePath)) {
        fs.unlinkSync(this.pipePath);
      }
      if (fs.existsSync(this.askpassPath)) {
        fs.unlinkSync(this.askpassPath);
      }
      if (fs.existsSync(this.sshAskpassPath)) {
        fs.unlinkSync(this.sshAskpassPath);
      }
      if (fs.existsSync(this.askpassMainPath)) {
        fs.unlinkSync(this.askpassMainPath);
      }
    } catch (error) {
      console.error('[GitAuth] Error cleaning up:', error);
    }
  }
}