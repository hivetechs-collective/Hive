/**
 * ProcessManager - Manages child processes lifecycle for production
 * Handles spawning, monitoring, restarting, and cleanup
 */

import { ChildProcess, fork, spawn } from 'child_process';
import { EventEmitter } from 'events';
import path from 'path';
import { PortManager } from './PortManager';

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
    
    console.log(`[ProcessManager] Registered process: ${config.name}`);
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
      console.log(`[ProcessManager] Process ${name} is already running`);
      return true;
    }
    
    info.status = 'starting';
    this.emit('process:starting', name);
    
    try {
      // Allocate port if needed
      let port = config.port;
      if (port) {
        port = await PortManager.allocatePort({
          port,
          serviceName: name,
          alternativePorts: config.alternativePorts
        });
        info.port = port;
      }
      
      // Prepare environment
      const env = {
        ...process.env,
        ...config.env,
        ...(port ? { PORT: port.toString(), MEMORY_SERVICE_PORT: port.toString() } : {})
      };
      
      console.log(`[ProcessManager] Starting ${name} on port ${port || 'N/A'}`);
      
      // Spawn the process - handle TypeScript files with ts-node
      let childProcess: ChildProcess;
      
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
      } else {
        // For JavaScript files, use fork normally
        childProcess = fork(config.scriptPath, config.args || [], {
          env,
          silent: false,
          detached: false
        });
      }
      
      info.process = childProcess;
      info.pid = childProcess.pid;
      info.lastStartTime = new Date();
      
      // Set up event handlers
      childProcess.on('message', (msg: any) => {
        this.handleProcessMessage(name, msg);
      });
      
      childProcess.on('error', (error) => {
        console.error(`[ProcessManager] Process ${name} error:`, error);
        info.lastError = error.message;
        this.handleProcessCrash(name);
      });
      
      childProcess.on('exit', (code, signal) => {
        console.log(`[ProcessManager] Process ${name} exited with code ${code}, signal ${signal}`);
        
        if (!this.shutdownInProgress && info.status !== 'stopping') {
          this.handleProcessCrash(name);
        } else {
          info.status = 'stopped';
          this.emit('process:stopped', name);
        }
      });
      
      // Wait for process to be ready - check for 'ready' message instead of port binding
      // since the process uses IPC to signal readiness
      const readyPromise = new Promise<boolean>((resolve) => {
        const timeout = setTimeout(() => {
          console.log(`[ProcessManager] Timeout waiting for ${name} ready signal`);
          resolve(false);
        }, 10000);
        
        const messageHandler = (msg: any) => {
          if (msg.type === 'ready') {
            clearTimeout(timeout);
            resolve(true);
          }
        };
        
        childProcess.once('message', messageHandler);
      });
      
      const isReady = await readyPromise;
      if (!isReady && port) {
        // Fallback to port check if no ready message
        const portReady = await PortManager.waitForService(port, 5000);
        if (!portReady) {
          throw new Error(`Process ${name} failed to start properly`);
        }
      }
      
      info.status = 'running';
      this.emit('process:started', name);
      
      // Start health checks if configured
      if (config.healthCheckUrl && config.healthCheckInterval) {
        this.startHealthCheck(name);
      }
      
      console.log(`[ProcessManager] Process ${name} started successfully (PID: ${info.pid})`);
      return true;
      
    } catch (error: any) {
      console.error(`[ProcessManager] Failed to start ${name}:`, error.message);
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
        console.error(`[ProcessManager] Error stopping ${name}:`, error);
      }
    }
    
    // Release port
    if (info.port) {
      PortManager.releasePort(name);
      info.port = undefined;
    }
    
    info.status = 'stopped';
    this.emit('process:stopped', name);
    
    console.log(`[ProcessManager] Process ${name} stopped`);
    return true;
  }
  
  /**
   * Restart a process
   */
  async restartProcess(name: string): Promise<boolean> {
    console.log(`[ProcessManager] Restarting ${name}...`);
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
    
    // Stop health checks
    this.stopHealthCheck(name);
    
    // Check if auto-restart is enabled
    if (config?.autoRestart && !this.shutdownInProgress) {
      const maxRestarts = config.maxRestarts || 5;
      
      if (info.restartCount < maxRestarts) {
        info.restartCount++;
        const delay = config.restartDelay || 3000;
        
        console.log(`[ProcessManager] Auto-restarting ${name} in ${delay}ms (attempt ${info.restartCount}/${maxRestarts})`);
        
        setTimeout(async () => {
          if (!this.shutdownInProgress) {
            const success = await this.startProcess(name);
            if (!success) {
              console.error(`[ProcessManager] Failed to restart ${name}`);
            }
          }
        }, delay);
      } else {
        console.error(`[ProcessManager] Process ${name} exceeded max restart attempts`);
        this.emit('process:failed', name, new Error('Max restarts exceeded'));
      }
    }
  }
  
  /**
   * Handle messages from child processes
   */
  private handleProcessMessage(name: string, message: any): void {
    console.log(`[ProcessManager] Message from ${name}:`, message);
    
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
          console.error(`[ProcessManager] Health check failed for ${name}:`, error.message);
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
   * Set up shutdown handlers
   */
  private setupShutdownHandlers(): void {
    const shutdown = async () => {
      if (this.shutdownInProgress) {
        return;
      }
      
      this.shutdownInProgress = true;
      console.log('[ProcessManager] Shutting down all processes...');
      
      // Stop all processes
      const stopPromises = Array.from(this.processes.keys()).map(name => 
        this.stopProcess(name, false)
      );
      
      await Promise.all(stopPromises);
      
      // Clean up ports
      await PortManager.cleanup();
      
      console.log('[ProcessManager] All processes stopped');
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