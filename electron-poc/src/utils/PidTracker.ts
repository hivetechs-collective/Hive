/**
 * PidTracker - Tracks process IDs across application restarts
 * Helps clean up orphaned processes from previous runs
 */

import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { logger } from './SafeLogger';

export class PidTracker {
  private static pidFile = path.join(os.tmpdir(), 'hive-electron-poc.pids');
  
  /**
   * Record a PID for tracking
   */
  static addPid(pid: number, name: string): void {
    try {
      let pids: Record<string, number[]> = {};
      
      if (fs.existsSync(this.pidFile)) {
        const content = fs.readFileSync(this.pidFile, 'utf-8');
        try {
          pids = JSON.parse(content);
        } catch {
          // Invalid JSON, start fresh
          pids = {};
        }
      }
      
      if (!pids[name]) {
        pids[name] = [];
      }
      
      if (!pids[name].includes(pid)) {
        pids[name].push(pid);
      }
      
      fs.writeFileSync(this.pidFile, JSON.stringify(pids, null, 2));
      logger.info(`[PidTracker] Recorded PID ${pid} for ${name}`);
    } catch (error) {
      logger.error(`[PidTracker] Failed to record PID: ${error}`);
    }
  }
  
  /**
   * Remove a PID from tracking
   */
  static removePid(pid: number): void {
    try {
      if (!fs.existsSync(this.pidFile)) return;
      
      const content = fs.readFileSync(this.pidFile, 'utf-8');
      let pids: Record<string, number[]> = {};
      
      try {
        pids = JSON.parse(content);
      } catch {
        return; // Invalid JSON, nothing to remove
      }
      
      for (const name in pids) {
        pids[name] = pids[name].filter(p => p !== pid);
        if (pids[name].length === 0) {
          delete pids[name];
        }
      }
      
      if (Object.keys(pids).length > 0) {
        fs.writeFileSync(this.pidFile, JSON.stringify(pids, null, 2));
      } else {
        // No more PIDs, remove the file
        fs.unlinkSync(this.pidFile);
      }
      
      logger.info(`[PidTracker] Removed PID ${pid}`);
    } catch (error) {
      logger.error(`[PidTracker] Failed to remove PID: ${error}`);
    }
  }
  
  /**
   * Clean up orphaned processes from previous runs
   */
  static async cleanupOrphans(): Promise<void> {
    try {
      if (!fs.existsSync(this.pidFile)) return;
      
      const content = fs.readFileSync(this.pidFile, 'utf-8');
      let pids: Record<string, number[]> = {};
      
      try {
        pids = JSON.parse(content);
      } catch {
        // Invalid JSON, remove file
        fs.unlinkSync(this.pidFile);
        return;
      }
      
      logger.info('[PidTracker] Checking for orphaned processes...');
      
      for (const name in pids) {
        for (const pid of pids[name]) {
          if (this.isProcessRunning(pid)) {
            logger.info(`[PidTracker] Killing orphaned process ${name} (PID: ${pid})`);
            try {
              process.kill(pid, 'SIGTERM');
              // Give it a moment to terminate gracefully
              await new Promise(resolve => setTimeout(resolve, 1000));
              
              // Force kill if still running
              if (this.isProcessRunning(pid)) {
                process.kill(pid, 'SIGKILL');
              }
            } catch (error) {
              // Process might have already died
              logger.debug(`[PidTracker] Failed to kill PID ${pid}: ${error}`);
            }
          }
        }
      }
      
      // Clear the PID file after cleanup
      fs.unlinkSync(this.pidFile);
      logger.info('[PidTracker] Orphaned process cleanup complete');
    } catch (error) {
      logger.error(`[PidTracker] Cleanup failed: ${error}`);
    }
  }
  
  /**
   * Check if a process is running
   */
  private static isProcessRunning(pid: number): boolean {
    try {
      // Sending signal 0 checks if process exists without killing it
      process.kill(pid, 0);
      return true;
    } catch {
      return false;
    }
  }
  
  /**
   * Get all tracked PIDs
   */
  static getTrackedPids(): Record<string, number[]> {
    try {
      if (!fs.existsSync(this.pidFile)) return {};
      
      const content = fs.readFileSync(this.pidFile, 'utf-8');
      return JSON.parse(content);
    } catch {
      return {};
    }
  }
}