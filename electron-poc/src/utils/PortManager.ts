/**
 * PortManager - Handles port allocation and conflicts for production services
 * Ensures clean startup and shutdown of services
 */

import * as net from 'net';
import { exec, execSync } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export interface PortConfig {
  port: number;
  serviceName: string;
  retryCount?: number;
  alternativePorts?: number[];
}

export class PortManager {
  private static allocatedPorts: Map<string, number> = new Map();
  
  /**
   * Check if a port is available
   */
  static async isPortAvailable(port: number): Promise<boolean> {
    return new Promise((resolve) => {
      const server = net.createServer();
      
      server.once('error', (err: any) => {
        if (err.code === 'EADDRINUSE') {
          resolve(false);
        } else {
          resolve(false);
        }
      });
      
      server.once('listening', () => {
        server.close();
        resolve(true);
      });
      
      server.listen(port, '127.0.0.1');
    });
  }
  
  /**
   * Find an available port starting from a preferred port
   */
  static async findAvailablePort(
    preferredPort: number,
    alternativePorts?: number[]
  ): Promise<number> {
    // Try preferred port first
    if (await this.isPortAvailable(preferredPort)) {
      return preferredPort;
    }
    
    // Try alternative ports
    if (alternativePorts) {
      for (const port of alternativePorts) {
        if (await this.isPortAvailable(port)) {
          return port;
        }
      }
    }
    
    // Find next available port
    for (let port = preferredPort + 1; port < preferredPort + 100; port++) {
      if (await this.isPortAvailable(port)) {
        return port;
      }
    }
    
    throw new Error(`No available ports found near ${preferredPort}`);
  }
  
  /**
   * Kill process using a specific port
   */
  static async killProcessOnPort(port: number): Promise<boolean> {
    try {
      // Find process ID using the port
      const { stdout } = await execAsync(
        `lsof -i :${port} -t 2>/dev/null || true`
      );
      
      const pids = stdout.trim().split('\n').filter(Boolean);
      
      if (pids.length > 0) {
        console.log(`[PortManager] Killing processes on port ${port}: ${pids.join(', ')}`);
        
        // Kill each process
        for (const pid of pids) {
          try {
            process.kill(parseInt(pid), 'SIGTERM');
            
            // Give it time to terminate gracefully
            await new Promise(resolve => setTimeout(resolve, 100));
            
            // Force kill if still running
            try {
              process.kill(parseInt(pid), 0); // Check if still alive
              process.kill(parseInt(pid), 'SIGKILL');
            } catch {
              // Process already terminated
            }
          } catch (error: any) {
            if (error.code !== 'ESRCH') { // Process doesn't exist
              console.error(`[PortManager] Error killing process ${pid}:`, error.message);
            }
          }
        }
        
        // Wait a bit for port to be released
        await new Promise(resolve => setTimeout(resolve, 500));
        return true;
      }
      
      return false;
    } catch (error) {
      console.error('[PortManager] Error finding process on port:', error);
      return false;
    }
  }
  
  /**
   * Allocate a port for a service with automatic conflict resolution
   */
  static async allocatePort(config: PortConfig): Promise<number> {
    const { port, serviceName, retryCount = 3, alternativePorts } = config;
    
    // Check if service already has an allocated port
    if (this.allocatedPorts.has(serviceName)) {
      const existingPort = this.allocatedPorts.get(serviceName)!;
      if (await this.isPortAvailable(existingPort)) {
        return existingPort;
      }
    }
    
    for (let attempt = 0; attempt < retryCount; attempt++) {
      try {
        // Check if port is available
        const isAvailable = await this.isPortAvailable(port);
        
        if (isAvailable) {
          this.allocatedPorts.set(serviceName, port);
          console.log(`[PortManager] Port ${port} allocated for ${serviceName}`);
          return port;
        }
        
        console.log(`[PortManager] Port ${port} is in use, attempting to clean up...`);
        
        // Try to kill the process using the port
        const killed = await this.killProcessOnPort(port);
        
        if (killed) {
          // Wait a bit more for port to be fully released
          await new Promise(resolve => setTimeout(resolve, 1000));
          
          // Check again
          if (await this.isPortAvailable(port)) {
            this.allocatedPorts.set(serviceName, port);
            console.log(`[PortManager] Port ${port} recovered for ${serviceName}`);
            return port;
          }
        }
        
        // If we can't use the preferred port, find an alternative
        if (attempt === retryCount - 1) {
          const availablePort = await this.findAvailablePort(port, alternativePorts);
          this.allocatedPorts.set(serviceName, availablePort);
          console.log(`[PortManager] Using alternative port ${availablePort} for ${serviceName}`);
          return availablePort;
        }
        
      } catch (error) {
        console.error(`[PortManager] Error on attempt ${attempt + 1}:`, error);
        if (attempt === retryCount - 1) {
          throw error;
        }
      }
      
      // Wait before retry
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    throw new Error(`Failed to allocate port for ${serviceName} after ${retryCount} attempts`);
  }
  
  /**
   * Release a port allocation
   */
  static releasePort(serviceName: string): void {
    if (this.allocatedPorts.has(serviceName)) {
      const port = this.allocatedPorts.get(serviceName)!;
      this.allocatedPorts.delete(serviceName);
      console.log(`[PortManager] Released port ${port} for ${serviceName}`);
    }
  }
  
  /**
   * Clean up all allocated ports
   */
  static async cleanup(): Promise<void> {
    console.log('[PortManager] Cleaning up all allocated ports...');
    
    for (const [serviceName, port] of this.allocatedPorts) {
      try {
        await this.killProcessOnPort(port);
        console.log(`[PortManager] Cleaned up port ${port} for ${serviceName}`);
      } catch (error) {
        console.error(`[PortManager] Error cleaning up ${serviceName}:`, error);
      }
    }
    
    this.allocatedPorts.clear();
  }
  
  /**
   * Get all allocated ports
   */
  static getAllocatedPorts(): Map<string, number> {
    return new Map(this.allocatedPorts);
  }
  
  /**
   * Wait for a service to be ready on a port
   */
  static async waitForService(
    port: number,
    timeout: number = 10000,
    checkInterval: number = 100
  ): Promise<boolean> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeout) {
      try {
        const isInUse = !(await this.isPortAvailable(port));
        if (isInUse) {
          // Port is in use, service is likely ready
          // Try to make a health check request
          try {
            const response = await fetch(`http://localhost:${port}/health`);
            if (response.ok) {
              return true;
            }
          } catch {
            // Service might not have HTTP endpoint yet
          }
          
          // Port is at least bound
          return true;
        }
      } catch {
        // Ignore errors during wait
      }
      
      await new Promise(resolve => setTimeout(resolve, checkInterval));
    }
    
    return false;
  }
}

export default PortManager;