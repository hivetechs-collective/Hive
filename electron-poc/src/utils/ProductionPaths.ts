/**
 * ProductionPaths - Helper for resolving paths in production vs development
 * Ensures proper path resolution for packaged Electron apps
 */

import { app } from 'electron';
import * as path from 'path';
import * as fs from 'fs';

export class ProductionPaths {
  /**
   * Get the Node.js executable path
   * In production: uses Electron's built-in Node
   * In development: finds system Node
   */
  static getNodeExecutable(): string {
    // For bundled app, use Electron's node
    if (app.isPackaged) {
      return process.execPath;  // Electron's built-in Node
    }
    
    // Development: find system Node
    return this.findSystemNode();
  }

  /**
   * Find system Node.js installation
   */
  private static findSystemNode(): string {
    const possiblePaths = [
      '/usr/local/bin/node',     // Homebrew Intel
      '/opt/homebrew/bin/node',   // Homebrew Apple Silicon
      '/usr/bin/node',            // System Node (rare)
    ];
    
    // Add paths from PATH environment variable
    if (process.env.PATH) {
      const pathDirs = process.env.PATH.split(':');
      for (const dir of pathDirs) {
        possiblePaths.push(path.join(dir, 'node'));
      }
    }
    
    // Try each possible path
    for (const nodePath of possiblePaths) {
      try {
        if (fs.existsSync(nodePath)) {
          fs.accessSync(nodePath, fs.constants.X_OK);
          return nodePath;
        }
      } catch {
        // Path doesn't exist or isn't executable, try next
      }
    }
    
    // Fallback to 'node' and hope it's in PATH
    return 'node';
  }

  /**
   * Get resource path for bundled resources
   * @param resource - Relative path to resource
   */
  static getResourcePath(resource: string): string {
    if (app.isPackaged) {
      // Production: ./resources/app.asar.unpacked/
      return path.join(process.resourcesPath, 'app.asar.unpacked', resource);
    }
    // Development: project root
    return path.join(__dirname, '../../../', resource);
  }
  
  /**
   * Get binary path for bundled binaries
   * @param binaryName - Name of the binary
   */
  static getBinaryPath(binaryName: string): string {
    const basePath = this.getResourcePath('.webpack/main/binaries');
    const binaryPath = path.join(basePath, binaryName);
    
    // On Windows, add .exe extension if not present
    if (process.platform === 'win32' && !binaryName.endsWith('.exe')) {
      return `${binaryPath}.exe`;
    }
    
    return binaryPath;
  }
  
  /**
   * Get the memory service path
   */
  static getMemoryServicePath(): string {
    if (app.isPackaged) {
      // Production: bundled memory service
      return path.join(
        process.resourcesPath,
        'app.asar.unpacked',
        '.webpack',
        'main',
        'memory-service',
        'index.js'
      );
    }
    // Development: compiled memory service
    return path.join(__dirname, '../../../.webpack/main/memory-service/index.js');
  }
  
  /**
   * Get Python runtime path
   */
  static getPythonPath(): string {
    if (app.isPackaged) {
      const pythonRuntimePath = path.join(
        process.resourcesPath,
        'app.asar.unpacked',
        '.webpack',
        'main',
        'resources',
        'python-runtime',
        'python'
      );
      
      // Platform-specific Python executable
      return process.platform === 'win32'
        ? path.join(pythonRuntimePath, 'python.exe')
        : path.join(pythonRuntimePath, 'bin', 'python3');
    }
    
    // Development: find Python in various locations
    const possiblePythonPaths = [
      path.join(__dirname, '../../../../venv/bin/python3'),
      path.join(__dirname, '../../../../.venv/bin/python3'),
      '/usr/bin/python3',
      '/usr/local/bin/python3',
      'python3'
    ];
    
    for (const pythonPath of possiblePythonPaths) {
      try {
        if (fs.existsSync(pythonPath)) {
          return pythonPath;
        }
      } catch {
        // Continue to next path
      }
    }
    
    return 'python3'; // Fallback
  }
  
  /**
   * Get app data directory for persistent storage
   */
  static getAppDataPath(): string {
    return app.getPath('userData');
  }
  
  /**
   * Get logs directory
   */
  static getLogsPath(): string {
    return path.join(this.getAppDataPath(), 'logs');
  }
  
  /**
   * Get temporary directory for runtime files
   */
  static getTempPath(): string {
    return app.getPath('temp');
  }
  
  /**
   * Ensure a directory exists, creating it if necessary
   */
  static ensureDir(dirPath: string): void {
    if (!fs.existsSync(dirPath)) {
      fs.mkdirSync(dirPath, { recursive: true });
    }
  }
  
  /**
   * Check if running in production
   */
  static isProduction(): boolean {
    return app.isPackaged;
  }
  
  /**
   * Get the main Hive backend server path
   */
  static getBackendServerPath(): string {
    return this.getBinaryPath('hive-backend-server-enhanced');
  }
}