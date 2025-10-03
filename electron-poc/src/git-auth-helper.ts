/**
 * Git Authentication Helper
 * Handles Git credentials and authentication prompts
 * Similar to VS Code's askpass implementation
 */

import { spawn } from 'child_process';
import { dialog, BrowserWindow } from 'electron';
import * as path from 'path';
import * as fs from 'fs';
import * as net from 'net';
import * as crypto from 'crypto';

interface Credential {
  username?: string;
  password?: string;
  timestamp: number;
}

export class GitAuthHelper {
  private credentialCache: Map<string, Credential> = new Map();
  private readonly CACHE_DURATION = 60000; // 60 seconds
  private ipcServer: net.Server | null = null;
  private ipcSocketPath: string;
  
  constructor() {
    // Create unique socket path for IPC
    const socketId = crypto.randomBytes(8).toString('hex');
    this.ipcSocketPath = process.platform === 'win32' 
      ? `\\\\.\\pipe\\git-askpass-${socketId}`
      : `/tmp/git-askpass-${socketId}.sock`;
      
    this.setupIpcServer();
  }
  
  /**
   * Set up IPC server to handle askpass requests
   */
  private setupIpcServer() {
    // Clean up any existing socket file
    if (process.platform !== 'win32' && fs.existsSync(this.ipcSocketPath)) {
      fs.unlinkSync(this.ipcSocketPath);
    }
    
    this.ipcServer = net.createServer((socket) => {
      let data = '';
      
      socket.on('data', (chunk) => {
        data += chunk.toString();
      });
      
      socket.on('end', async () => {
        try {
          const request = JSON.parse(data);
          console.log('[GitAuthHelper] Received askpass request:', request.type);
          
          let response = '';
          
          if (request.type === 'username') {
            response = await this.getUsername(request.host);
          } else if (request.type === 'password') {
            response = await this.getPassword(request.host, request.username);
          } else if (request.type === 'ssh-passphrase') {
            response = await this.getSshPassphrase(request.keyPath);
          }
          
          socket.write(response);
        } catch (error) {
          console.error('[GitAuthHelper] Error handling askpass request:', error);
          socket.write('');
        }
        
        socket.end();
      });
    });
    
    this.ipcServer.listen(this.ipcSocketPath);
    console.log('[GitAuthHelper] IPC server listening on:', this.ipcSocketPath);
  }
  
  /**
   * Get environment variables for Git authentication
   */
  getGitEnvironment(): NodeJS.ProcessEnv {
    const askpassScript = path.join(__dirname, 'askpass.js');
    
    return {
      ...process.env,
      GIT_ASKPASS: process.execPath, // Use Node.js to run the script
      ELECTRON_RUN_AS_NODE: '1',
      GIT_ASKPASS_SCRIPT: askpassScript,
      GIT_ASKPASS_IPC: this.ipcSocketPath,
      GIT_TERMINAL_PROMPT: '0', // Disable terminal prompts
      SSH_ASKPASS_REQUIRE: 'force',
      // Disable system/global credential helpers (e.g., osxkeychain) for this process
      // Equivalent to: git -c credential.helper=
      GIT_CONFIG_COUNT: '1',
      GIT_CONFIG_KEY_0: 'credential.helper',
      GIT_CONFIG_VALUE_0: '',
    };
  }
  
  /**
   * Get username for a host
   */
  private async getUsername(host: string): Promise<string> {
    // Check cache first
    const cached = this.credentialCache.get(host);
    if (cached && cached.username && Date.now() - cached.timestamp < this.CACHE_DURATION) {
      return cached.username;
    }
    
    // Try to get from system keychain/credential manager
    // For now, we'll prompt the user
    const win = BrowserWindow.getFocusedWindow();
    
    return new Promise((resolve) => {
      if (!win) {
        resolve('');
        return;
      }
      
      // Create a simple prompt dialog
      // In a real implementation, you'd want a proper UI
      const html = `
        <!DOCTYPE html>
        <html>
        <head>
          <style>
            body { font-family: system-ui; padding: 20px; }
            input { width: 100%; padding: 8px; margin: 10px 0; }
            button { padding: 8px 16px; margin: 5px; }
          </style>
        </head>
        <body>
          <h3>Git Authentication Required</h3>
          <p>Please enter your username for ${host}:</p>
          <input type="text" id="username" placeholder="Username" autofocus>
          <div>
            <button onclick="submit()">OK</button>
            <button onclick="cancel()">Cancel</button>
          </div>
          <script>
            const { ipcRenderer } = require('electron');
            function submit() {
              ipcRenderer.send('git-auth-username', document.getElementById('username').value);
              window.close();
            }
            function cancel() {
              ipcRenderer.send('git-auth-username', '');
              window.close();
            }
            document.getElementById('username').addEventListener('keypress', (e) => {
              if (e.key === 'Enter') submit();
            });
          </script>
        </body>
        </html>
      `;
      
      // For now, use a simple prompt
      // In production, you'd create a proper dialog window
      resolve('git'); // Default to 'git' for now
    });
  }
  
  /**
   * Get password for a host and username
   */
  private async getPassword(host: string, username: string): Promise<string> {
    // Check cache first
    const cached = this.credentialCache.get(host);
    if (cached && cached.password && Date.now() - cached.timestamp < this.CACHE_DURATION) {
      return cached.password;
    }
    
    // For now, return empty (will need proper implementation)
    return '';
  }
  
  /**
   * Get SSH passphrase
   */
  private async getSshPassphrase(keyPath: string): Promise<string> {
    // For now, return empty (SSH keys should ideally not have passphrases for automation)
    return '';
  }
  
  /**
   * Execute Git command with authentication support
   */
  async executeGitCommand(args: string[], cwd: string): Promise<{ stdout: string; stderr: string }> {
    return new Promise((resolve, reject) => {
      const env = this.getGitEnvironment();
      
      console.log('[GitAuthHelper] Executing git with auth support:', args.join(' '));
      
      const gitProcess = spawn('git', args, {
        cwd,
        env,
      });
      
      let stdout = '';
      let stderr = '';
      
      gitProcess.stdout.on('data', (data) => {
        stdout += data.toString();
      });
      
      gitProcess.stderr.on('data', (data) => {
        stderr += data.toString();
        console.log('[GitAuthHelper] Git stderr:', data.toString());
      });
      
      gitProcess.on('error', (error) => {
        console.error('[GitAuthHelper] Git process error:', error);
        reject(error);
      });
      
      gitProcess.on('close', (code) => {
        if (code === 0) {
          resolve({ stdout, stderr });
        } else {
          reject(new Error(`Git command failed with code ${code}: ${stderr}`));
        }
      });
    });
  }
  
  /**
   * Clean up resources
   */
  dispose() {
    if (this.ipcServer) {
      this.ipcServer.close();
      
      // Clean up socket file on Unix
      if (process.platform !== 'win32' && fs.existsSync(this.ipcSocketPath)) {
        fs.unlinkSync(this.ipcSocketPath);
      }
    }
    
    this.credentialCache.clear();
  }
}
