/**
 * Generic Credential Provider
 * Prompts the user for credentials via UI dialogs
 */

import { BrowserWindow, dialog } from 'electron';
import { CredentialProvider, CredentialRequest, Credential } from '../types';

export class GenericCredentialProvider implements CredentialProvider {
  id = 'generic';
  name = 'Generic Credential Provider';
  
  canHandle(request: CredentialRequest): boolean {
    // This provider can handle any request as a fallback
    return true;
  }
  
  async getCredentials(request: CredentialRequest): Promise<Credential | null> {
    const win = BrowserWindow.getFocusedWindow();
    
    if (!win) {
      console.error('[GenericCredentialProvider] No focused window');
      return null;
    }
    
    try {
      switch (request.type) {
        case 'username':
          return await this.promptForUsername(request, win);
          
        case 'password':
          return await this.promptForPassword(request, win);
          
        case 'token':
          return await this.promptForToken(request, win);
          
        case 'ssh-passphrase':
          return await this.promptForSshPassphrase(request, win);
          
        case 'host-verification':
          return await this.promptForHostVerification(request, win);
          
        default:
          console.error(`[GenericCredentialProvider] Unknown request type: ${request.type}`);
          return null;
      }
    } catch (error) {
      console.error('[GenericCredentialProvider] Error getting credentials:', error);
      return null;
    }
  }
  
  private async promptForUsername(
    request: CredentialRequest, 
    parentWindow: BrowserWindow
  ): Promise<Credential | null> {
    return new Promise((resolve) => {
      // Create a simple modal window for username input
      const promptWindow = new BrowserWindow({
        parent: parentWindow,
        modal: true,
        width: 400,
        height: 200,
        webPreferences: {
          nodeIntegration: true,
          contextIsolation: false,
        },
        autoHideMenuBar: true,
        resizable: false,
      });
      
      const html = `
        <!DOCTYPE html>
        <html>
        <head>
          <style>
            body { 
              font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
              padding: 20px;
              background: #1e1e1e;
              color: #cccccc;
            }
            h3 { margin-top: 0; color: #ffffff; }
            input { 
              width: 100%; 
              padding: 8px; 
              margin: 10px 0;
              background: #3c3c3c;
              border: 1px solid #555;
              color: #cccccc;
              border-radius: 4px;
            }
            input:focus {
              outline: none;
              border-color: #007acc;
            }
            .buttons {
              display: flex;
              justify-content: flex-end;
              gap: 10px;
              margin-top: 20px;
            }
            button { 
              padding: 6px 14px;
              background: #007acc;
              color: white;
              border: none;
              border-radius: 4px;
              cursor: pointer;
            }
            button:hover {
              background: #005a9e;
            }
            button.cancel {
              background: #3c3c3c;
            }
            button.cancel:hover {
              background: #555;
            }
            .info {
              color: #999;
              font-size: 13px;
              margin-bottom: 10px;
            }
          </style>
        </head>
        <body>
          <h3>Git Authentication Required</h3>
          <div class="info">Enter your username for ${request.host || 'the repository'}</div>
          <input type="text" id="username" placeholder="Username" autofocus>
          <div class="buttons">
            <button class="cancel" onclick="cancel()">Cancel</button>
            <button onclick="submit()">OK</button>
          </div>
          <script>
            const { ipcRenderer } = require('electron');
            
            function submit() {
              const username = document.getElementById('username').value;
              ipcRenderer.send('credential-response', { username });
              window.close();
            }
            
            function cancel() {
              ipcRenderer.send('credential-response', null);
              window.close();
            }
            
            document.getElementById('username').addEventListener('keypress', (e) => {
              if (e.key === 'Enter') submit();
              if (e.key === 'Escape') cancel();
            });
          </script>
        </body>
        </html>
      `;
      
      promptWindow.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(html)}`);
      
      // Handle response
      const { ipcMain } = require('electron');
      ipcMain.once('credential-response', (event: any, credential: any) => {
        promptWindow.close();
        resolve(credential ? { username: credential.username, timestamp: Date.now() } : null);
      });
      
      promptWindow.on('closed', () => {
        resolve(null);
      });
    });
  }
  
  private async promptForPassword(
    request: CredentialRequest,
    parentWindow: BrowserWindow
  ): Promise<Credential | null> {
    return new Promise((resolve) => {
      const promptWindow = new BrowserWindow({
        parent: parentWindow,
        modal: true,
        width: 400,
        height: 250,
        webPreferences: {
          nodeIntegration: true,
          contextIsolation: false,
        },
        autoHideMenuBar: true,
        resizable: false,
      });
      
      const html = `
        <!DOCTYPE html>
        <html>
        <head>
          <style>
            body { 
              font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
              padding: 20px;
              background: #1e1e1e;
              color: #cccccc;
            }
            h3 { margin-top: 0; color: #ffffff; }
            input { 
              width: 100%; 
              padding: 8px; 
              margin: 10px 0;
              background: #3c3c3c;
              border: 1px solid #555;
              color: #cccccc;
              border-radius: 4px;
            }
            input:focus {
              outline: none;
              border-color: #007acc;
            }
            .buttons {
              display: flex;
              justify-content: flex-end;
              gap: 10px;
              margin-top: 20px;
            }
            button { 
              padding: 6px 14px;
              background: #007acc;
              color: white;
              border: none;
              border-radius: 4px;
              cursor: pointer;
            }
            button:hover {
              background: #005a9e;
            }
            button.cancel {
              background: #3c3c3c;
            }
            button.cancel:hover {
              background: #555;
            }
            .info {
              color: #999;
              font-size: 13px;
              margin-bottom: 10px;
            }
            .user-info {
              color: #007acc;
              margin-bottom: 5px;
            }
          </style>
        </head>
        <body>
          <h3>Git Authentication Required</h3>
          <div class="info">Enter your password for ${request.host || 'the repository'}</div>
          ${request.username ? `<div class="user-info">Username: ${request.username}</div>` : ''}
          <input type="password" id="password" placeholder="Password / Personal Access Token" autofocus>
          <div class="buttons">
            <button class="cancel" onclick="cancel()">Cancel</button>
            <button onclick="submit()">OK</button>
          </div>
          <script>
            const { ipcRenderer } = require('electron');
            
            function submit() {
              const password = document.getElementById('password').value;
              ipcRenderer.send('credential-response', { 
                username: '${request.username || ''}',
                password 
              });
              window.close();
            }
            
            function cancel() {
              ipcRenderer.send('credential-response', null);
              window.close();
            }
            
            document.getElementById('password').addEventListener('keypress', (e) => {
              if (e.key === 'Enter') submit();
              if (e.key === 'Escape') cancel();
            });
          </script>
        </body>
        </html>
      `;
      
      promptWindow.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(html)}`);
      
      // Handle response
      const { ipcMain } = require('electron');
      ipcMain.once('credential-response', (event: any, credential: any) => {
        promptWindow.close();
        resolve(credential ? { 
          username: credential.username,
          password: credential.password,
          timestamp: Date.now() 
        } : null);
      });
      
      promptWindow.on('closed', () => {
        resolve(null);
      });
    });
  }
  
  private async promptForToken(
    request: CredentialRequest,
    parentWindow: BrowserWindow
  ): Promise<Credential | null> {
    // Similar to password prompt but for tokens
    return this.promptForPassword(request, parentWindow);
  }
  
  private async promptForSshPassphrase(
    request: CredentialRequest,
    parentWindow: BrowserWindow
  ): Promise<Credential | null> {
    return new Promise((resolve) => {
      const promptWindow = new BrowserWindow({
        parent: parentWindow,
        modal: true,
        width: 400,
        height: 200,
        webPreferences: {
          nodeIntegration: true,
          contextIsolation: false,
        },
        autoHideMenuBar: true,
        resizable: false,
      });
      
      const keyName = request.keyPath ? path.basename(request.keyPath) : 'SSH key';
      
      const html = `
        <!DOCTYPE html>
        <html>
        <head>
          <style>
            body { 
              font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
              padding: 20px;
              background: #1e1e1e;
              color: #cccccc;
            }
            h3 { margin-top: 0; color: #ffffff; }
            input { 
              width: 100%; 
              padding: 8px; 
              margin: 10px 0;
              background: #3c3c3c;
              border: 1px solid #555;
              color: #cccccc;
              border-radius: 4px;
            }
            input:focus {
              outline: none;
              border-color: #007acc;
            }
            .buttons {
              display: flex;
              justify-content: flex-end;
              gap: 10px;
              margin-top: 20px;
            }
            button { 
              padding: 6px 14px;
              background: #007acc;
              color: white;
              border: none;
              border-radius: 4px;
              cursor: pointer;
            }
            button:hover {
              background: #005a9e;
            }
            button.cancel {
              background: #3c3c3c;
            }
            button.cancel:hover {
              background: #555;
            }
            .info {
              color: #999;
              font-size: 13px;
              margin-bottom: 10px;
            }
          </style>
        </head>
        <body>
          <h3>SSH Key Passphrase Required</h3>
          <div class="info">Enter passphrase for ${keyName}</div>
          <input type="password" id="passphrase" placeholder="Passphrase" autofocus>
          <div class="buttons">
            <button class="cancel" onclick="cancel()">Cancel</button>
            <button onclick="submit()">OK</button>
          </div>
          <script>
            const { ipcRenderer } = require('electron');
            
            function submit() {
              const passphrase = document.getElementById('passphrase').value;
              ipcRenderer.send('credential-response', { password: passphrase });
              window.close();
            }
            
            function cancel() {
              ipcRenderer.send('credential-response', null);
              window.close();
            }
            
            document.getElementById('passphrase').addEventListener('keypress', (e) => {
              if (e.key === 'Enter') submit();
              if (e.key === 'Escape') cancel();
            });
          </script>
        </body>
        </html>
      `;
      
      promptWindow.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(html)}`);
      
      // Handle response
      const { ipcMain } = require('electron');
      ipcMain.once('credential-response', (event: any, credential: any) => {
        promptWindow.close();
        resolve(credential ? { 
          password: credential.password,
          timestamp: Date.now() 
        } : null);
      });
      
      promptWindow.on('closed', () => {
        resolve(null);
      });
    });
  }
  
  private async promptForHostVerification(
    request: CredentialRequest,
    parentWindow: BrowserWindow
  ): Promise<Credential | null> {
    const result = await dialog.showMessageBox(parentWindow, {
      type: 'question',
      title: 'SSH Host Verification',
      message: `The authenticity of host '${request.host}' can't be established.`,
      detail: `Fingerprint: ${request.fingerprint}\n\nAre you sure you want to continue connecting?`,
      buttons: ['Yes', 'No'],
      defaultId: 1,
      cancelId: 1,
    });
    
    return result.response === 0 ? { timestamp: Date.now() } : null;
  }
}

const path = require('path');