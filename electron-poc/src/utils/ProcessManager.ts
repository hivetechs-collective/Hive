/**
 * ProcessManager - Manages child processes lifecycle for production
 * Handles spawning, monitoring, restarting, and cleanup
 *
 * Diagrams
 * - electron-poc/MASTER_ARCHITECTURE.md#diagram-system-overview
 * - electron-poc/MASTER_ARCHITECTURE.md#diagram-startup-sequence
 * - electron-poc/MASTER_ARCHITECTURE.md#diagram-dynamic-port-allocation
 */

import { ChildProcess, fork, spawn } from 'child_process';
import { EventEmitter } from 'events';
import { app } from 'electron';
import * as fs from 'fs';
import path from 'path';
import { PortManager } from './PortManager';
import { PidTracker } from './PidTracker';

import { logger } from './SafeLogger';
export interface ProcessConfig {
  name: string;
  scriptPath: string;
  args?: string[];
  env?: NodeJS.ProcessEnv;
  port?: number;
  alternativePorts?: number[];
  autoRestart?: boolean;
  maxRestarts?: number;
  restartDelay?: number;
  healthCheckUrl?: string;
  healthCheckInterval?: number;
}

export interface ProcessInfo {
  name: string;
  process: ChildProcess | null;
  pid?: number;
  port?: number;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'crashed';
  restartCount: number;
  lastStartTime?: Date;
  lastError?: string;
}

export class ProcessManager extends EventEmitter {
  private processes: Map<string, ProcessInfo> = new Map();
  private configs: Map<string, ProcessConfig> = new Map();
  private healthCheckTimers: Map<string, NodeJS.Timeout> = new Map();
  private shutdownInProgress = false;
  
  constructor() {
    super();
    this.setupShutdownHandlers();
  }
  
  private findNodeExecutable(): string {
    // First check if build script found Node.js and saved the path
    if (app.isPackaged) {
      const path = require('path');
      const fs = require('fs');
      const envPath = path.join(__dirname, '.env.production');
      
      if (fs.existsSync(envPath)) {
        const envContent = fs.readFileSync(envPath, 'utf8');
        const nodePathMatch = envContent.match(/NODE_PATH=(.+)/);
        const useElectronMatch = envContent.match(/USE_ELECTRON_AS_NODE=true/);
        
        if (nodePathMatch && nodePathMatch[1]) {
          const discoveredPath = nodePathMatch[1].trim();
          logger.info(`[ProcessManager] Using Node.js from .env.production: ${discoveredPath}`);
          
          // If it's the Electron binary, we already handle ELECTRON_RUN_AS_NODE later
          if (useElectronMatch || discoveredPath === process.execPath) {
            logger.info(`[ProcessManager] Will use ELECTRON_RUN_AS_NODE=1 for Electron binary`);
          }
          
          return discoveredPath;
        }
      }
      
      // Fallback to Electron if no saved path
      logger.info(`[ProcessManager] No saved Node path, using Electron's Node.js: ${process.execPath}`);
      return process.execPath;
    }
    
    // In development, find system Node.js
    // macOS apps launch with minimal PATH: /usr/bin:/bin:/usr/sbin:/sbin
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
    
    const { execSync } = require('child_process');
    
    // Try each possible path
    for (const nodePath of possiblePaths) {
      try {
        if (fs.existsSync(nodePath)) {
          // Verify it's executable
          fs.accessSync(nodePath, fs.constants.X_OK);
          execSync(`${nodePath} --version`, { stdio: 'pipe' });
          logger.info(`[ProcessManager] Found Node.js at: ${nodePath}`);
          return nodePath;
        }
      } catch (e) {
        // Path doesn't exist or isn't executable, try next
      }
    }
    
    // Last resort: try 'which node' command
    try {
      const nodePath = execSync('which node', { encoding: 'utf8' }).trim();
      if (nodePath) {
        logger.info(`[ProcessManager] Found Node.js via which: ${nodePath}`);
        return nodePath;
      }
    } catch (e) {
      // which command failed
    }
    
    // Ultimate fallback - use Electron's node
    logger.warn('[ProcessManager] Could not find system Node.js, using Electron\'s node');
    return process.execPath;
  }
  
  /**
   * Register a process configuration
   */
  registerProcess(config: ProcessConfig): void {
    this.configs.set(config.name, config);
    this.processes.set(config.name, {
      name: config.name,
      process: null,
      status: 'stopped',
      restartCount: 0
    });
    
    logger.info(`[ProcessManager] Registered process: ${config.name}`);
  }
  
  /**
   * Start a managed process
   */
  async startProcess(name: string): Promise<boolean> {
    const config = this.configs.get(name);
    if (!config) {
      throw new Error(`Process ${name} not registered`);
    }
    
    const info = this.processes.get(name)!;
    
    if (info.status === 'running') {
      logger.info(`[ProcessManager] Process ${name} is already running`);
      return true;
    }
    
    info.status = 'starting';
    this.emit('process:starting', name);
    
    try {
      // Allocate port dynamically if service needs one
      let port: number | undefined;
      if (config.port !== undefined) {
        // Use the new dynamic allocation - just need the service name!
        port = await PortManager.allocatePortForService(name);
        info.port = port;
        logger.info(`[ProcessManager] ${name} allocated port ${port} (dynamic)`);
      }
      
      // Prepare environment
      const env = {
        ...process.env,
        ...config.env,
        ...(port ? { PORT: port.toString(), MEMORY_SERVICE_PORT: port.toString() } : {})
      };
      
      logger.info(`[ProcessManager] Starting ${name} on port ${port || 'N/A'}`);
      
      // Spawn the process - handle different file types
      let childProcess: ChildProcess;
      let binaryReadyPromise: Promise<boolean> | null = null;
      
      if (config.scriptPath.endsWith('.ts')) {
        // CRITICAL: In production Electron app, we CANNOT use fork() as it tries to use Electron's Node
        // We must use spawn with the system's Node.js executable
        if (app.isPackaged) {
          // In production, use Electron's Node.js in Node mode
          const nodePath = this.findNodeExecutable();
          const tsNodePath = require.resolve('ts-node/register');
          // If using Electron's executable, we need to pass special flags
          const isElectronNode = nodePath === process.execPath;
          const nodeArgs = isElectronNode 
            ? ['--no-asar', '-r', tsNodePath, config.scriptPath, ...(config.args || [])]
            : ['-r', tsNodePath, config.scriptPath, ...(config.args || [])];
          
          childProcess = spawn(nodePath, nodeArgs, {
            env: isElectronNode ? {
              ...env,  // Include all env vars including PORT!
              ELECTRON_RUN_AS_NODE: '1' // Run Electron as plain Node
            } : env,  // For non-Electron node, just use env as-is
            stdio: ['pipe', 'pipe', 'pipe', 'ipc'],  // Enable IPC channel
            detached: false
          });
        } else {
          // In development, fork works fine
          const tsNodePath = require.resolve('ts-node/register');
          childProcess = fork(config.scriptPath, config.args || [], {
            env,
            silent: true,  // Capture stdout/stderr so we can log errors
            detached: false,
            execArgv: ['-r', tsNodePath]
          });
        }
        
        // Capture stderr to see why it's crashing
        if (childProcess.stderr) {
          childProcess.stderr.on('data', (data: Buffer) => {
            logger.error(`[ProcessManager] ${name} stderr:`, data.toString());
          });
        }
        
        // Capture stdout too for debugging
        if (childProcess.stdout) {
          childProcess.stdout.on('data', (data: Buffer) => {
            logger.info(`[ProcessManager] ${name} stdout:`, data.toString());
          });
        }
      } else if (config.scriptPath.endsWith('.js')) {
        // CRITICAL FIX: Use spawn('node') in production to avoid "Unable to find helper app" error
        // Per MASTER_ARCHITECTURE.md line 3537-3558
        
        if (app.isPackaged) {
          // Production: Use spawn with node to avoid Electron helper issues
          logger.info(`[ProcessManager] ${name} using spawn('node') in production to avoid helper app issues`);
          
          // Find node executable (will check .env.production first)
          const nodePath = this.findNodeExecutable();
          const isElectronNode = nodePath === process.execPath;
          
          childProcess = spawn(nodePath, [config.scriptPath, ...(config.args || [])], {
            env: isElectronNode ? {
              ...env,  // Include all env vars including PORT!
              ELECTRON_RUN_AS_NODE: '1' // Only needed if using Electron as Node
            } : env,
            stdio: ['pipe', 'pipe', 'pipe', 'ipc'],  // Enable IPC channel
            detached: false
          });
        } else {
          // In development, fork works fine
          childProcess = fork(config.scriptPath, config.args || [], {
            env,
            silent: true,  // Capture stdout/stderr so we can log errors
            detached: false
          });
        }
        
        // Capture stderr to see why it's crashing
        if (childProcess.stderr) {
          childProcess.stderr.on('data', (data: Buffer) => {
            logger.error(`[ProcessManager] ${name} stderr:`, data.toString());
          });
        }
        
        // Capture stdout too for debugging
        if (childProcess.stdout) {
          childProcess.stdout.on('data', (data: Buffer) => {
            logger.info(`[ProcessManager] ${name} stdout:`, data.toString());
          });
        }
      } else {
        // For binary executables (Rust, Go, etc.), use spawn
        logger.info(`[ProcessManager] Spawning binary executable: ${config.scriptPath}`);
        
        // CRITICAL: In production, binaries lose execute permissions when unpacked from asar
        // We need to ensure they're executable before spawning
        if (app.isPackaged) {
          const fs = require('fs');
          const { execSync } = require('child_process');
          try {
            // Check if file exists and make it executable
            if (fs.existsSync(config.scriptPath)) {
              logger.info(`[ProcessManager] Setting execute permissions on ${config.scriptPath}`);
              fs.chmodSync(config.scriptPath, 0o755);
              
              // On macOS, also remove quarantine attribute if present
              // This prevents "cannot be opened because the developer cannot be verified" errors
              if (process.platform === 'darwin') {
                try {
                  execSync(`xattr -d com.apple.quarantine "${config.scriptPath}" 2>/dev/null || true`);
                  logger.info(`[ProcessManager] Removed quarantine attribute from ${config.scriptPath}`);
                } catch (e) {
                  // Ignore if xattr fails (file might not have the attribute)
                }
              }
            } else {
              logger.error(`[ProcessManager] Binary does not exist at path: ${config.scriptPath}`);
              throw new Error(`Binary not found: ${config.scriptPath}`);
            }
          } catch (error) {
            logger.error(`[ProcessManager] Failed to prepare binary:`, error);
            throw error;
          }
        }
        
        // CRITICAL CONFIGURATION - DO NOT CHANGE
        // AI Helpers spawn Python subprocesses that require full stdio access
        // Webpack minification must NOT alter this value
        const STDIO_CONFIG = 'inherit' as const; // Explicit constant to prevent webpack transformation
        
        // Runtime verification of critical configuration
        if (STDIO_CONFIG !== 'inherit') {
          logger.error(`[ProcessManager] CRITICAL ERROR: stdio configuration has been altered!`);
          logger.error(`[ProcessManager] Expected: 'inherit', Got: ${STDIO_CONFIG}`);
          throw new Error('Critical configuration corrupted - stdio must be "inherit" for AI Helpers');
        }
        
        try {
          const spawnOptions = {
            env,
            stdio: STDIO_CONFIG,  // Use explicit constant to prevent webpack issues
            detached: false,
            shell: false  // Don't use shell to avoid issues with spaces in paths
          };
          
          // Log spawn configuration for debugging
          logger.info(`[ProcessManager] Spawning ${name} with stdio: ${spawnOptions.stdio}`);
          
          childProcess = spawn(config.scriptPath, config.args || [], spawnOptions);
          
          // With 'inherit' stdio, we can't capture output, so we won't have a binaryReadyPromise
          // We'll rely solely on port checking for readiness detection
          logger.info(`[ProcessManager] Binary process ${name} spawned with inherited stdio`);
          logger.info(`[ProcessManager] Binary PID: ${childProcess.pid}`);
          logger.info(`[ProcessManager] Will use port checking for readiness (port ${port})`);
        } catch (spawnError: any) {
          logger.error(`[ProcessManager] Failed to spawn binary ${name}:`, spawnError);
          logger.error(`[ProcessManager] Path: ${config.scriptPath}`);
          logger.error(`[ProcessManager] Error code: ${spawnError.code}`);
          logger.error(`[ProcessManager] Error message: ${spawnError.message}`);
          
          // Try to provide more helpful error messages
          if (spawnError.code === 'ENOENT') {
            logger.error(`[ProcessManager] Binary not found at path. Checking if file exists...`);
            const fs = require('fs');
            if (fs.existsSync(config.scriptPath)) {
              logger.error(`[ProcessManager] File exists but cannot be executed. This might be a permission or code signing issue.`);
              // Try executing with explicit /bin/sh
              logger.info(`[ProcessManager] Attempting to run binary directly with execFile...`);
              const { execFile } = require('child_process');
              childProcess = execFile(config.scriptPath, config.args || [], {
                env,
                maxBuffer: 10 * 1024 * 1024  // 10MB buffer for output
              }, (error: any, stdout: any, stderr: any) => {
                if (error) {
                  logger.error(`[ProcessManager] execFile also failed:`, error);
                }
              });
            } else {
              logger.error(`[ProcessManager] File does not exist at path: ${config.scriptPath}`);
            }
          }
          
          if (!childProcess) {
            throw spawnError;
          }
        }
        
        // Note: childProcess.stdout and childProcess.stderr are null with 'inherit'
        // These blocks won't execute with 'inherit' stdio, but keeping them for future reference
        // if we need to switch back to captured stdio for debugging
      }
      
      info.process = childProcess;
      info.pid = childProcess.pid;
      info.lastStartTime = new Date();
      
      // Track the PID for cleanup on crash/restart
      if (childProcess.pid) {
        PidTracker.addPid(childProcess.pid, name);
      }
      
      // Wait for process to be ready - check for 'ready' message or port binding
      // Binary processes like Rust servers may take longer to initialize
      let isReady = false;
      let readyResolver: ((value: boolean) => void) | null = null;
      let readyTimeout: NodeJS.Timeout | null = null;
      
      // Create ready promise for Node.js processes BEFORE setting up message handlers
      const readyPromise = (config.scriptPath.endsWith('.ts') || config.scriptPath.endsWith('.js')) 
        ? new Promise<boolean>((resolve) => {
            readyResolver = resolve;
            readyTimeout = setTimeout(() => {
              logger.info(`[ProcessManager] Timeout waiting for ${name} ready signal (waited 15000ms)`);
              resolve(false);
            }, 15000);
          })
        : null;
      
      // Set up event handlers - now the ready promise is already created
      childProcess.on('message', (msg: any) => {
        logger.info(`[ProcessManager] Message received from ${name}:`, msg);
        // Handle ready message first if we're waiting for it
        if (readyResolver && msg.type === 'ready') {
          logger.info(`[ProcessManager] Got ready message from ${name}`);
          if (readyTimeout) clearTimeout(readyTimeout);
          readyResolver(true);
          readyResolver = null; // Clear so we don't resolve twice
        }
        // Then handle normally
        this.handleProcessMessage(name, msg);
      });
      
      childProcess.on('error', (error) => {
        logger.error(`[ProcessManager] Process ${name} error:`, error);
        info.lastError = error.message;
        this.handleProcessCrash(name);
      });
      
      childProcess.on('exit', (code, signal) => {
        logger.info(`[ProcessManager] Process ${name} exited with code ${code}, signal ${signal}`);
        
        if (!this.shutdownInProgress && info.status !== 'stopping') {
          this.handleProcessCrash(name);
        } else {
          info.status = 'stopped';
          this.emit('process:stopped', name);
        }
      });
      
      if (readyPromise) {
        // For Node.js processes, wait for IPC 'ready' message
        logger.info(`[ProcessManager] Waiting for IPC ready message from ${name}...`);
        isReady = await readyPromise;
        logger.info(`[ProcessManager] IPC ready check complete for ${name}: ${isReady}`);
      } else if (binaryReadyPromise) {
        // For binary processes with captured output, wait for our custom ready detection
        const timeoutPromise = new Promise<boolean>((resolve) => {
          setTimeout(() => {
            logger.info(`[ProcessManager] Timeout waiting for ${name} startup output (waited 30000ms)`);
            resolve(false);
          }, 30000);
        });
        
        // Race between the binary ready promise and timeout (binaryReadyPromise is guaranteed to be non-null here)
        isReady = await Promise.race([binaryReadyPromise!, timeoutPromise]);
        
        if (isReady) {
          logger.info(`[ProcessManager] Binary process ${name} confirmed ready via output detection`);
        }
      } else {
        // For binary processes with 'inherit' stdio, we can't capture output
        logger.info(`[ProcessManager] Binary process ${name} uses inherited stdio - will check port only`);
      }
      
      // For processes without ready signal, check the port
      if (!isReady && port) {
        logger.info(`[ProcessManager] Checking port ${port} for ${name}...`);
        
        // Emit progress event
        this.emit('process:progress', {
          name,
          status: 'port-check',
          message: `Checking port ${port}...`,
          port
        });
        
        // Binary servers may take longer to bind to port after process starts
        // AI Helpers initialization can take time, so give them enough time to start
        const isBinary = !config.scriptPath.endsWith('.ts') && !config.scriptPath.endsWith('.js');
        
        if (isBinary) {
          // For binary processes, add initial delay to allow process to initialize
          logger.info(`[ProcessManager] Waiting 2 seconds for ${name} to initialize before port check...`);
          this.emit('process:progress', {
            name,
            status: 'initializing',
            message: 'Service initializing...',
            port
          });
          await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        // Keep checking until the service is ready - no arbitrary timeouts
        const checkInterval = 250; // Check every 250ms
        // We'll keep checking indefinitely until the service is ready
        // The user can cancel if they want, but we won't timeout
        
        let attempts = 0;
        let portReady = false;
        
        // Keep checking until the service is ready - no timeout
        while (!portReady) {
          attempts++;
          
          // CRITICAL: Check if the process has already crashed
          // Refresh info to get current status
          const currentInfo = this.processes.get(name);
          if (!currentInfo || !currentInfo.process || currentInfo.process.killed || currentInfo.status === 'crashed' || currentInfo.status === 'stopped') {
            logger.error(`[ProcessManager] Process ${name} has crashed/stopped while waiting for port ${port}`);
            this.emit('process:progress', {
              name,
              status: 'failed',
              message: `Service crashed during startup`,
              port
            });
            return false; // Return false to indicate startup failed
          }
          
          // Quick check if port is listening
          portReady = await PortManager.waitForService(port, checkInterval);
          
          if (portReady) {
            logger.info(`[ProcessManager] ✅ Port ${port} is ready for ${name} (${attempts * checkInterval}ms)`);
            this.emit('process:progress', {
              name,
              status: 'ready',
              message: `Service ready on port ${port}`,
              port
            });
            break;
          }
          
          // Report progress periodically (every 2.5 seconds)
          if (attempts % 10 === 0) {
            const elapsed = attempts * checkInterval;
            this.emit('process:progress', {
              name,
              status: 'waiting',
              message: `Waiting for service to start... (${Math.round(elapsed/1000)}s)`,
              port
            });
            
            // Log occasionally
            if (attempts % 40 === 0) {
              logger.info(`[ProcessManager] Still waiting for ${name} on port ${port}... (${Math.round(elapsed/1000)}s)`);
            }
          }
        }
        
        // If we get here, the service is ready
        isReady = true;
      }
      
      info.status = 'running';
      this.emit('process:started', name);
      
      // Validate AI Helpers for backend process
      const aiHelperValidation = await this.validateAIHelpers(name);
      if (!aiHelperValidation.valid) {
        logger.error(`[ProcessManager] AI Helper validation failed for ${name}: ${aiHelperValidation.issue}`);
        // Emit error but don't fail the process start - some operations may still work
        this.emit('aihelper:validation:failed', {
          process: name,
          issue: aiHelperValidation.issue,
          timestamp: new Date().toISOString()
        });
      }
      
      // Start AI Helper monitoring for backend
      this.startAIHelperMonitoring(name);
      
      // Start health checks if configured
      if (config.healthCheckUrl && config.healthCheckInterval) {
        this.startHealthCheck(name);
      }
      
      logger.info(`[ProcessManager] Process ${name} started successfully (PID: ${info.pid})`);
      return true;
      
    } catch (error: any) {
      logger.error(`[ProcessManager] Failed to start ${name}:`, error.message);
      info.status = 'crashed';
      info.lastError = error.message;
      this.emit('process:failed', name, error);
      
      // Release port if allocated
      if (info.port) {
        PortManager.releasePort(name);
      }
      
      return false;
    }
  }
  
  /**
   * Stop a managed process
   */
  async stopProcess(name: string, force = false): Promise<boolean> {
    const info = this.processes.get(name);
    if (!info || info.status === 'stopped') {
      return true;
    }
    
    info.status = 'stopping';
    this.emit('process:stopping', name);
    
    // Stop health checks
    this.stopHealthCheck(name);
    
    if (info.process) {
      try {
        // Send shutdown message first
        if (info.process.send) {
          info.process.send({ type: 'shutdown' });
        }
        
        // Give process time to shutdown gracefully
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Check if process is still running
        if (info.process.killed === false) {
          info.process.kill(force ? 'SIGKILL' : 'SIGTERM');
          
          // Wait for process to exit
          await new Promise(resolve => setTimeout(resolve, 1000));
        }
        
        // Remove PID tracking
        if (info.pid) {
          PidTracker.removePid(info.pid);
        }
        
        info.process = null;
        info.pid = undefined;
        
      } catch (error) {
        logger.error(`[ProcessManager] Error stopping ${name}:`, error);
      }
    }
    
    // Release port
    if (info.port) {
      PortManager.releasePort(name);
      info.port = undefined;
    }
    
    info.status = 'stopped';
    this.emit('process:stopped', name);
    
    logger.info(`[ProcessManager] Process ${name} stopped`);
    return true;
  }
  
  /**
   * Restart a process
   */
  async restartProcess(name: string): Promise<boolean> {
    logger.info(`[ProcessManager] Restarting ${name}...`);
    await this.stopProcess(name);
    await new Promise(resolve => setTimeout(resolve, 1000));
    return this.startProcess(name);
  }
  
  /**
   * Handle process crash and auto-restart if configured
   */
  private async handleProcessCrash(name: string): Promise<void> {
    const config = this.configs.get(name);
    const info = this.processes.get(name)!;
    
    info.status = 'crashed';
    this.emit('process:crashed', name);
    
    // Clean up process reference
    if (info.process) {
      // Remove PID tracking
      if (info.pid) {
        PidTracker.removePid(info.pid);
      }
      
      info.process = null;
      info.pid = undefined;
    }
    
    // CRITICAL: Release the port when process crashes
    if (info.port) {
      logger.info(`[ProcessManager] Releasing port ${info.port} after ${name} crashed`);
      PortManager.releasePort(name);
      info.port = undefined;
    }
    
    // Stop health checks
    this.stopHealthCheck(name);
    
    // Check if auto-restart is enabled
    if (config?.autoRestart && !this.shutdownInProgress) {
      const maxRestarts = config.maxRestarts || 5;
      
      if (info.restartCount < maxRestarts) {
        info.restartCount++;
        const delay = config.restartDelay || 3000;
        
        logger.info(`[ProcessManager] Auto-restarting ${name} in ${delay}ms (attempt ${info.restartCount}/${maxRestarts})`);
        
        setTimeout(async () => {
          if (!this.shutdownInProgress) {
            const success = await this.startProcess(name);
            if (!success) {
              logger.error(`[ProcessManager] Failed to restart ${name}`);
            }
          }
        }, delay);
      } else {
        logger.error(`[ProcessManager] Process ${name} exceeded max restart attempts`);
        this.emit('process:failed', name, new Error('Max restarts exceeded'));
      }
    }
  }
  
  /**
   * Handle messages from child processes
   */
  private handleProcessMessage(name: string, message: any): void {
    logger.info(`[ProcessManager] Message from ${name}:`, message);
    
    if (message.type === 'ready') {
      const info = this.processes.get(name)!;
      info.status = 'running';
      this.emit('process:ready', name, message);
    }
    
    // Forward message to main process
    this.emit('process:message', name, message);
  }
  
  /**
   * Start health checks for a process
   */
  private startHealthCheck(name: string): void {
    const config = this.configs.get(name)!;
    const info = this.processes.get(name)!;
    
    if (!config.healthCheckUrl || !config.healthCheckInterval) {
      return;
    }
    
    const timer = setInterval(async () => {
      if (info.status === 'running' && info.port) {
        try {
          const url = config.healthCheckUrl!.replace('{port}', info.port.toString());
          const controller = new AbortController();
          const timeout = setTimeout(() => controller.abort(), 5000);
          
          const response = await fetch(url, { signal: controller.signal });
          clearTimeout(timeout);
          
          if (!response.ok) {
            throw new Error(`Health check failed with status ${response.status}`);
          }
          
          // Reset restart count on successful health check
          if (info.restartCount > 0) {
            info.restartCount = 0;
          }
          
        } catch (error: any) {
          logger.error(`[ProcessManager] Health check failed for ${name}:`, error.message);
          this.emit('process:unhealthy', name, error);
          
          // Restart if health check fails
          if (config.autoRestart) {
            this.handleProcessCrash(name);
          }
        }
      }
    }, config.healthCheckInterval);
    
    this.healthCheckTimers.set(name, timer);
  }
  
  /**
   * Stop health checks for a process
   */
  private stopHealthCheck(name: string): void {
    const timer = this.healthCheckTimers.get(name);
    if (timer) {
      clearInterval(timer);
      this.healthCheckTimers.delete(name);
    }
  }
  
  /**
   * Get process status
   */
  getProcessStatus(name: string): ProcessInfo | undefined {
    return this.processes.get(name);
  }
  
  /**
   * Get all process statuses
   */
  getAllProcesses(): ProcessInfo[] {
    return Array.from(this.processes.values());
  }
  
  /**
   * Get comprehensive status report for all processes
   */
  getFullStatus(): {
    processes: Array<{
      name: string;
      status: string;
      pid?: number;
      port?: number;
      uptime?: number;
      restartCount: number;
      lastError?: string;
      isPortListening?: boolean;
    }>;
    allocatedPorts: Map<string, number>;
    summary: {
      total: number;
      running: number;
      stopped: number;
      crashed: number;
      starting: number;
    };
  } {
    const processes = Array.from(this.processes.values()).map(info => ({
      name: info.name,
      status: info.status,
      pid: info.pid,
      port: info.port,
      uptime: info.lastStartTime ? Date.now() - info.lastStartTime.getTime() : undefined,
      restartCount: info.restartCount,
      lastError: info.lastError,
      isPortListening: info.port ? !PortManager.isPortAvailable(info.port) : undefined
    }));
    
    const summary = {
      total: processes.length,
      running: processes.filter(p => p.status === 'running').length,
      stopped: processes.filter(p => p.status === 'stopped').length,
      crashed: processes.filter(p => p.status === 'crashed').length,
      starting: processes.filter(p => p.status === 'starting').length
    };
    
    return {
      processes,
      allocatedPorts: PortManager.getAllocatedPorts(),
      summary
    };
  }
  
  /**
   * Get detailed debug information for a specific process
   */
  async debugProcess(name: string): Promise<{
    config?: ProcessConfig;
    info?: ProcessInfo;
    portStatus?: {
      allocated: boolean;
      port?: number;
      isListening: boolean;
      canConnect: boolean;
    };
    healthCheck?: {
      url?: string;
      status: 'healthy' | 'unhealthy' | 'not-configured';
      lastCheck?: Date;
      error?: string;
    };
  }> {
    const config = this.configs.get(name);
    const info = this.processes.get(name);
    
    let portStatus;
    if (info?.port) {
      const isAvailable = await PortManager.isPortAvailable(info.port);
      portStatus = {
        allocated: true,
        port: info.port,
        isListening: !isAvailable,
        canConnect: false
      };
      
      // Try to connect to the port
      try {
        const testConnection = await PortManager.waitForService(info.port, 100);
        portStatus.canConnect = testConnection;
      } catch {
        portStatus.canConnect = false;
      }
    }
    
    let healthCheck;
    if (config?.healthCheckUrl) {
      healthCheck = {
        url: config.healthCheckUrl,
        status: 'not-configured' as const,
        error: 'Health check not yet implemented'
      };
    }
    
    return {
      config,
      info,
      portStatus,
      healthCheck
    };
  }
  
  /**
   * Log detailed status to console
   */
  logStatus(): void {
    const status = this.getFullStatus();
    logger.info('\n[ProcessManager] === Status Report ===');
    logger.info(`Summary: ${status.summary.running}/${status.summary.total} running`);
    logger.info('Processes:');
    
    status.processes.forEach(p => {
      const uptimeStr = p.uptime ? `${Math.floor(p.uptime / 1000)}s` : 'N/A';
      const portStr = p.port ? `:${p.port}` : '';
      const pidStr = p.pid ? `PID:${p.pid}` : '';
      const errorStr = p.lastError ? ` [Error: ${p.lastError}]` : '';
      
      logger.info(`  - ${p.name}${portStr}: ${p.status} ${pidStr} (uptime: ${uptimeStr}, restarts: ${p.restartCount})${errorStr}`);
    });
    
    if (status.allocatedPorts.size > 0) {
      logger.info('\nAllocated Ports:');
      status.allocatedPorts.forEach((port, service) => {
        logger.info(`  - ${service}: ${port}`);
      });
    }
    logger.info('==================\n');
  }
  
  /**
   * AI Helper Monitoring System
   * Validates Python subprocess spawning and health for consensus routing
   */
  private async validateAIHelpers(processName: string): Promise<{valid: boolean, issue?: string}> {
    // Only check for websocket-backend process
    if (processName !== 'websocket-backend') {
      return {valid: true};
    }

    logger.info('[ProcessManager] Validating AI Helpers for consensus routing...');
    
    const info = this.processes.get(processName);
    const config = this.configs.get(processName);
    
    if (!info || !config) {
      return {valid: false, issue: 'Process not registered'};
    }

    // Check 1: Verify environment variables are set
    const env = config.env || {};
    const pythonPath = env.HIVE_BUNDLED_PYTHON;
    const modelScript = env.HIVE_BUNDLED_MODEL_SCRIPT;
    
    if (!pythonPath || !modelScript) {
      const issue = 'Missing Python environment variables';
      logger.error(`[ProcessManager] AI Helper validation failed: ${issue}`);
      logger.error(`[ProcessManager] HIVE_BUNDLED_PYTHON: ${pythonPath || 'NOT SET'}`);
      logger.error(`[ProcessManager] HIVE_BUNDLED_MODEL_SCRIPT: ${modelScript || 'NOT SET'}`);
      this.emit('aihelper:error', {
        process: processName,
        issue,
        pythonPath,
        modelScript
      });
      return {valid: false, issue};
    }

    // Check 2: Verify Python runtime exists
    if (!fs.existsSync(pythonPath)) {
      const issue = `Python runtime not found at: ${pythonPath}`;
      logger.error(`[ProcessManager] AI Helper validation failed: ${issue}`);
      this.emit('aihelper:error', {
        process: processName,
        issue,
        pythonPath
      });
      return {valid: false, issue};
    }

    // Check 3: Verify model script exists and is correct one
    if (!fs.existsSync(modelScript)) {
      const issue = `Model script not found at: ${modelScript}`;
      logger.error(`[ProcessManager] AI Helper validation failed: ${issue}`);
      this.emit('aihelper:error', {
        process: processName,
        issue,
        modelScript
      });
      return {valid: false, issue};
    }

    // Check 4: Verify model script is wrapper (not base model_service.py)
    if (!modelScript.includes('model_service_wrapper.py')) {
      const issue = `Wrong model script: using ${path.basename(modelScript)} instead of model_service_wrapper.py`;
      logger.error(`[ProcessManager] AI Helper validation failed: ${issue}`);
      logger.error(`[ProcessManager] CRITICAL: Backend needs model_service_wrapper.py for subprocess communication`);
      this.emit('aihelper:error', {
        process: processName,
        issue,
        modelScript,
        required: 'model_service_wrapper.py'
      });
      return {valid: false, issue};
    }

    // Check 5: Test Python subprocess spawning capability
    const { execSync } = require('child_process');
    try {
      // Quick test to ensure Python can be executed
      const testResult = execSync(`"${pythonPath}" --version`, {
        encoding: 'utf8',
        timeout: 5000
      });
      logger.info(`[ProcessManager] Python runtime validated: ${testResult.trim()}`);
    } catch (error: any) {
      const issue = `Python runtime cannot be executed: ${error.message}`;
      logger.error(`[ProcessManager] AI Helper validation failed: ${issue}`);
      this.emit('aihelper:error', {
        process: processName,
        issue,
        pythonPath,
        error: error.message
      });
      return {valid: false, issue};
    }

    // Check 6: Monitor for Python subprocess (give backend time to spawn it)
    logger.info('[ProcessManager] Waiting for Python AI Helper subprocess to spawn...');
    
    // Wait up to 5 seconds for Python subprocess to appear
    const startTime = Date.now();
    const maxWaitTime = 5000;
    let pythonFound = false;
    
    while (Date.now() - startTime < maxWaitTime && !pythonFound) {
      try {
        // Check for Python process with model_service in command
        const psResult = execSync('ps aux | grep -E "model_service|python.*hive" | grep -v grep', {
          encoding: 'utf8'
        });
        
        if (psResult && psResult.trim()) {
          pythonFound = true;
          logger.info('[ProcessManager] Python AI Helper subprocess detected:');
          logger.info(psResult.trim());
          this.emit('aihelper:ready', {
            process: processName,
            pythonProcess: psResult.trim()
          });
        }
      } catch {
        // grep returns non-zero if no match, that's ok
      }
      
      if (!pythonFound) {
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    }

    if (!pythonFound) {
      // Check if backend is actually running and healthy
      const backendPort = info.port;
      if (backendPort) {
        try {
          const healthCheck = execSync(`curl -s http://localhost:${backendPort}/health`, {
            encoding: 'utf8',
            timeout: 2000
          });
          
          const health = JSON.parse(healthCheck);
          if (health.features?.ai_helpers) {
            // Backend thinks AI Helpers are ready but Python not detected
            const issue = 'Backend reports AI Helpers ready but Python subprocess not detected';
            logger.warn(`[ProcessManager] AI Helper warning: ${issue}`);
            logger.warn('[ProcessManager] Consensus routing may fail without Python subprocess');
            
            // Test consensus endpoint to confirm
            try {
              const consensusTest = execSync(
                `curl -s -X POST http://localhost:${backendPort}/api/consensus ` +
                `-H "Content-Type: application/json" ` +
                `-d '{"query":"test","profile":"speed","stream":false}' --max-time 3`,
                { encoding: 'utf8' }
              );
              
              if (consensusTest.includes('AI Helpers required')) {
                logger.error('[ProcessManager] CONFIRMED: Consensus routing broken - AI Helpers not functional');
                this.emit('aihelper:error', {
                  process: processName,
                  issue: 'Consensus routing broken - Python subprocess not running',
                  backendHealth: health,
                  consensusResponse: consensusTest
                });
                return {valid: false, issue: 'Python subprocess not running despite backend ready'};
              }
            } catch (e) {
              logger.error('[ProcessManager] Consensus endpoint test failed:', e);
            }
          }
        } catch (e) {
          logger.error('[ProcessManager] Backend health check failed:', e);
        }
      }
      
      const issue = 'Python AI Helper subprocess did not start within timeout';
      logger.warn(`[ProcessManager] AI Helper warning: ${issue}`);
      this.emit('aihelper:warning', {
        process: processName,
        issue,
        waitedMs: maxWaitTime
      });
      // Don't fail completely, backend might work without it for some operations
    }

    logger.info('[ProcessManager] AI Helper validation completed');
    return {valid: true};
  }

  /**
   * Monitor AI Helper health continuously
   */
  private startAIHelperMonitoring(processName: string): void {
    if (processName !== 'websocket-backend') {
      return;
    }

    // Clear any existing monitor
    const existingTimer = this.healthCheckTimers.get(`${processName}-aihelper`);
    if (existingTimer) {
      clearInterval(existingTimer);
    }

    // Check every 30 seconds
    const timer = setInterval(async () => {
      const result = await this.validateAIHelpers(processName);
      if (!result.valid) {
        logger.error(`[ProcessManager] AI Helper health check failed: ${result.issue}`);
        
        // Report to monitoring system
        this.emit('health:failed', {
          process: processName,
          component: 'ai-helpers',
          issue: result.issue,
          timestamp: new Date().toISOString()
        });
        
        // Try to recover by restarting backend if consensus is broken
        if (result.issue?.includes('Consensus routing broken')) {
          logger.warn('[ProcessManager] Attempting to recover by restarting backend...');
          await this.restartProcess(processName);
        }
      }
    }, 30000);

    this.healthCheckTimers.set(`${processName}-aihelper`, timer);
    logger.info(`[ProcessManager] Started AI Helper monitoring for ${processName}`);
  }

  /**
   * Set up shutdown handlers
   */
  private setupShutdownHandlers(): void {
    const shutdown = async () => {
      if (this.shutdownInProgress) {
        return;
      }
      
      this.shutdownInProgress = true;
      logger.info('[ProcessManager] Shutting down all processes...');
      
      // Stop all processes
      const stopPromises = Array.from(this.processes.keys()).map(name => 
        this.stopProcess(name, false)
      );
      
      await Promise.all(stopPromises);
      
      // Clean up ports
      await PortManager.cleanup();
      
      logger.info('[ProcessManager] All processes stopped');
    };
    
    // Handle various shutdown signals
    process.on('SIGTERM', shutdown);
    process.on('SIGINT', shutdown);
    process.on('beforeExit', shutdown);
  }
  
  /**
   * Clean up all processes
   */
  async cleanup(): Promise<void> {
    this.shutdownInProgress = true;
    
    // Stop all health checks
    for (const timer of this.healthCheckTimers.values()) {
      clearInterval(timer);
    }
    this.healthCheckTimers.clear();
    
    // Stop all processes
    for (const name of this.processes.keys()) {
      await this.stopProcess(name, true);
    }
    
    // Clean up ports
    await PortManager.cleanup();
    
    this.processes.clear();
    this.configs.clear();
  }
}

export default ProcessManager;
