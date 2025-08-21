/**
 * ProcessManager - Manages child processes lifecycle for production
 * Handles spawning, monitoring, restarting, and cleanup
 */

import { ChildProcess, fork, spawn } from 'child_process';
import { EventEmitter } from 'events';
import path from 'path';
import { PortManager } from './PortManager';

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
      // Allocate port if needed - PortManager will find an available port
      let port = config.port;
      if (port) {
        // PortManager will intelligently find an available port
        port = await PortManager.allocatePort({
          port,
          serviceName: name,
          alternativePorts: config.alternativePorts
        });
        info.port = port;
        logger.info(`[ProcessManager] ${name} will use port ${port}`);
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
        // For TypeScript files, we need to use fork with ts-node to get IPC
        // Create a wrapper that uses ts-node/register
        const tsNodePath = require.resolve('ts-node/register');
        childProcess = fork(config.scriptPath, config.args || [], {
          env,
          silent: false,
          detached: false,
          execArgv: ['-r', tsNodePath]
        });
      } else if (config.scriptPath.endsWith('.js')) {
        // For JavaScript files, use fork normally
        childProcess = fork(config.scriptPath, config.args || [], {
          env,
          silent: false,
          detached: false
        });
      } else {
        // For binary executables (Rust, Go, etc.), use spawn
        logger.info(`[ProcessManager] Spawning binary executable: ${config.scriptPath}`);
        // Use 'inherit' for stdio to allow subprocess communication (e.g., Python processes spawned by AI Helpers)
        // CRITICAL: AI Helpers spawn Python subprocesses that require full stdio access
        childProcess = spawn(config.scriptPath, config.args || [], {
          env,
          stdio: 'inherit',  // Allow full stdio inheritance for subprocess communication
          detached: false
        });
        
        // With 'inherit' stdio, we can't capture output, so we won't have a binaryReadyPromise
        // We'll rely solely on port checking for readiness detection
        logger.info(`[ProcessManager] Binary process ${name} spawned with inherited stdio`);
        logger.info(`[ProcessManager] Will use port checking for readiness (port ${port})`);
        
        // Note: childProcess.stdout and childProcess.stderr are null with 'inherit'
        // These blocks won't execute with 'inherit' stdio, but keeping them for future reference
        // if we need to switch back to captured stdio for debugging
      }
      
      info.process = childProcess;
      info.pid = childProcess.pid;
      info.lastStartTime = new Date();
      
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
        // Handle ready message first if we're waiting for it
        if (readyResolver && msg.type === 'ready') {
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
        isReady = await readyPromise;
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
        
        // Binary servers may take longer to bind to port after process starts
        // AI Helpers initialization can take time, so give them enough time to start
        const isBinary = !config.scriptPath.endsWith('.ts') && !config.scriptPath.endsWith('.js');
        
        if (isBinary) {
          // For binary processes, add initial delay to allow process to initialize
          logger.info(`[ProcessManager] Waiting 2 seconds for ${name} to initialize before port check...`);
          await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        // Fast, efficient port checking (2025 best practice)
        const maxWaitTime = isBinary ? 15000 : 3000; // 15s for binaries, 3s for Node.js
        const checkInterval = 250; // Check every 250ms
        const maxAttempts = Math.floor(maxWaitTime / checkInterval);
        
        let attempts = 0;
        let portReady = false;
        
        // Simple, fast checking - no exponential backoff
        while (attempts < maxAttempts && !portReady) {
          attempts++;
          
          // Quick check if port is listening
          portReady = await PortManager.waitForService(port, checkInterval);
          
          if (portReady) {
            logger.info(`[ProcessManager] ✅ Port ${port} is ready for ${name} (${attempts * checkInterval}ms)`);
            break;
          }
          
          // Only log occasionally to reduce noise
          if (attempts === maxAttempts / 2) {
            logger.info(`[ProcessManager] Waiting for ${name} on port ${port}...`);
          }
        }
        
        if (!portReady) {
          // Try to get more debug info before failing
          const debugInfo = await this.debugProcess(name);
          logger.error(`[ProcessManager] Debug info for ${name}:`, JSON.stringify(debugInfo, null, 2));
          
          throw new Error(`Process ${name} failed to start properly - port ${port} not responding after ${attempts} attempts (${maxWaitTime}ms)`);
        }
        
        isReady = true;
      }
      
      info.status = 'running';
      this.emit('process:started', name);
      
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