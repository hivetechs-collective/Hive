#!/usr/bin/env node

/**
 * Cline CLI Wrapper - Fixes VS Code API compatibility issues
 * This wrapper provides shims for VS Code APIs that cline-cli expects
 * allowing it to run properly in terminal environments like TTYD
 */

// Create VS Code API shims before loading cline-cli
global.vscode = {
  window: {
    outputChannel: {
      appendLine: (text) => console.log(text),
      append: (text) => process.stdout.write(text),
      clear: () => console.clear(),
      show: () => {},
      hide: () => {},
      dispose: () => {}
    },
    showErrorMessage: (message) => {
      console.error(`❌ Error: ${message}`);
      return Promise.resolve();
    },
    showWarningMessage: (message) => {
      console.warn(`⚠️ Warning: ${message}`);
      return Promise.resolve();
    },
    showInformationMessage: (message) => {
      console.log(`ℹ️ ${message}`);
      return Promise.resolve();
    },
    createOutputChannel: (name) => {
      return {
        appendLine: (text) => console.log(`[${name}] ${text}`),
        append: (text) => process.stdout.write(`[${name}] ${text}`),
        clear: () => console.clear(),
        show: () => {},
        hide: () => {},
        dispose: () => {}
      };
    },
    activeTextEditor: undefined,
    visibleTextEditors: []
  },
  workspace: {
    workspaceFolders: [{
      uri: {
        fsPath: process.cwd(),
        scheme: 'file',
        path: process.cwd()
      },
      name: 'workspace',
      index: 0
    }],
    getConfiguration: (section) => {
      return {
        get: (key, defaultValue) => defaultValue,
        has: (key) => false,
        inspect: (key) => undefined,
        update: () => Promise.resolve()
      };
    },
    openTextDocument: () => Promise.resolve({
      getText: () => '',
      fileName: '',
      isDirty: false,
      save: () => Promise.resolve(true)
    }),
    fs: {
      readFile: require('fs').promises.readFile,
      writeFile: require('fs').promises.writeFile,
      stat: require('fs').promises.stat,
      readDirectory: require('fs').promises.readdir
    }
  },
  Uri: {
    file: (path) => ({
      fsPath: path,
      scheme: 'file',
      path: path,
      toString: () => `file://${path}`
    }),
    parse: (str) => {
      const path = str.replace('file://', '');
      return {
        fsPath: path,
        scheme: 'file',
        path: path,
        toString: () => str
      };
    }
  },
  commands: {
    executeCommand: () => Promise.resolve(),
    registerCommand: () => ({ dispose: () => {} })
  },
  env: {
    openExternal: (uri) => {
      const url = typeof uri === 'string' ? uri : uri.toString();
      console.log(`🔗 Opening: ${url}`);
      return Promise.resolve(true);
    },
    clipboard: {
      readText: () => Promise.resolve(''),
      writeText: (text) => {
        console.log(`📋 Copied to clipboard: ${text}`);
        return Promise.resolve();
      }
    }
  },
  extensions: {
    all: [],
    getExtension: () => undefined
  }
};

// Set up process to handle terminal properly
process.stdin.setRawMode = process.stdin.setRawMode || (() => {});
process.stdout.isTTY = true;
process.stdin.isTTY = true;

// Capture original console methods before cline-cli modifies them
const originalConsoleLog = console.log;
const originalConsoleError = console.error;

// Handle uncaught exceptions gracefully
process.on('uncaughtException', (error) => {
  if (error.message && error.message.includes('appendLine')) {
    // Silently handle appendLine errors we couldn't catch
    return;
  }
  originalConsoleError('Uncaught exception:', error.message);
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  if (reason && reason.toString().includes('appendLine')) {
    // Silently handle appendLine rejections
    return;
  }
  originalConsoleError('Unhandled rejection:', reason);
});

// Now load the actual cline-cli
try {
  // Try to load the cline-cli module
  const clinePath = '/opt/homebrew/lib/node_modules/@yaegaki/cline-cli/dist/main.js';

  // Check if the module exists
  const fs = require('fs');
  if (!fs.existsSync(clinePath)) {
    console.error('❌ Cline CLI is not installed.');
    console.error('Please run: npm install -g @yaegaki/cline-cli');
    process.exit(1);
  }

  // Load and run cline-cli with our patches in place
  require(clinePath);

} catch (error) {
  if (error.message && error.message.includes('appendLine')) {
    // If we still get appendLine errors, just continue
    // The tool should still work with our shims
  } else {
    originalConsoleError('Failed to load Cline CLI:', error.message);
    originalConsoleError('Please ensure @yaegaki/cline-cli is installed globally');
    process.exit(1);
  }
}