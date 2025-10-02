/**
 * PTY Integration Tests
 *
 * Comprehensive integration tests for the PTY terminal system.
 * Tests basic I/O, command execution, multiple terminals, resize handling, and cleanup.
 *
 * NOTE: This test requires Electron environment as node-pty is compiled for Electron.
 * Run with: npm run test:pty
 * Or in Electron context using the test runner.
 *
 * For standalone testing with ts-node, the module compatibility issues prevent direct execution.
 * Use the Electron test harness or run after building the Electron app.
 */

import { PtyService, PtyTerminal } from '../src/main/terminal/PtyService';

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

/**
 * Helper to wait for specific output pattern from terminal
 */
function waitForOutput(
  ptyService: PtyService,
  terminalId: string,
  pattern: string | RegExp,
  timeout: number = 3000
): Promise<string> {
  return new Promise((resolve, reject) => {
    let buffer = '';
    const timeoutId = setTimeout(() => {
      cleanup();
      reject(new Error(`Timeout waiting for pattern: ${pattern} (received: ${JSON.stringify(buffer)})`));
    }, timeout);

    const dataHandler = (id: string, data: string) => {
      if (id !== terminalId) return;
      buffer += data;

      const matches = typeof pattern === 'string'
        ? buffer.includes(pattern)
        : pattern.test(buffer);

      if (matches) {
        cleanup();
        resolve(buffer);
      }
    };

    const cleanup = () => {
      clearTimeout(timeoutId);
      ptyService.off('data', dataHandler);
    };

    ptyService.on('data', dataHandler);
  });
}

/**
 * Helper to wait for terminal exit
 */
function waitForExit(
  ptyService: PtyService,
  terminalId: string,
  timeout: number = 3000
): Promise<{ exitCode: number; signal?: number }> {
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      cleanup();
      reject(new Error(`Timeout waiting for terminal ${terminalId} to exit`));
    }, timeout);

    const exitHandler = (id: string, exitCode: number, signal?: number) => {
      if (id !== terminalId) return;
      cleanup();
      resolve({ exitCode, signal });
    };

    const cleanup = () => {
      clearTimeout(timeoutId);
      ptyService.off('exit', exitHandler);
    };

    ptyService.on('exit', exitHandler);
  });
}

/**
 * Helper to sleep for specified milliseconds
 */
function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

(async () => {
  console.log('[PTY Integration Tests] Starting...\n');

  // Test 1: Basic I/O Test
  console.log('Test 1: Basic I/O');
  {
    const ptyService = new PtyService();

    try {
      const terminal = await ptyService.spawn({
        id: 'test-basic-io',
        cols: 80,
        rows: 24
      });

      assert(terminal.id === 'test-basic-io', 'Terminal ID should match');
      assert(ptyService.hasTerminal('test-basic-io'), 'Terminal should exist in service');

      // Wait for shell prompt to appear
      await sleep(500);

      // Write command and wait for output
      const outputPromise = waitForOutput(ptyService, terminal.id, 'test');
      await ptyService.write(terminal.id, 'echo test\n');
      const output = await outputPromise;

      assert(output.includes('test'), 'Output should contain "test"');
      console.log('  ✓ Basic I/O working\n');

      await ptyService.cleanup();
    } catch (error) {
      await ptyService.cleanup();
      throw error;
    }
  }

  // Test 2: Command Execution (ls, pwd, echo $SHELL)
  console.log('Test 2: Command Execution');
  {
    const ptyService = new PtyService();

    try {
      const terminal = await ptyService.spawn({
        id: 'test-commands',
        cols: 80,
        rows: 24,
        cwd: process.env.HOME
      });

      // Wait for shell to be ready
      await sleep(500);

      // Test: ls command
      {
        const outputPromise = waitForOutput(ptyService, terminal.id, /Desktop|Documents|Downloads/);
        await ptyService.write(terminal.id, 'ls\n');
        const output = await outputPromise;
        assert(
          output.match(/Desktop|Documents|Downloads/),
          'ls output should contain typical home directory folders'
        );
        console.log('  ✓ ls command executed successfully');
      }

      await sleep(200);

      // Test: pwd command
      {
        const outputPromise = waitForOutput(ptyService, terminal.id, process.env.HOME!);
        await ptyService.write(terminal.id, 'pwd\n');
        const output = await outputPromise;
        assert(
          output.includes(process.env.HOME!),
          `pwd output should contain ${process.env.HOME}`
        );
        console.log('  ✓ pwd command executed successfully');
      }

      await sleep(200);

      // Test: echo $SHELL to verify shell type
      {
        const outputPromise = waitForOutput(ptyService, terminal.id, /\/(bash|zsh|sh)/);
        await ptyService.write(terminal.id, 'echo $SHELL\n');
        const output = await outputPromise;
        assert(
          output.match(/\/(bash|zsh|sh)/),
          'echo $SHELL should show shell path'
        );
        console.log('  ✓ echo $SHELL verified shell type\n');
      }

      await ptyService.cleanup();
    } catch (error) {
      await ptyService.cleanup();
      throw error;
    }
  }

  // Test 3: Multiple Terminals
  console.log('Test 3: Multiple Terminals');
  {
    const ptyService = new PtyService();

    try {
      // Spawn 3 terminals simultaneously
      const terminal1 = await ptyService.spawn({ id: 'multi-1', cols: 80, rows: 24 });
      const terminal2 = await ptyService.spawn({ id: 'multi-2', cols: 80, rows: 24 });
      const terminal3 = await ptyService.spawn({ id: 'multi-3', cols: 80, rows: 24 });

      // Verify each gets unique ID
      assert(terminal1.id === 'multi-1', 'Terminal 1 should have correct ID');
      assert(terminal2.id === 'multi-2', 'Terminal 2 should have correct ID');
      assert(terminal3.id === 'multi-3', 'Terminal 3 should have correct ID');
      console.log('  ✓ Each terminal has unique ID');

      // Verify all terminals exist
      const terminals = ptyService.listTerminals();
      assert(terminals.length === 3, 'Should have 3 active terminals');
      console.log('  ✓ All 3 terminals registered');

      // Wait for shells to be ready
      await sleep(500);

      // Write to each terminal independently and verify outputs don't cross-contaminate
      const outputs: Record<string, string> = {};

      // Set up listeners for each terminal
      const output1Promise = waitForOutput(ptyService, 'multi-1', 'TERM1');
      const output2Promise = waitForOutput(ptyService, 'multi-2', 'TERM2');
      const output3Promise = waitForOutput(ptyService, 'multi-3', 'TERM3');

      // Write to each terminal
      await ptyService.write('multi-1', 'echo TERM1\n');
      await ptyService.write('multi-2', 'echo TERM2\n');
      await ptyService.write('multi-3', 'echo TERM3\n');

      // Wait for all outputs
      outputs['multi-1'] = await output1Promise;
      outputs['multi-2'] = await output2Promise;
      outputs['multi-3'] = await output3Promise;

      // Verify no cross-contamination
      assert(outputs['multi-1'].includes('TERM1'), 'Terminal 1 should receive TERM1');
      assert(!outputs['multi-1'].includes('TERM2'), 'Terminal 1 should not receive TERM2');
      assert(!outputs['multi-1'].includes('TERM3'), 'Terminal 1 should not receive TERM3');

      assert(outputs['multi-2'].includes('TERM2'), 'Terminal 2 should receive TERM2');
      assert(!outputs['multi-2'].includes('TERM1'), 'Terminal 2 should not receive TERM1');
      assert(!outputs['multi-2'].includes('TERM3'), 'Terminal 2 should not receive TERM3');

      assert(outputs['multi-3'].includes('TERM3'), 'Terminal 3 should receive TERM3');
      assert(!outputs['multi-3'].includes('TERM1'), 'Terminal 3 should not receive TERM1');
      assert(!outputs['multi-3'].includes('TERM2'), 'Terminal 3 should not receive TERM2');

      console.log('  ✓ Output isolation verified (no cross-contamination)\n');

      await ptyService.cleanup();
    } catch (error) {
      await ptyService.cleanup();
      throw error;
    }
  }

  // Test 4: Resize Handling
  console.log('Test 4: Resize Handling');
  {
    const ptyService = new PtyService();

    try {
      const terminal = await ptyService.spawn({
        id: 'test-resize',
        cols: 80,
        rows: 24
      });

      assert(terminal.cols === 80, 'Initial cols should be 80');
      assert(terminal.rows === 24, 'Initial rows should be 24');

      // Resize to 100x30
      await ptyService.resize(terminal.id, 100, 30);

      const updatedTerminal = ptyService.getTerminal(terminal.id);
      assert(updatedTerminal?.cols === 100, 'Cols should be updated to 100');
      assert(updatedTerminal?.rows === 30, 'Rows should be updated to 30');
      console.log('  ✓ Resize to 100x30 successful');

      // Test invalid dimensions
      let errorThrown = false;
      try {
        await ptyService.resize(terminal.id, 0, 30);
      } catch (error) {
        errorThrown = true;
        assert(
          error instanceof Error && error.message.includes('Invalid terminal dimensions'),
          'Should throw error for invalid dimensions'
        );
      }
      assert(errorThrown, 'Should throw error for cols = 0');
      console.log('  ✓ Invalid dimension handling verified\n');

      await ptyService.cleanup();
    } catch (error) {
      await ptyService.cleanup();
      throw error;
    }
  }

  // Test 5: Cleanup
  console.log('Test 5: Cleanup');
  {
    const ptyService = new PtyService();

    try {
      const terminal = await ptyService.spawn({
        id: 'test-cleanup',
        cols: 80,
        rows: 24
      });

      assert(ptyService.hasTerminal('test-cleanup'), 'Terminal should exist before kill');

      // Set up exit listener
      const exitPromise = waitForExit(ptyService, terminal.id);

      // Kill terminal
      await ptyService.kill(terminal.id);

      // Wait for exit event
      const { exitCode } = await exitPromise;
      console.log(`  ✓ Exit event fired (code: ${exitCode})`);

      // Verify terminal removed from registry
      assert(!ptyService.hasTerminal('test-cleanup'), 'Terminal should be removed after kill');
      console.log('  ✓ Terminal removed from registry');

      const terminals = ptyService.listTerminals();
      assert(terminals.length === 0, 'Should have no active terminals');
      console.log('  ✓ Terminal registry empty\n');

      await ptyService.cleanup();
    } catch (error) {
      await ptyService.cleanup();
      throw error;
    }
  }

  // Test 6: Error Handling
  console.log('Test 6: Error Handling');
  {
    const ptyService = new PtyService();

    try {
      // Test writing to non-existent terminal
      let errorThrown = false;
      try {
        await ptyService.write('non-existent', 'test\n');
      } catch (error) {
        errorThrown = true;
        assert(
          error instanceof Error && error.message.includes('not found'),
          'Should throw error for non-existent terminal'
        );
      }
      assert(errorThrown, 'Should throw error when writing to non-existent terminal');
      console.log('  ✓ Write to non-existent terminal handled');

      // Test resizing non-existent terminal
      errorThrown = false;
      try {
        await ptyService.resize('non-existent', 80, 24);
      } catch (error) {
        errorThrown = true;
        assert(
          error instanceof Error && error.message.includes('not found'),
          'Should throw error for non-existent terminal'
        );
      }
      assert(errorThrown, 'Should throw error when resizing non-existent terminal');
      console.log('  ✓ Resize non-existent terminal handled');

      // Test killing non-existent terminal
      errorThrown = false;
      try {
        await ptyService.kill('non-existent');
      } catch (error) {
        errorThrown = true;
        assert(
          error instanceof Error && error.message.includes('not found'),
          'Should throw error for non-existent terminal'
        );
      }
      assert(errorThrown, 'Should throw error when killing non-existent terminal');
      console.log('  ✓ Kill non-existent terminal handled\n');

      await ptyService.cleanup();
    } catch (error) {
      await ptyService.cleanup();
      throw error;
    }
  }

  // Test 7: Service Lifecycle
  console.log('Test 7: Service Lifecycle');
  {
    const ptyService = new PtyService();

    try {
      // Spawn multiple terminals
      await ptyService.spawn({ id: 'lifecycle-1', cols: 80, rows: 24 });
      await ptyService.spawn({ id: 'lifecycle-2', cols: 80, rows: 24 });
      await ptyService.spawn({ id: 'lifecycle-3', cols: 80, rows: 24 });

      assert(ptyService.listTerminals().length === 3, 'Should have 3 terminals');
      console.log('  ✓ 3 terminals spawned');

      // Cleanup should kill all terminals
      await ptyService.cleanup();

      assert(ptyService.listTerminals().length === 0, 'All terminals should be cleaned up');
      console.log('  ✓ Cleanup removed all terminals\n');

    } catch (error) {
      await ptyService.cleanup();
      throw error;
    }
  }

  console.log('[PTY Integration Tests] All tests passed! ✓\n');
})().catch((error) => {
  console.error('[PTY Integration Tests] FAILED:', error.message);
  console.error(error.stack);
  process.exit(1);
});
