"use strict";
/**
 * Generic Credential Provider
 * Prompts the user for credentials via UI dialogs
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericCredentialProvider = void 0;
const electron_1 = require("electron");
class GenericCredentialProvider {
    constructor() {
        this.id = 'generic';
        this.name = 'Generic Credential Provider';
    }
    canHandle(request) {
        // This provider can handle any request as a fallback
        return true;
    }
    getCredentials(request) {
        return __awaiter(this, void 0, void 0, function* () {
            const win = electron_1.BrowserWindow.getFocusedWindow();
            if (!win) {
                console.error('[GenericCredentialProvider] No focused window');
                return null;
            }
            try {
                switch (request.type) {
                    case 'username':
                        return yield this.promptForUsername(request, win);
                    case 'password':
                        return yield this.promptForPassword(request, win);
                    case 'token':
                        return yield this.promptForToken(request, win);
                    case 'ssh-passphrase':
                        return yield this.promptForSshPassphrase(request, win);
                    case 'host-verification':
                        return yield this.promptForHostVerification(request, win);
                    default:
                        console.error(`[GenericCredentialProvider] Unknown request type: ${request.type}`);
                        return null;
                }
            }
            catch (error) {
                console.error('[GenericCredentialProvider] Error getting credentials:', error);
                return null;
            }
        });
    }
    promptForUsername(request, parentWindow) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve) => {
                // Create a simple modal window for username input
                const promptWindow = new electron_1.BrowserWindow({
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
                ipcMain.once('credential-response', (event, credential) => {
                    promptWindow.close();
                    resolve(credential ? { username: credential.username, timestamp: Date.now() } : null);
                });
                promptWindow.on('closed', () => {
                    resolve(null);
                });
            });
        });
    }
    promptForPassword(request, parentWindow) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve) => {
                const promptWindow = new electron_1.BrowserWindow({
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
                ipcMain.once('credential-response', (event, credential) => {
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
        });
    }
    promptForToken(request, parentWindow) {
        return __awaiter(this, void 0, void 0, function* () {
            // Similar to password prompt but for tokens
            return this.promptForPassword(request, parentWindow);
        });
    }
    promptForSshPassphrase(request, parentWindow) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve) => {
                const promptWindow = new electron_1.BrowserWindow({
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
                ipcMain.once('credential-response', (event, credential) => {
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
        });
    }
    promptForHostVerification(request, parentWindow) {
        return __awaiter(this, void 0, void 0, function* () {
            const result = yield electron_1.dialog.showMessageBox(parentWindow, {
                type: 'question',
                title: 'SSH Host Verification',
                message: `The authenticity of host '${request.host}' can't be established.`,
                detail: `Fingerprint: ${request.fingerprint}\n\nAre you sure you want to continue connecting?`,
                buttons: ['Yes', 'No'],
                defaultId: 1,
                cancelId: 1,
            });
            return result.response === 0 ? { timestamp: Date.now() } : null;
        });
    }
}
exports.GenericCredentialProvider = GenericCredentialProvider;
const path = require('path');
//# sourceMappingURL=GenericCredentialProvider.js.map