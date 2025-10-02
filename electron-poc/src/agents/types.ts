/**
 * Type definitions for Claude Agent SDK integration
 */

export interface TerminalMigrationResult {
  success: boolean;
  filesCreated: string[];
  filesModified: string[];
  filesDeleted: string[];
  testsPass: boolean;
  errors: string[];
  warnings: string[];
}

export interface PtyPattern {
  spawning: string[];
  dataFlow: string[];
  cleanup: string[];
  interfaces: string[];
}

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
  checks: {
    ipcHandlers: boolean;
    dataFlow: boolean;
    eventBindings: boolean;
    typeDefinitions: boolean;
  };
}

export interface DeprecatedCodeInfo {
  filePath: string;
  lineCount: number;
  references: string[];
  safeToDelete: boolean;
  reason?: string;
}
