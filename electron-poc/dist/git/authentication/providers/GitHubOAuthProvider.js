"use strict";
/**
 * GitHub OAuth Provider
 * Handles GitHub authentication via OAuth flow
 * Similar to VS Code's GitHub authentication
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
exports.GitHubOAuthProvider = void 0;
const electron_1 = require("electron");
const crypto = __importStar(require("crypto"));
const http = __importStar(require("http"));
class GitHubOAuthProvider {
    constructor() {
        this.id = 'github-oauth';
        this.name = 'GitHub OAuth';
        // GitHub OAuth App settings (you'll need to register your app)
        this.clientId = process.env.GITHUB_CLIENT_ID || 'your-client-id';
        this.clientSecret = process.env.GITHUB_CLIENT_SECRET || 'your-client-secret';
        this.scopes = ['repo', 'user:email'];
        // Cached tokens
        this.tokenCache = new Map();
    }
    canHandle(request) {
        var _a;
        // Only handle GitHub hosts
        return ((_a = request.host) === null || _a === void 0 ? void 0 : _a.includes('github.com')) === true;
    }
    getCredentials(request) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[GitHubOAuthProvider] Handling GitHub authentication');
            // Check for cached token
            const cachedToken = this.tokenCache.get('github.com');
            if (cachedToken) {
                console.log('[GitHubOAuthProvider] Using cached token');
                return {
                    username: 'oauth',
                    password: cachedToken,
                    timestamp: Date.now(),
                };
            }
            try {
                // Perform OAuth flow
                const token = yield this.performOAuthFlow();
                if (token) {
                    // Cache the token
                    this.tokenCache.set('github.com', token);
                    console.log('[GitHubOAuthProvider] OAuth successful');
                    return {
                        username: 'oauth',
                        password: token,
                        timestamp: Date.now(),
                    };
                }
            }
            catch (error) {
                console.error('[GitHubOAuthProvider] OAuth flow failed:', error);
            }
            return null;
        });
    }
    /**
     * Perform the OAuth flow
     */
    performOAuthFlow() {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve, reject) => {
                // Generate state for CSRF protection
                const state = crypto.randomBytes(16).toString('hex');
                // Create a local server to receive the callback
                const server = http.createServer();
                let authWindow = null;
                // Handle the callback
                server.on('request', (req, res) => __awaiter(this, void 0, void 0, function* () {
                    const url = new URL(req.url, `http://localhost:${server.address().port}`);
                    if (url.pathname === '/callback') {
                        const code = url.searchParams.get('code');
                        const receivedState = url.searchParams.get('state');
                        // Verify state
                        if (receivedState !== state) {
                            res.writeHead(400, { 'Content-Type': 'text/html' });
                            res.end('<h1>Error: Invalid state</h1>');
                            server.close();
                            reject(new Error('Invalid state parameter'));
                            return;
                        }
                        if (code) {
                            // Exchange code for token
                            try {
                                const token = yield this.exchangeCodeForToken(code);
                                // Send success response
                                res.writeHead(200, { 'Content-Type': 'text/html' });
                                res.end(`
                <!DOCTYPE html>
                <html>
                <head>
                  <style>
                    body {
                      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                      display: flex;
                      justify-content: center;
                      align-items: center;
                      height: 100vh;
                      margin: 0;
                      background: #0d1117;
                      color: #c9d1d9;
                    }
                    .container {
                      text-align: center;
                      padding: 40px;
                      background: #161b22;
                      border-radius: 10px;
                      box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                    }
                    h1 { color: #58a6ff; margin-bottom: 10px; }
                    p { color: #8b949e; }
                    .success-icon {
                      width: 60px;
                      height: 60px;
                      margin: 0 auto 20px;
                      background: #238636;
                      border-radius: 50%;
                      display: flex;
                      align-items: center;
                      justify-content: center;
                    }
                    .checkmark {
                      color: white;
                      font-size: 30px;
                    }
                  </style>
                </head>
                <body>
                  <div class="container">
                    <div class="success-icon">
                      <span class="checkmark">✓</span>
                    </div>
                    <h1>Authentication Successful!</h1>
                    <p>You can close this window and return to the IDE.</p>
                  </div>
                  <script>
                    setTimeout(() => window.close(), 3000);
                  </script>
                </body>
                </html>
              `);
                                // Close the auth window
                                if (authWindow && !authWindow.isDestroyed()) {
                                    authWindow.close();
                                }
                                server.close();
                                resolve(token);
                            }
                            catch (error) {
                                res.writeHead(500, { 'Content-Type': 'text/html' });
                                res.end('<h1>Error exchanging code for token</h1>');
                                server.close();
                                reject(error);
                            }
                        }
                        else {
                            res.writeHead(400, { 'Content-Type': 'text/html' });
                            res.end('<h1>Error: No authorization code received</h1>');
                            server.close();
                            reject(new Error('No authorization code received'));
                        }
                    }
                }));
                // Start the server
                server.listen(0, '127.0.0.1', () => {
                    const port = server.address().port;
                    const redirectUri = `http://localhost:${port}/callback`;
                    // Build GitHub OAuth URL
                    const authUrl = new URL('https://github.com/login/oauth/authorize');
                    authUrl.searchParams.set('client_id', this.clientId);
                    authUrl.searchParams.set('redirect_uri', redirectUri);
                    authUrl.searchParams.set('scope', this.scopes.join(' '));
                    authUrl.searchParams.set('state', state);
                    console.log('[GitHubOAuthProvider] Opening OAuth URL:', authUrl.toString());
                    // Create auth window
                    authWindow = new electron_1.BrowserWindow({
                        width: 800,
                        height: 600,
                        webPreferences: {
                            nodeIntegration: false,
                            contextIsolation: true,
                        },
                        autoHideMenuBar: true,
                    });
                    // Load the OAuth URL
                    authWindow.loadURL(authUrl.toString());
                    // Handle window closed
                    authWindow.on('closed', () => {
                        authWindow = null;
                        server.close();
                        resolve(null);
                    });
                });
                // Timeout after 5 minutes
                setTimeout(() => {
                    if (authWindow && !authWindow.isDestroyed()) {
                        authWindow.close();
                    }
                    server.close();
                    reject(new Error('OAuth flow timed out'));
                }, 5 * 60 * 1000);
            });
        });
    }
    /**
     * Exchange authorization code for access token
     */
    exchangeCodeForToken(code) {
        return __awaiter(this, void 0, void 0, function* () {
            const https = require('https');
            return new Promise((resolve, reject) => {
                const data = JSON.stringify({
                    client_id: this.clientId,
                    client_secret: this.clientSecret,
                    code: code,
                });
                const options = {
                    hostname: 'github.com',
                    path: '/login/oauth/access_token',
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Accept': 'application/json',
                        'Content-Length': data.length,
                    },
                };
                const req = https.request(options, (res) => {
                    let responseData = '';
                    res.on('data', (chunk) => {
                        responseData += chunk;
                    });
                    res.on('end', () => {
                        try {
                            const response = JSON.parse(responseData);
                            if (response.access_token) {
                                resolve(response.access_token);
                            }
                            else {
                                reject(new Error(response.error_description || 'Failed to get access token'));
                            }
                        }
                        catch (error) {
                            reject(error);
                        }
                    });
                });
                req.on('error', reject);
                req.write(data);
                req.end();
            });
        });
    }
    /**
     * Revoke the cached token
     */
    revokeToken() {
        this.tokenCache.clear();
        console.log('[GitHubOAuthProvider] Token revoked');
    }
}
exports.GitHubOAuthProvider = GitHubOAuthProvider;
//# sourceMappingURL=GitHubOAuthProvider.js.map