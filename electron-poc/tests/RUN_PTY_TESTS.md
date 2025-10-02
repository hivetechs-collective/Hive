# Quick Reference: Running PTY Tests

## Unit Tests (Fast, Node.js)

```bash
# Run mock-based unit tests (instant execution)
ts-node tests/terminal-pty-unit.test.ts
```

**Expected Output:**
```
[PTY Unit Tests] Starting...

Test 1: Basic Spawn and Properties
  ✓ Spawn and properties verified

Test 2: Multiple Terminals
  ✓ Multiple terminals created

... (11 tests total)

[PTY Unit Tests] All tests passed! ✓
```

## Integration Tests (Electron Required)

### Option 1: Add npm script (Recommended)

Add to `package.json`:
```json
{
  "scripts": {
    "test:pty": "electron tests/terminal-pty-integration.test.ts",
    "test:pty:unit": "ts-node tests/terminal-pty-unit.test.ts"
  }
}
```

Then run:
```bash
npm run test:pty:unit    # Unit tests (fast)
npm run test:pty         # Integration tests (Electron)
```

### Option 2: Direct electron execution

```bash
electron tests/terminal-pty-integration.test.ts
```

### Option 3: Run in Electron app

Import in main process or test harness:
```typescript
import './tests/terminal-pty-integration.test';
```

## Test Files

| File | Type | Requires | Run With |
|------|------|----------|----------|
| `terminal-pty-unit.test.ts` | Unit (Mock) | Node.js | `ts-node` |
| `terminal-pty-integration.test.ts` | Integration (Real) | Electron | `electron` |

## Why Two Test Files?

- **node-pty** is compiled for Electron's Node version
- Cannot run directly with `ts-node` due to native module mismatch
- **Solution:** Unit tests use mocks, Integration tests use real PTY

## Test Coverage

### Unit Tests ✅
- Service logic
- Error handling
- Event isolation
- Resource cleanup

### Integration Tests ✅
- Real PTY processes
- Shell commands (ls, pwd, echo)
- Multiple terminals
- I/O verification

## Quick Commands

```bash
# Unit tests only (fastest)
ts-node tests/terminal-pty-unit.test.ts

# Add integration test support
echo '{ "scripts": { "test:pty": "electron tests/terminal-pty-integration.test.ts" } }' | jq -s '.[0] * .[1]' package.json - > package.json.tmp && mv package.json.tmp package.json

# Run integration tests
npm run test:pty
```
