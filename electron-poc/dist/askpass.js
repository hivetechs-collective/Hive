#!/usr/bin/env node
/**
 * Git Askpass Script
 * This script is called by Git when it needs credentials
 * It communicates with the Electron app via IPC
 */
const net = require('net');
const path = require('path');
// Get the IPC socket path from environment
const ipcPath = process.env.GIT_ASKPASS_IPC;
if (!ipcPath) {
    console.error('GIT_ASKPASS_IPC not set');
    process.exit(1);
}
// Parse the askpass request
const args = process.argv.slice(2);
const prompt = args[0] || '';
let requestType = 'unknown';
let host = '';
let username = '';
let keyPath = '';
// Parse different types of prompts
if (prompt.includes('Username')) {
    requestType = 'username';
    // Extract host from prompt like "Username for 'https://github.com':"
    const match = prompt.match(/Username for '([^']+)':/);
    if (match) {
        host = match[1];
    }
}
else if (prompt.includes('Password')) {
    requestType = 'password';
    // Extract username and host from prompt like "Password for 'https://user@github.com':"
    const match = prompt.match(/Password for '([^']+)':/);
    if (match) {
        const url = match[1];
        const urlMatch = url.match(/https?:\/\/([^@]+)@(.+)/);
        if (urlMatch) {
            username = urlMatch[1];
            host = 'https://' + urlMatch[2];
        }
        else {
            host = url;
        }
    }
}
else if (prompt.includes('passphrase')) {
    requestType = 'ssh-passphrase';
    // Extract key path from prompt
    const match = prompt.match(/Enter passphrase for key '([^']+)':/);
    if (match) {
        keyPath = match[1];
    }
}
// Connect to IPC server and get credentials
const client = net.createConnection(ipcPath, () => {
    const request = JSON.stringify({
        type: requestType,
        prompt: prompt,
        host: host,
        username: username,
        keyPath: keyPath
    });
    client.write(request);
});
let response = '';
client.on('data', (data) => {
    response += data.toString();
});
client.on('end', () => {
    // Output the credential to stdout for Git to use
    process.stdout.write(response);
    process.exit(0);
});
client.on('error', (err) => {
    console.error('Failed to connect to askpass IPC:', err);
    process.exit(1);
});
//# sourceMappingURL=askpass.js.map