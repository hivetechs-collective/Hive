/**
 * High-Performance File System Operations
 * Runs in main process for optimal performance
 */

import * as fs from 'fs';
import * as path from 'path';
import { promisify } from 'util';
import { createReadStream, createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';

const readdir = promisify(fs.readdir);
const stat = promisify(fs.stat);
const readFile = promisify(fs.readFile);
const writeFile = promisify(fs.writeFile);

export interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  size?: number;
  modified?: Date;
}

export class FileSystemManager {
  private watchHandlers = new Map<string, fs.FSWatcher>();
  private fileCache = new Map<string, { content: string; timestamp: number }>();
  private CACHE_TTL = 5000; // 5 seconds cache

  /**
   * Get file tree with depth limit for performance
   */
  async getFileTree(rootPath: string, maxDepth = 2): Promise<FileNode[]> {
    return this.scanDirectory(rootPath, 0, maxDepth);
  }

  private async scanDirectory(dirPath: string, currentDepth: number, maxDepth: number): Promise<FileNode[]> {
    if (currentDepth >= maxDepth) {
      return [];
    }

    try {
      const entries = await readdir(dirPath, { withFileTypes: true });
      const nodes: FileNode[] = [];

      // Use Promise.all for parallel processing but limit concurrency
      const BATCH_SIZE = 10;
      for (let i = 0; i < entries.length; i += BATCH_SIZE) {
        const batch = entries.slice(i, i + BATCH_SIZE);
        const batchNodes = await Promise.all(
          batch.map(async (entry) => {
            // Skip hidden files and node_modules for performance
            if (entry.name.startsWith('.') || entry.name === 'node_modules') {
              return null;
            }

            const fullPath = path.join(dirPath, entry.name);
            const node: FileNode = {
              name: entry.name,
              path: fullPath,
              type: entry.isDirectory() ? 'directory' : 'file'
            };

            if (entry.isDirectory() && currentDepth < maxDepth - 1) {
              // Lazy load children - don't load immediately
              node.children = [];
            }

            if (entry.isFile()) {
              try {
                const stats = await stat(fullPath);
                node.size = stats.size;
                node.modified = stats.mtime;
              } catch (error) {
                // Ignore stat errors
              }
            }

            return node;
          })
        );

        nodes.push(...batchNodes.filter(Boolean) as FileNode[]);
      }

      return nodes.sort((a, b) => {
        // Directories first, then alphabetical
        if (a.type !== b.type) {
          return a.type === 'directory' ? -1 : 1;
        }
        return a.name.localeCompare(b.name);
      });
    } catch (error) {
      console.error(`Error scanning directory ${dirPath}:`, error);
      return [];
    }
  }

  /**
   * Get directory contents (for lazy loading)
   */
  async getDirectoryContents(dirPath: string): Promise<FileNode[]> {
    return this.scanDirectory(dirPath, 0, 1);
  }

  /**
   * Read file with caching and streaming for large files
   */
  async readFile(filePath: string): Promise<string> {
    // Check cache first
    const cached = this.fileCache.get(filePath);
    if (cached && Date.now() - cached.timestamp < this.CACHE_TTL) {
      return cached.content;
    }

    const stats = await stat(filePath);
    
    // For large files (>1MB), use streaming
    if (stats.size > 1024 * 1024) {
      return this.readLargeFile(filePath);
    }

    // For small files, read normally
    const content = await readFile(filePath, 'utf-8');
    
    // Cache the content
    this.fileCache.set(filePath, {
      content,
      timestamp: Date.now()
    });

    return content;
  }

  /**
   * Read large file in chunks
   */
  private async readLargeFile(filePath: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const chunks: Buffer[] = [];
      const stream = createReadStream(filePath, { encoding: 'utf8' });
      
      stream.on('data', (chunk) => chunks.push(Buffer.from(chunk)));
      stream.on('end', () => resolve(Buffer.concat(chunks).toString()));
      stream.on('error', reject);
    });
  }

  /**
   * Write file with atomic write for safety
   */
  async writeFileContent(filePath: string, content: string): Promise<void> {
    // Write to temp file first
    const tempPath = `${filePath}.tmp`;
    await writeFile(tempPath, content, 'utf-8');
    
    // Atomic rename
    fs.renameSync(tempPath, filePath);
    
    // Invalidate cache
    this.fileCache.delete(filePath);
  }

  /**
   * Watch file for changes
   */
  watchFile(filePath: string, callback: () => void): void {
    // Close existing watcher
    this.unwatchFile(filePath);
    
    const watcher = fs.watch(filePath, { persistent: false }, (eventType) => {
      if (eventType === 'change') {
        // Invalidate cache
        this.fileCache.delete(filePath);
        // Debounce callback
        setTimeout(callback, 100);
      }
    });
    
    this.watchHandlers.set(filePath, watcher);
  }

  /**
   * Stop watching file
   */
  unwatchFile(filePath: string): void {
    const watcher = this.watchHandlers.get(filePath);
    if (watcher) {
      watcher.close();
      this.watchHandlers.delete(filePath);
    }
  }

  /**
   * Search files with ripgrep for performance
   */
  async searchFiles(rootPath: string, pattern: string): Promise<string[]> {
    return new Promise((resolve, reject) => {
      const { spawn } = require('child_process');
      const rg = spawn('rg', ['-l', pattern, rootPath]);
      
      let output = '';
      rg.stdout.on('data', (data: Buffer) => {
        output += data.toString();
      });
      
      rg.on('close', (code: number) => {
        if (code === 0) {
          resolve(output.split('\n').filter(Boolean));
        } else {
          // Fallback to Node.js search if ripgrep not available
          resolve([]);
        }
      });
      
      rg.on('error', () => {
        // Fallback to Node.js search
        resolve([]);
      });
    });
  }

  /**
   * Get file stats
   */
  async getFileStats(filePath: string): Promise<fs.Stats | null> {
    try {
      return await stat(filePath);
    } catch (error) {
      return null;
    }
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    // Close all watchers
    this.watchHandlers.forEach(watcher => watcher.close());
    this.watchHandlers.clear();
    
    // Clear cache
    this.fileCache.clear();
  }
}