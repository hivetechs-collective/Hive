/**
 * Process Management Type Definitions
 * Unified process management for all process types
 */

/**
 * Types of processes we manage
 */
export enum ProcessType {
  SERVICE = 'service',        // Background services (memory-service, backend)
  TERMINAL = 'terminal',      // Terminal processes (CLI tools)
  NODE = 'node',             // Node.js processes
  SYSTEM = 'system'          // System commands
}

/**
 * Process status
 */
export enum ProcessStatus {
  STARTING = 'starting',
  RUNNING = 'running',
  STOPPING = 'stopping',
  STOPPED = 'stopped',
  CRASHED = 'crashed',
  RESTARTING = 'restarting'
}

/**
 * Base process information
 */
export interface ProcessInfo {
  id: string;
  type: ProcessType;
  status: ProcessStatus;
  pid?: number;
  name: string;
  command: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
  startedAt?: Date;
  stoppedAt?: Date;
  restartCount: number;
  port?: number;
  metadata?: Record<string, any>;
}

/**
 * Terminal-specific process info
 */
export interface TerminalProcessInfo extends ProcessInfo {
  type: ProcessType.TERMINAL;
  terminalId: string;
  toolId?: string;
  cols?: number;
  rows?: number;
}

/**
 * Service-specific process info
 */
export interface ServiceProcessInfo extends ProcessInfo {
  type: ProcessType.SERVICE;
  healthCheckUrl?: string;
  healthCheckInterval?: number;
  maxRestarts?: number;
  autoRestart?: boolean;
}

/**
 * Process spawn options
 */
export interface ProcessSpawnOptions {
  id: string;
  type: ProcessType;
  name: string;
  command: string;
  args?: string[];
  cwd?: string;
  env?: Record<string, string>;
  shell?: boolean | string;
  detached?: boolean;
  stdio?: 'pipe' | 'ignore' | 'inherit';
  timeout?: number;
  killSignal?: NodeJS.Signals;
}

/**
 * Terminal spawn options
 */
export interface TerminalSpawnOptions extends ProcessSpawnOptions {
  type: ProcessType.TERMINAL;
  cols?: number;
  rows?: number;
  terminalId: string;
  toolId?: string;
}

/**
 * Process events
 */
export enum ProcessEvent {
  STARTED = 'process:started',
  STOPPED = 'process:stopped',
  CRASHED = 'process:crashed',
  RESTARTED = 'process:restarted',
  OUTPUT = 'process:output',
  ERROR = 'process:error',
  EXIT = 'process:exit'
}

/**
 * Process event data
 */
export interface ProcessEventData {
  processId: string;
  event: ProcessEvent;
  timestamp: Date;
  data?: any;
  error?: Error;
}