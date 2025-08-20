"use strict";
/**
 * Git Authentication Helper
 * Handles Git credentials and authentication prompts
 * Similar to VS Code's askpass implementation
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
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
exports.GitAuthHelper = void 0;
const child_process_1 = require("child_process");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const net = __importStar(require("net"));
const crypto = __importStar(require("crypto"));
class GitAuthHelper {
    constructor() {
        this.credentialCache = new Map();
        this.CACHE_DURATION = 60000; // 60 seconds
        this.ipcServer = null;
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
    setupIpcServer() {
        // Clean up any existing socket file
        if (process.platform !== 'win32' && fs.existsSync(this.ipcSocketPath)) {
            fs.unlinkSync(this.ipcSocketPath);
        }
        this.ipcServer = net.createServer((socket) => {
            let data = '';
            socket.on('data', (chunk) => {
                data += chunk.toString();
            });
            socket.on('end', () => __awaiter(this, void 0, void 0, function* () {
                try {
                    const request = JSON.parse(data);
                    console.log('[GitAuthHelper] Received askpass request:', request.type);
                    let response = '';
                    if (request.type === 'username') {
                        response = yield this.getUsername(request.host);
                    }
                    else if (request.type === 'password') {
                        response = yield this.getPassword(request.host, request.username);
                    }
                    else if (request.type === 'ssh-passphrase') {
                        response = yield this.getSshPassphrase(request.keyPath);
                    }
                    socket.write(response);
                }
                catch (error) {
                    console.error('[GitAuthHelper] Error handling askpass request:', error);
                    socket.write('');
                }
                socket.end();
            }));
        });
        this.ipcServer.listen(this.ipcSocketPath);
        console.log('[GitAuthHelper] IPC server listening on:', this.ipcSocketPath);
    }
    /**
     * Get environment variables for Git authentication
     */
    getGitEnvironment() {
        const askpassScript = path.join(__dirname, 'askpass.js');
        return Object.assign(Object.assign({}, process.env), { GIT_ASKPASS: process.execPath, ELECTRON_RUN_AS_NODE: '1', GIT_ASKPASS_SCRIPT: askpassScript, GIT_ASKPASS_IPC: this.ipcSocketPath, GIT_TERMINAL_PROMPT: '0', SSH_ASKPASS_REQUIRE: 'force' });
    }
    /**
     * Get username for a host
     */
    getUsername(host) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check cache first
            const cached = this.credentialCache.get(host);
            if (cached && cached.username && Date.now() - cached.timestamp < this.CACHE_DURATION) {
                return cached.username;
            }
            // Try to get from system keychain/credential manager
            // For now, we'll prompt the user
            const win = electron_1.BrowserWindow.getFocusedWindow();
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
        });
    }
    /**
     * Get password for a host and username
     */
    getPassword(host, username) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check cache first
            const cached = this.credentialCache.get(host);
            if (cached && cached.password && Date.now() - cached.timestamp < this.CACHE_DURATION) {
                return cached.password;
            }
            // For now, return empty (will need proper implementation)
            return '';
        });
    }
    /**
     * Get SSH passphrase
     */
    getSshPassphrase(keyPath) {
        return __awaiter(this, void 0, void 0, function* () {
            // For now, return empty (SSH keys should ideally not have passphrases for automation)
            return '';
        });
    }
    /**
     * Execute Git command with authentication support
     */
    executeGitCommand(args, cwd) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve, reject) => {
                const env = this.getGitEnvironment();
                console.log('[GitAuthHelper] Executing git with auth support:', args.join(' '));
                const gitProcess = (0, child_process_1.spawn)('git', args, {
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
                    }
                    else {
                        reject(new Error(`Git command failed with code ${code}: ${stderr}`));
                    }
                });
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
exports.GitAuthHelper = GitAuthHelper;
//# sourceMappingURL=git-auth-helper.js.map