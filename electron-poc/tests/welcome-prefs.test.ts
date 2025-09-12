import assert from 'assert';
import { shouldShowOnStartup } from '../src/utils/welcome-prefs';

function test(name: string, fn: () => void) {
  try { fn(); console.log(`✓ ${name}`); } catch (e) { console.error(`✗ ${name}`); throw e; }
}

test('should show on startup when value is null/undefined', () => {
  assert.strictEqual(shouldShowOnStartup(null as any), true);
  assert.strictEqual(shouldShowOnStartup(undefined as any), true);
});

test('should show on startup when value is "1"', () => {
  assert.strictEqual(shouldShowOnStartup('1'), true);
});

test('should NOT show on startup when value is "0"', () => {
  assert.strictEqual(shouldShowOnStartup('0'), false);
});

console.log('All welcome-prefs tests passed');

