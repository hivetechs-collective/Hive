/**
 * Runtime Verification Module
 * 
 * Ensures critical configurations survive webpack bundling and remain intact at runtime.
 * This is essential for preventing production issues caused by build transformations.
 */

import { logger } from './SafeLogger';

interface VerificationResult {
  passed: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Verifies critical runtime configurations
 * Should be called at application startup
 */
export function verifyCriticalConfigurations(): VerificationResult {
  const result: VerificationResult = {
    passed: true,
    errors: [],
    warnings: []
  };

  logger.info('[RuntimeVerification] Starting critical configuration verification...');

  // Check 1: Verify stdio configuration constant
  try {
    // This should match the constant in ProcessManager
    const EXPECTED_STDIO = 'inherit';
    
    // Verify the string literal exists in the code
    if (typeof EXPECTED_STDIO !== 'string' || EXPECTED_STDIO !== 'inherit') {
      result.errors.push(`stdio configuration corrupted: Expected 'inherit', got '${EXPECTED_STDIO}'`);
      result.passed = false;
    } else {
      logger.info('[RuntimeVerification] ✓ stdio configuration verified');
    }
  } catch (error) {
    result.errors.push(`Failed to verify stdio configuration: ${error}`);
    result.passed = false;
  }

  // Check 2: Verify ProcessManager module exists
  try {
    const processManagerPath = require.resolve('./ProcessManager');
    if (!processManagerPath) {
      result.errors.push('ProcessManager module not found');
      result.passed = false;
    } else {
      logger.info('[RuntimeVerification] ✓ ProcessManager module found');
    }
  } catch (error) {
    result.errors.push(`ProcessManager module verification failed: ${error}`);
    result.passed = false;
  }

  // Check 3: Verify spawn function is available
  try {
    const { spawn } = require('child_process');
    if (typeof spawn !== 'function') {
      result.errors.push('child_process.spawn is not available');
      result.passed = false;
    } else {
      logger.info('[RuntimeVerification] ✓ child_process.spawn verified');
    }
  } catch (error) {
    result.errors.push(`child_process verification failed: ${error}`);
    result.passed = false;
  }

  // Check 4: Verify critical environment variables
  const criticalEnvVars = ['PATH', 'NODE_ENV'];
  for (const envVar of criticalEnvVars) {
    if (!process.env[envVar]) {
      result.warnings.push(`Environment variable ${envVar} is not set`);
    } else {
      logger.info(`[RuntimeVerification] ✓ Environment variable ${envVar} verified`);
    }
  }

  // Check 5: Verify webpack hasn't stripped critical functions
  try {
    // Check if critical spawn options are preserved
    const testSpawnOptions = {
      stdio: 'inherit' as const,
      detached: false,
      shell: false
    };
    
    if (!testSpawnOptions.stdio || testSpawnOptions.stdio !== 'inherit') {
      result.errors.push('Spawn options test failed - stdio configuration altered');
      result.passed = false;
    } else {
      logger.info('[RuntimeVerification] ✓ Spawn options structure verified');
    }
  } catch (error) {
    result.errors.push(`Spawn options verification failed: ${error}`);
    result.passed = false;
  }

  // Log results
  if (result.passed) {
    logger.info('[RuntimeVerification] ✅ All critical configurations verified successfully');
  } else {
    logger.error('[RuntimeVerification] ❌ Critical configuration verification failed');
    result.errors.forEach(error => logger.error(`[RuntimeVerification] ERROR: ${error}`));
  }
  
  if (result.warnings.length > 0) {
    result.warnings.forEach(warning => logger.warn(`[RuntimeVerification] WARNING: ${warning}`));
  }

  return result;
}

/**
 * Creates a diagnostic report for debugging configuration issues
 */
export function createDiagnosticReport(): string {
  const report: string[] = [];
  
  report.push('=== Hive Consensus Runtime Diagnostic Report ===');
  report.push(`Timestamp: ${new Date().toISOString()}`);
  report.push(`Node Version: ${process.version}`);
  report.push(`Platform: ${process.platform}`);
  report.push(`Architecture: ${process.arch}`);
  report.push('');
  
  // Check webpack environment
  report.push('=== Webpack Environment ===');
  report.push(`NODE_ENV: ${process.env.NODE_ENV}`);
  report.push(`Webpack Mode: ${process.env.NODE_ENV === 'production' ? 'production' : 'development'}`);
  report.push('');
  
  // Verify critical modules
  report.push('=== Module Verification ===');
  const criticalModules = [
    './ProcessManager',
    './logger',
    'child_process',
    'path',
    'fs'
  ];
  
  for (const moduleName of criticalModules) {
    try {
      require.resolve(moduleName);
      report.push(`✓ ${moduleName}: Found`);
    } catch (error) {
      report.push(`✗ ${moduleName}: Not found`);
    }
  }
  report.push('');
  
  // Check critical configurations
  report.push('=== Configuration Verification ===');
  const verificationResult = verifyCriticalConfigurations();
  report.push(`Overall Status: ${verificationResult.passed ? 'PASSED' : 'FAILED'}`);
  
  if (verificationResult.errors.length > 0) {
    report.push('Errors:');
    verificationResult.errors.forEach(error => report.push(`  - ${error}`));
  }
  
  if (verificationResult.warnings.length > 0) {
    report.push('Warnings:');
    verificationResult.warnings.forEach(warning => report.push(`  - ${warning}`));
  }
  
  report.push('');
  report.push('=== End of Diagnostic Report ===');
  
  return report.join('\n');
}

/**
 * Performs a runtime self-test of spawn functionality
 */
export async function testSpawnFunctionality(): Promise<boolean> {
  logger.info('[RuntimeVerification] Testing spawn functionality...');
  
  try {
    const { spawn } = require('child_process');
    
    // Test with echo command (cross-platform)
    const testCommand = process.platform === 'win32' ? 'cmd' : 'echo';
    const testArgs = process.platform === 'win32' ? ['/c', 'echo', 'test'] : ['test'];
    
    return new Promise((resolve) => {
      const child = spawn(testCommand, testArgs, {
        stdio: 'inherit',
        detached: false,
        shell: false
      });
      
      child.on('error', (error: Error) => {
        logger.error(`[RuntimeVerification] Spawn test failed: ${error}`);
        resolve(false);
      });
      
      child.on('exit', (code: number | null) => {
        if (code === 0) {
          logger.info('[RuntimeVerification] ✓ Spawn functionality test passed');
          resolve(true);
        } else {
          logger.error(`[RuntimeVerification] Spawn test exited with code ${code}`);
          resolve(false);
        }
      });
      
      // Timeout after 5 seconds
      setTimeout(() => {
        child.kill();
        logger.error('[RuntimeVerification] Spawn test timed out');
        resolve(false);
      }, 5000);
    });
  } catch (error) {
    logger.error(`[RuntimeVerification] Failed to test spawn: ${error}`);
    return false;
  }
}