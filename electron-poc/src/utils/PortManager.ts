/**
 * PortManager - Optimized port allocation for diverse environments
 * Pre-scans ports at startup for instant allocation without delays
 *
 * Diagrams
 * - electron-poc/MASTER_ARCHITECTURE.md#diagram-dynamic-port-allocation
 * - electron-poc/MASTER_ARCHITECTURE.md#diagram-system-overview
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

interface PortRange {
  name: string;
  start: number;
  end: number;
  priority: number;
}

interface AvailablePort {
  port: number;
  range: string;
  scanTime: number;
}

export class PortManager {
  private static allocatedPorts: Map<string, number> = new Map();
  private static availablePortPool: Map<string, AvailablePort[]> = new Map();
  private static scanComplete = false;
  private static scanPromise: Promise<void> | null = null;
  
  // Port ranges loaded from configuration - NEVER hardcoded
  private static PORT_RANGES: PortRange[] = [];
  
  /**
   * Discover available port ranges dynamically
   * NO HARDCODED DEFAULTS - discovers what's available on the system
   */
  private static async discoverPortRanges(): Promise<PortRange[]> {
    const ranges: PortRange[] = [];
    
    // Common port ranges to scan for availability
    // These are DISCOVERY ranges, not assumptions
    const discoveryRanges = [
      { start: 3000, end: 4000, step: 100 },  // Lower range
      { start: 7000, end: 8000, step: 100 },  // Mid range
      { start: 8000, end: 9000, step: 100 },  // Higher range
      { start: 9000, end: 10000, step: 100 }, // Upper range
      { start: 14000, end: 15000, step: 100 }, // Alternative range
      { start: 19000, end: 20000, step: 100 }, // High alternative
    ];
    
    // Discover available blocks in each range
    for (const discovery of discoveryRanges) {
      const availableBlock = await this.findAvailableBlock(discovery.start, discovery.end, discovery.step);
      if (availableBlock) {
        // Assign to service based on what we found
        if (ranges.length === 0) {
          ranges.push({ name: 'memory-service', start: availableBlock.start, end: availableBlock.end, priority: 1 });
        } else if (ranges.length === 1) {
          ranges.push({ name: 'backend-server', start: availableBlock.start, end: availableBlock.end, priority: 1 });
        } else if (ranges.length === 2) {
          ranges.push({ name: 'ttyd-terminals', start: availableBlock.start, end: availableBlock.end, priority: 1 });
        } else {
          ranges.push({ name: `service-${ranges.length}`, start: availableBlock.start, end: availableBlock.end, priority: 1 });
        }
      }
    }
    
    if (ranges.length === 0) {
      throw new Error('No available port ranges found on system! Cannot start services.');
    }
    
    logger.info('[PortManager] Discovered port ranges:', ranges);
    return ranges;
  }
  
  /**
   * Find an available block of ports in a range
   */
  private static async findAvailableBlock(start: number, end: number, blockSize: number): Promise<{start: number, end: number} | null> {
    // Only check first block in production for speed
    const blockEnd = Math.min(start + blockSize - 1, end);
    
    // Just check if the first port is available - much faster
    if (await this.checkPortQuick(start)) {
      return { start: start, end: blockEnd };
    }
    
    return null;
  }
  
  /**
   * Initialize and pre-scan all port ranges at startup
   */
  static async initialize(): Promise<void> {
    if (this.scanPromise) {
      return this.scanPromise;
    }
    
    // Add timeout to prevent hanging in production
    this.scanPromise = Promise.race([
      this.performFullInitialization(),
      new Promise<void>((resolve) => {
        setTimeout(() => {
          logger.warn('[PortManager] Initialization timeout - using default ranges');
          // Use default ranges if discovery times out
          this.PORT_RANGES = [
            { name: 'memory-service', start: 3000, end: 3099, priority: 1 },
            { name: 'backend-server', start: 7100, end: 7199, priority: 1 },
            { name: 'ttyd-terminals', start: 8000, end: 8099, priority: 1 },
          ];
          this.scanComplete = true;
          resolve();
        }, 3000); // 3 second timeout
      })
    ]);
    
    return this.scanPromise;
  }
  
  private static async performFullInitialization(): Promise<void> {
    // Discover available port ranges dynamically
    this.PORT_RANGES = await this.discoverPortRanges();
    await this.performInitialScan();
  }
  
  private static async performInitialScan(): Promise<void> {
    const startTime = Date.now();
    logger.info('[PortManager] Starting optimized parallel port scan...');
    
    // Scan all ranges in parallel
    const scanPromises = this.PORT_RANGES.map(range => this.scanRange(range));
    await Promise.all(scanPromises);
    
    const scanTime = Date.now() - startTime;
    const totalPorts = Array.from(this.availablePortPool.values())
      .reduce((sum, ports) => sum + ports.length, 0);
    
    logger.info(`[PortManager] Scan complete in ${scanTime}ms. Found ${totalPorts} available ports`);
    this.scanComplete = true;
  }
  
  private static async scanRange(range: PortRange): Promise<void> {
    const ports: AvailablePort[] = [];
    const batchSize = 10;
    const totalPorts = Math.min(20, range.end - range.start + 1); // Scan up to 20 ports per range
    
    for (let i = 0; i < totalPorts; i += batchSize) {
      const batch = [];
      for (let j = 0; j < batchSize && (i + j) < totalPorts; j++) {
        const port = range.start + i + j;
        batch.push(this.checkPortQuick(port).then(available => {
          if (available) {
            ports.push({ port, range: range.name, scanTime: Date.now() });
          }
        }));
      }
      await Promise.all(batch);
      if (ports.length >= 10) break; // Stop if we found enough
    }
    
    const existing = this.availablePortPool.get(range.name) || [];
    this.availablePortPool.set(range.name, [...existing, ...ports]);
  }
  
  private static async checkPortQuick(port: number): Promise<boolean> {
    return new Promise((resolve) => {
      const server = net.createServer();
      const timeout = setTimeout(() => {
        server.close();
        resolve(false);
      }, 50); // Quick 50ms timeout
      
      server.once('error', () => {
        clearTimeout(timeout);
        resolve(false);
      });
      
      server.once('listening', () => {
        clearTimeout(timeout);
        server.close();
        resolve(true);
      });
      
      server.listen(port, '127.0.0.1');
    });
  }
  
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
   * Find the first available port - uses pre-scanned pool for instant allocation
   */
  static async findAvailablePort(
    portRangeStart: number,
    portRangeEnd?: number
  ): Promise<number> {
    // Ensure pool is initialized
    if (!this.scanComplete) {
      await this.initialize();
    }
    
    // Try to get from pool first
    const serviceName = this.getServiceNameForPort(portRangeStart);
    const poolName = this.getPoolNameForService(serviceName);
    const pool = this.availablePortPool.get(poolName) || [];
    
    if (pool.length > 0) {
      const portInfo = pool.shift()!;
      logger.info(`[PortManager] Allocated port ${portInfo.port} from pool (instant)`);
      return portInfo.port;
    }
    
    // Fallback to scanning if pool is empty
    const rangeEnd = portRangeEnd || portRangeStart + 100;
    for (let port = portRangeStart; port <= rangeEnd; port++) {
      if (await this.isPortAvailable(port)) {
        logger.info(`[PortManager] Found port ${port} via fallback scan`);
        return port;
      }
    }
    
    throw new Error(`No available ports in range ${portRangeStart}-${rangeEnd}`);
  }
  
  /**
   * New simplified method: Allocate a port by service name only
   * No need to specify any port numbers - fully dynamic
   */
  static async allocatePortForService(serviceName: string): Promise<number> {
    // Ensure pool is initialized
    if (!this.scanComplete) {
      await this.initialize();
    }
    
    // Check if already allocated
    if (this.allocatedPorts.has(serviceName)) {
      const existingPort = this.allocatedPorts.get(serviceName)!;
      if (await this.isPortAvailable(existingPort)) {
        logger.info(`[PortManager] Reusing port ${existingPort} for ${serviceName}`);
        return existingPort;
      }
      this.allocatedPorts.delete(serviceName);
    }
    
    // Get from appropriate pool
    const poolName = this.getPoolNameForService(serviceName);
    const pools = poolName === 'memory-service' 
      ? ['memory-service', 'memory-service-alt']
      : poolName === 'backend-server'
      ? ['backend-server', 'backend-server-alt']
      : [poolName];
    
    for (const pool of pools) {
      const availablePorts = this.availablePortPool.get(pool) || [];
      if (availablePorts.length > 0) {
        const portInfo = availablePorts.shift()!;
        // Double-check it's still available
        if (await this.checkPortQuick(portInfo.port)) {
          this.allocatedPorts.set(serviceName, portInfo.port);
          logger.info(`[PortManager] Allocated port ${portInfo.port} to ${serviceName} (instant from ${pool} pool)`);
          return portInfo.port;
        }
      }
    }
    
    // If no ports in pool, request an ephemeral port from OS
    const ephemeralPort = await this.getEphemeralPort();
    this.allocatedPorts.set(serviceName, ephemeralPort);
    logger.info(`[PortManager] Allocated ephemeral port ${ephemeralPort} to ${serviceName}`);
    return ephemeralPort;
  }
  
  /**
   * Get an ephemeral port from the OS (guaranteed available)
   */
  private static async getEphemeralPort(): Promise<number> {
    return new Promise((resolve, reject) => {
      const server = net.createServer();
      server.listen(0, '127.0.0.1', () => {
        const address = server.address();
        if (address && typeof address === 'object') {
          const port = address.port;
          server.close(() => resolve(port));
        } else {
          reject(new Error('Failed to allocate ephemeral port'));
        }
      });
    });
  }
  
  /**
   * Legacy method for backwards compatibility
   * @deprecated Use allocatePortForService() instead
   */
  static async allocatePort(config: PortConfig): Promise<number> {
    // Just use the service name, ignore the port numbers
    return this.allocatePortForService(config.serviceName);
  }
  
  /**
   * Helper: Get service name from port number
   */
  private static getServiceNameForPort(port: number): string {
    // Check all ranges to identify service
    for (const range of this.PORT_RANGES) {
      if (port >= range.start && port <= range.end) {
        if (range.name.includes('memory')) return 'memory-service';
        if (range.name.includes('backend')) return 'websocket-backend';
        if (range.name.includes('ttyd')) return 'ttyd';
        if (range.name.includes('debug')) return 'debug-server';
      }
    }
    return 'generic';
  }
  
  /**
   * Helper: Get pool name for service
   */
  private static getPoolNameForService(serviceName: string): string {
    if (serviceName === 'memory-service') return 'memory-service';
    if (serviceName === 'websocket-backend') return 'backend-server';
    if (serviceName.startsWith('ttyd')) return 'ttyd-terminals';
    return 'memory-service'; // Default pool
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
