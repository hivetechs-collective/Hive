/**
 * PortManager - Handles port allocation and conflicts for production services
 * Ensures clean startup and shutdown of services
 */

import * as net from 'net';
import { exec, execSync } from 'child_process';
import { promisify } from 'util';

import { logger } from './SafeLogger';
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
      
      // Check on all interfaces (0.0.0.0) to catch servers listening on any interface
      server.listen(port, '0.0.0.0');
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
        logger.info(`[PortManager] Killing processes on port ${port}: ${pids.join(', ')}`);
        
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
              logger.error(`[PortManager] Error killing process ${pid}:`, error.message);
            }
          }
        }
        
        // Wait a bit for port to be released
        await new Promise(resolve => setTimeout(resolve, 500));
        return true;
      }
      
      return false;
    } catch (error) {
      logger.error('[PortManager] Error finding process on port:', error);
      return false;
    }
  }
  
  /**
   * Allocate a port for a service with automatic conflict resolution
   */
  static async allocatePort(config: PortConfig): Promise<number> {
    const { port, serviceName, alternativePorts } = config;
    
    // Check if service already has an allocated port that's still free
    if (this.allocatedPorts.has(serviceName)) {
      const existingPort = this.allocatedPorts.get(serviceName)!;
      if (await this.isPortAvailable(existingPort)) {
        logger.info(`[PortManager] Reusing existing port ${existingPort} for ${serviceName}`);
        return existingPort;
      }
      // Release the old allocation since it's no longer valid
      this.allocatedPorts.delete(serviceName);
    }
    
    // Start with preferred port
    let currentPort = port;
    let portToUse: number | null = null;
    
    // Check if preferred port is already allocated to another service
    const isPortAllocatedToAnother = Array.from(this.allocatedPorts.values()).includes(currentPort);
    
    // Check preferred port first
    if (!isPortAllocatedToAnother && await this.isPortAvailable(currentPort)) {
      portToUse = currentPort;
      logger.info(`[PortManager] Port ${currentPort} is available for ${serviceName}`);
    } else {
      if (isPortAllocatedToAnother) {
        logger.info(`[PortManager] Port ${currentPort} is allocated to another service, finding next available port...`);
      } else {
        logger.info(`[PortManager] Port ${currentPort} is in use, finding next available port...`);
      }
      
      // Try alternative ports if provided
      if (alternativePorts && alternativePorts.length > 0) {
        for (const altPort of alternativePorts) {
          // Check if this port is allocated to another service
          const isAltPortAllocated = Array.from(this.allocatedPorts.values()).includes(altPort);
          if (!isAltPortAllocated && await this.isPortAvailable(altPort)) {
            portToUse = altPort;
            logger.info(`[PortManager] Using alternative port ${altPort} for ${serviceName}`);
            break;
          }
        }
      }
      
      // If still no port, scan for the next available port
      if (!portToUse) {
        currentPort = port + 1;
        const maxPort = port + 100; // Search up to 100 ports ahead
        
        while (currentPort < maxPort) {
          // Check if this port is allocated to another service
          const isCurrentPortAllocated = Array.from(this.allocatedPorts.values()).includes(currentPort);
          if (!isCurrentPortAllocated && await this.isPortAvailable(currentPort)) {
            portToUse = currentPort;
            logger.info(`[PortManager] Found available port ${currentPort} for ${serviceName}`);
            break;
          }
          currentPort++;
        }
      }
    }
    
    if (!portToUse) {
      // This should never happen unless all 100 ports are taken
      throw new Error(`Could not find any available port for ${serviceName} (searched ${port} to ${port + 100})`);
    }
    
    // Allocate the port
    this.allocatedPorts.set(serviceName, portToUse);
    logger.info(`[PortManager] ✅ Port ${portToUse} allocated for ${serviceName}`);
    return portToUse;
  }
  
  /**
   * Release a port allocation
   */
  static releasePort(serviceName: string): void {
    if (this.allocatedPorts.has(serviceName)) {
      const port = this.allocatedPorts.get(serviceName)!;
      this.allocatedPorts.delete(serviceName);
      logger.info(`[PortManager] Released port ${port} for ${serviceName}`);
    }
  }
  
  /**
   * Clean up all allocated ports
   */
  static async cleanup(): Promise<void> {
    logger.info('[PortManager] Cleaning up all allocated ports...');
    
    for (const [serviceName, port] of this.allocatedPorts) {
      try {
        await this.killProcessOnPort(port);
        logger.info(`[PortManager] Cleaned up port ${port} for ${serviceName}`);
      } catch (error) {
        logger.error(`[PortManager] Error cleaning up ${serviceName}:`, error);
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
        // Try to connect to the port to see if something is listening
        const isListening = await this.isPortListening(port);
        if (isListening) {
          // Port is listening, service is likely ready
          // Try to make a health check request
          try {
            const response = await fetch(`http://localhost:${port}/health`);
            if (response.ok) {
              return true;
            }
          } catch {
            // Service might not have HTTP endpoint yet, but port is listening
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

  /**
   * Check if a port is listening by trying to connect to it
   */
  private static async isPortListening(port: number): Promise<boolean> {
    return new Promise((resolve) => {
      const client = new net.Socket();
      
      const timeout = setTimeout(() => {
        client.destroy();
        resolve(false);
      }, 100);
      
      client.once('connect', () => {
        clearTimeout(timeout);
        client.destroy();
        resolve(true);
      });
      
      client.once('error', () => {
        clearTimeout(timeout);
        resolve(false);
      });
      
      // Try to connect to localhost on the port
      client.connect(port, 'localhost');
    });
  }
}

export default PortManager;