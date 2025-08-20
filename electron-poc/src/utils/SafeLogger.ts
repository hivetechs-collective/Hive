/**
 * SafeLogger - Production-ready logging system with EPIPE error handling
 * 
 * Replaces console.log/error to prevent EPIPE crashes when child processes
 * use stdio: 'inherit'. Implements 2025 best practices for Electron apps.
 */

import * as fs from 'fs';
import * as path from 'path';
import { app } from 'electron';

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
  FATAL = 4
}

export interface LoggerOptions {
  logToFile?: boolean;
  logToConsole?: boolean;
  logLevel?: LogLevel;
  maxFileSize?: number; // in bytes
  maxFiles?: number;
  logDir?: string;
}

class SafeLogger {
  private static instance: SafeLogger;
  private writeStream: fs.WriteStream | null = null;
  private logLevel: LogLevel;
  private logToFile: boolean;
  private logToConsole: boolean;
  private maxFileSize: number;
  private maxFiles: number;
  private logDir: string;
  private currentLogFile: string;
  private logQueue: string[] = [];
  private isWriting: boolean = false;

  private constructor(options: LoggerOptions = {}) {
    this.logLevel = options.logLevel ?? LogLevel.INFO;
    this.logToFile = options.logToFile ?? true;
    this.logToConsole = options.logToConsole ?? (process.env.NODE_ENV === 'development');
    this.maxFileSize = options.maxFileSize ?? 10 * 1024 * 1024; // 10MB default
    this.maxFiles = options.maxFiles ?? 5;
    
    // Use app userData directory for logs
    this.logDir = options.logDir ?? path.join(app.getPath('userData'), 'logs');
    
    if (this.logToFile) {
      this.initializeFileLogging();
    }

    // Handle process exit gracefully
    process.on('exit', () => this.close());
    process.on('SIGINT', () => this.close());
    process.on('SIGTERM', () => this.close());
  }

  public static getInstance(options?: LoggerOptions): SafeLogger {
    if (!SafeLogger.instance) {
      SafeLogger.instance = new SafeLogger(options);
    }
    return SafeLogger.instance;
  }

  private initializeFileLogging(): void {
    try {
      // Ensure log directory exists
      if (!fs.existsSync(this.logDir)) {
        fs.mkdirSync(this.logDir, { recursive: true });
      }

      // Create log filename with timestamp
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      this.currentLogFile = path.join(this.logDir, `hive-${timestamp}.log`);
      
      // Create write stream with append flag
      this.writeStream = fs.createWriteStream(this.currentLogFile, { 
        flags: 'a',
        encoding: 'utf8'
      });

      this.writeStream.on('error', (error) => {
        // If we can't write to file, fall back to console only
        this.logToFile = false;
        this.safeConsoleLog('ERROR', `Failed to write to log file: ${error.message}`);
      });

      // Rotate logs if needed
      this.rotateLogsIfNeeded();
    } catch (error) {
      this.logToFile = false;
      this.safeConsoleLog('ERROR', `Failed to initialize file logging: ${error}`);
    }
  }

  private rotateLogsIfNeeded(): void {
    try {
      const stats = fs.statSync(this.currentLogFile);
      if (stats.size > this.maxFileSize) {
        this.rotateLogs();
      }
    } catch {
      // File doesn't exist yet, that's fine
    }

    // Clean up old log files
    this.cleanOldLogs();
  }

  private rotateLogs(): void {
    if (this.writeStream) {
      this.writeStream.end();
      this.initializeFileLogging();
    }
  }

  private cleanOldLogs(): void {
    try {
      const files = fs.readdirSync(this.logDir)
        .filter(f => f.startsWith('hive-') && f.endsWith('.log'))
        .map(f => ({
          name: f,
          path: path.join(this.logDir, f),
          time: fs.statSync(path.join(this.logDir, f)).mtime.getTime()
        }))
        .sort((a, b) => b.time - a.time);

      // Keep only the most recent files
      if (files.length > this.maxFiles) {
        files.slice(this.maxFiles).forEach(file => {
          try {
            fs.unlinkSync(file.path);
          } catch {
            // Ignore errors when deleting old logs
          }
        });
      }
    } catch {
      // Ignore errors in cleanup
    }
  }

  private formatMessage(level: string, message: string, meta?: any): string {
    const timestamp = new Date().toISOString();
    const pid = process.pid;
    let formatted = `[${timestamp}] [${level}] [PID:${pid}] ${message}`;
    
    if (meta) {
      try {
        formatted += ` ${JSON.stringify(meta)}`;
      } catch {
        formatted += ` [Unserializable meta data]`;
      }
    }
    
    return formatted;
  }

  private safeConsoleLog(level: string, message: string): void {
    if (!this.logToConsole) return;

    try {
      // Check if stdout is writable before attempting to write
      if (process.stdout && process.stdout.writable) {
        const output = `[${level}] ${message}\n`;
        process.stdout.write(output);
      }
    } catch (error: any) {
      // Silently ignore EPIPE errors - this is the whole point of SafeLogger
      if (error.code !== 'EPIPE' && error.code !== 'EBADF') {
        // For non-EPIPE errors, try stderr as fallback
        try {
          if (process.stderr && process.stderr.writable) {
            process.stderr.write(`[LOGGER ERROR] Failed to write to stdout: ${error.message}\n`);
          }
        } catch {
          // Give up - we can't write anywhere
        }
      }
    }
  }

  private writeToFile(message: string): void {
    if (!this.logToFile || !this.writeStream) return;

    // Add to queue
    this.logQueue.push(message + '\n');
    
    // Process queue if not already processing
    if (!this.isWriting) {
      this.processQueue();
    }
  }

  private async processQueue(): Promise<void> {
    if (this.isWriting || this.logQueue.length === 0) return;
    
    this.isWriting = true;
    
    while (this.logQueue.length > 0) {
      const message = this.logQueue.shift()!;
      try {
        await new Promise<void>((resolve, reject) => {
          if (!this.writeStream) {
            resolve();
            return;
          }
          
          this.writeStream.write(message, (error) => {
            if (error) reject(error);
            else resolve();
          });
        });
      } catch (error) {
        // If write fails, try to reinitialize
        this.initializeFileLogging();
        break;
      }
    }
    
    this.isWriting = false;
    
    // Check for rotation after write
    this.rotateLogsIfNeeded();
  }

  private log(level: LogLevel, levelName: string, message: string, meta?: any): void {
    if (level < this.logLevel) return;

    const formatted = this.formatMessage(levelName, message, meta);
    
    // Write to console (safely)
    this.safeConsoleLog(levelName, message);
    
    // Write to file
    this.writeToFile(formatted);
  }

  // Public logging methods
  public debug(message: string, meta?: any): void {
    this.log(LogLevel.DEBUG, 'DEBUG', message, meta);
  }

  public info(message: string, meta?: any): void {
    this.log(LogLevel.INFO, 'INFO', message, meta);
  }

  public warn(message: string, meta?: any): void {
    this.log(LogLevel.WARN, 'WARN', message, meta);
  }

  public error(message: string, meta?: any): void {
    this.log(LogLevel.ERROR, 'ERROR', message, meta);
  }

  public fatal(message: string, meta?: any): void {
    this.log(LogLevel.FATAL, 'FATAL', message, meta);
  }

  // Compatibility methods for easy migration from console
  public log(message: string, ...args: any[]): void {
    const meta = args.length > 0 ? args : undefined;
    this.info(message, meta);
  }

  public close(): void {
    if (this.writeStream) {
      // Flush any remaining logs
      this.processQueue().then(() => {
        if (this.writeStream) {
          this.writeStream.end();
          this.writeStream = null;
        }
      });
    }
  }

  // Get log file path for debugging
  public getLogFilePath(): string | null {
    return this.currentLogFile;
  }

  // Set log level dynamically
  public setLogLevel(level: LogLevel): void {
    this.logLevel = level;
  }
}

// Export singleton instance for easy use
export const logger = SafeLogger.getInstance();

// Export default for convenience
export default SafeLogger;