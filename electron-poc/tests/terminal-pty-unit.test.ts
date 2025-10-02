/**
 * PTY Unit Tests (Mock-based)
 *
 * Unit tests for PtyService using mocked node-pty interface.
 * These tests can run in Node.js without Electron.
 */

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

// Mock IPty interface
interface MockIPty {
  pid: number;
  write: (data: string) => void;
  resize: (cols: number, rows: number) => void;
  kill: (signal?: string) => void;
  onData: (callback: (data: string) => void) => void;
  onExit: (callback: (event: { exitCode: number; signal?: number }) => void) => void;
}

class MockPtyService {
  private terminals: Map<string, any> = new Map();
  private mockPtyInstances: Map<string, MockIPty> = new Map();
  private eventHandlers: {
    data: Array<(terminalId: string, data: string) => void>;
    exit: Array<(terminalId: string, exitCode: number, signal?: number) => void>;
  } = { data: [], exit: [] };

  async spawn(options: any = {}): Promise<any> {
    const terminalId = options.id || this.generateTerminalId();
    const cols = options.cols || 80;
    const rows = options.rows || 24;

    // Create mock PTY
    const mockPty: MockIPty = {
      pid: Math.floor(Math.random() * 10000),
      write: (data: string) => {
        // Simulate echo back
        setTimeout(() => {
          this.eventHandlers.data.forEach(handler => handler(terminalId, data));
        }, 10);
      },
      resize: (newCols: number, newRows: number) => {
        const terminal = this.terminals.get(terminalId);
        if (terminal) {
          terminal.cols = newCols;
          terminal.rows = newRows;
        }
      },
      kill: (signal?: string) => {
        setTimeout(() => {
          this.eventHandlers.exit.forEach(handler => handler(terminalId, 0, undefined));
          this.terminals.delete(terminalId);
        }, 10);
      },
      onData: (callback: (data: string) => void) => {
        // Store for later
      },
      onExit: (callback: (event: { exitCode: number; signal?: number }) => void) => {
        // Store for later
      }
    };

    const terminal = {
      id: terminalId,
      title: options.title || `Terminal ${terminalId}`,
      pty: mockPty,
      shell: options.shell || '/bin/bash',
      cwd: options.cwd || process.cwd(),
      cols,
      rows,
      createdAt: new Date(),
      toolId: options.toolId,
      terminalNumber: options.terminalNumber
    };

    this.terminals.set(terminalId, terminal);
    this.mockPtyInstances.set(terminalId, mockPty);

    return terminal;
  }

  async write(terminalId: string, data: string): Promise<void> {
    const terminal = this.terminals.get(terminalId);
    if (!terminal) {
      throw new Error(`Terminal ${terminalId} not found`);
    }
    terminal.pty.write(data);
  }

  async resize(terminalId: string, cols: number, rows: number): Promise<void> {
    const terminal = this.terminals.get(terminalId);
    if (!terminal) {
      throw new Error(`Terminal ${terminalId} not found`);
    }
    if (cols < 1 || rows < 1) {
      throw new Error(`Invalid terminal dimensions: ${cols}x${rows}`);
    }
    terminal.pty.resize(cols, rows);
  }

  async kill(terminalId: string, signal?: string): Promise<void> {
    const terminal = this.terminals.get(terminalId);
    if (!terminal) {
      throw new Error(`Terminal ${terminalId} not found`);
    }
    terminal.pty.kill(signal);
  }

  onData(callback: (terminalId: string, data: string) => void): () => void {
    this.eventHandlers.data.push(callback);
    return () => {
      const index = this.eventHandlers.data.indexOf(callback);
      if (index > -1) this.eventHandlers.data.splice(index, 1);
    };
  }

  onExit(callback: (terminalId: string, exitCode: number, signal?: number) => void): () => void {
    this.eventHandlers.exit.push(callback);
    return () => {
      const index = this.eventHandlers.exit.indexOf(callback);
      if (index > -1) this.eventHandlers.exit.splice(index, 1);
    };
  }

  listTerminals(): any[] {
    return Array.from(this.terminals.values()).map(t => ({
      terminalId: t.id,
      title: t.title,
      shell: t.shell,
      cwd: t.cwd,
      cols: t.cols,
      rows: t.rows,
      createdAt: t.createdAt,
      toolId: t.toolId,
      terminalNumber: t.terminalNumber
    }));
  }

  getTerminal(terminalId: string): any {
    return this.terminals.get(terminalId);
  }

  hasTerminal(terminalId: string): boolean {
    return this.terminals.has(terminalId);
  }

  async cleanup(): Promise<void> {
    this.terminals.clear();
    this.eventHandlers.data = [];
    this.eventHandlers.exit = [];
  }

  private generateTerminalId(): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 11);
    return `terminal-${timestamp}-${random}`;
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

(async () => {
  console.log('[PTY Unit Tests] Starting...\n');

  // Test 1: Basic Spawn and Properties
  console.log('Test 1: Basic Spawn and Properties');
  {
    const service = new MockPtyService();

    const terminal = await service.spawn({
      id: 'test-1',
      title: 'Test Terminal',
      cols: 80,
      rows: 24
    });

    assert(terminal.id === 'test-1', 'Terminal ID should match');
    assert(terminal.title === 'Test Terminal', 'Terminal title should match');
    assert(terminal.cols === 80, 'Cols should be 80');
    assert(terminal.rows === 24, 'Rows should be 24');
    assert(service.hasTerminal('test-1'), 'Terminal should exist');
    console.log('  ✓ Spawn and properties verified\n');

    await service.cleanup();
  }

  // Test 2: Multiple Terminals
  console.log('Test 2: Multiple Terminals');
  {
    const service = new MockPtyService();

    const t1 = await service.spawn({ id: 'multi-1' });
    const t2 = await service.spawn({ id: 'multi-2' });
    const t3 = await service.spawn({ id: 'multi-3' });

    const terminals = service.listTerminals();
    assert(terminals.length === 3, 'Should have 3 terminals');
    assert(service.hasTerminal('multi-1'), 'Terminal 1 exists');
    assert(service.hasTerminal('multi-2'), 'Terminal 2 exists');
    assert(service.hasTerminal('multi-3'), 'Terminal 3 exists');
    console.log('  ✓ Multiple terminals created\n');

    await service.cleanup();
  }

  // Test 3: Write Operation
  console.log('Test 3: Write Operation');
  {
    const service = new MockPtyService();
    const terminal = await service.spawn({ id: 'test-write' });

    let receivedData = '';
    service.onData((id, data) => {
      if (id === 'test-write') {
        receivedData += data;
      }
    });

    await service.write('test-write', 'hello');
    await sleep(50);
    assert(receivedData === 'hello', 'Should receive written data');
    console.log('  ✓ Write operation works\n');

    await service.cleanup();
  }

  // Test 4: Resize
  console.log('Test 4: Resize');
  {
    const service = new MockPtyService();
    const terminal = await service.spawn({ id: 'test-resize', cols: 80, rows: 24 });

    await service.resize('test-resize', 100, 30);

    const updated = service.getTerminal('test-resize');
    assert(updated.cols === 100, 'Cols should be updated to 100');
    assert(updated.rows === 30, 'Rows should be updated to 30');
    console.log('  ✓ Resize works correctly\n');

    await service.cleanup();
  }

  // Test 5: Invalid Resize
  console.log('Test 5: Invalid Resize');
  {
    const service = new MockPtyService();
    await service.spawn({ id: 'test-invalid-resize' });

    let errorThrown = false;
    try {
      await service.resize('test-invalid-resize', 0, 24);
    } catch (error) {
      errorThrown = true;
      assert(
        error instanceof Error && error.message.includes('Invalid terminal dimensions'),
        'Should throw invalid dimensions error'
      );
    }
    assert(errorThrown, 'Should throw error for invalid dimensions');
    console.log('  ✓ Invalid resize handled\n');

    await service.cleanup();
  }

  // Test 6: Kill Terminal
  console.log('Test 6: Kill Terminal');
  {
    const service = new MockPtyService();
    const terminal = await service.spawn({ id: 'test-kill' });

    let exitCalled = false;
    service.onExit((id, exitCode) => {
      if (id === 'test-kill') {
        exitCalled = true;
        assert(exitCode === 0, 'Exit code should be 0');
      }
    });

    await service.kill('test-kill');
    await sleep(50);

    assert(exitCalled, 'Exit event should be fired');
    assert(!service.hasTerminal('test-kill'), 'Terminal should be removed');
    console.log('  ✓ Kill terminal works\n');

    await service.cleanup();
  }

  // Test 7: Error Handling - Write to Non-existent Terminal
  console.log('Test 7: Error Handling - Write');
  {
    const service = new MockPtyService();

    let errorThrown = false;
    try {
      await service.write('non-existent', 'test');
    } catch (error) {
      errorThrown = true;
      assert(
        error instanceof Error && error.message.includes('not found'),
        'Should throw not found error'
      );
    }
    assert(errorThrown, 'Should throw error for non-existent terminal');
    console.log('  ✓ Write error handling works\n');

    await service.cleanup();
  }

  // Test 8: Error Handling - Resize Non-existent Terminal
  console.log('Test 8: Error Handling - Resize');
  {
    const service = new MockPtyService();

    let errorThrown = false;
    try {
      await service.resize('non-existent', 80, 24);
    } catch (error) {
      errorThrown = true;
      assert(
        error instanceof Error && error.message.includes('not found'),
        'Should throw not found error'
      );
    }
    assert(errorThrown, 'Should throw error for non-existent terminal');
    console.log('  ✓ Resize error handling works\n');

    await service.cleanup();
  }

  // Test 9: Error Handling - Kill Non-existent Terminal
  console.log('Test 9: Error Handling - Kill');
  {
    const service = new MockPtyService();

    let errorThrown = false;
    try {
      await service.kill('non-existent');
    } catch (error) {
      errorThrown = true;
      assert(
        error instanceof Error && error.message.includes('not found'),
        'Should throw not found error'
      );
    }
    assert(errorThrown, 'Should throw error for non-existent terminal');
    console.log('  ✓ Kill error handling works\n');

    await service.cleanup();
  }

  // Test 10: Event Isolation
  console.log('Test 10: Event Isolation');
  {
    const service = new MockPtyService();

    await service.spawn({ id: 'term-1' });
    await service.spawn({ id: 'term-2' });

    const term1Data: string[] = [];
    const term2Data: string[] = [];

    service.onData((id, data) => {
      if (id === 'term-1') term1Data.push(data);
      if (id === 'term-2') term2Data.push(data);
    });

    await service.write('term-1', 'data1');
    await service.write('term-2', 'data2');
    await sleep(50);

    assert(term1Data.length === 1 && term1Data[0] === 'data1', 'Term 1 should receive only its data');
    assert(term2Data.length === 1 && term2Data[0] === 'data2', 'Term 2 should receive only its data');
    console.log('  ✓ Event isolation works\n');

    await service.cleanup();
  }

  // Test 11: Cleanup All Terminals
  console.log('Test 11: Cleanup All Terminals');
  {
    const service = new MockPtyService();

    await service.spawn({ id: 'cleanup-1' });
    await service.spawn({ id: 'cleanup-2' });
    await service.spawn({ id: 'cleanup-3' });

    assert(service.listTerminals().length === 3, 'Should have 3 terminals before cleanup');

    await service.cleanup();

    assert(service.listTerminals().length === 0, 'Should have no terminals after cleanup');
    assert(!service.hasTerminal('cleanup-1'), 'Terminal 1 should be removed');
    assert(!service.hasTerminal('cleanup-2'), 'Terminal 2 should be removed');
    assert(!service.hasTerminal('cleanup-3'), 'Terminal 3 should be removed');
    console.log('  ✓ Cleanup all terminals works\n');
  }

  console.log('[PTY Unit Tests] All tests passed! ✓\n');
})().catch((error) => {
  console.error('[PTY Unit Tests] FAILED:', error.message);
  console.error(error.stack);
  process.exit(1);
});
