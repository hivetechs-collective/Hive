import assert from 'assert';
import { shouldShowWhatsNewBadge } from '../src/utils/welcome-logic';

function test(name: string, fn: () => void) {
  try { fn(); console.log(`✓ ${name}`); }
  catch (e) { console.error(`✗ ${name}`); throw e; }
}

test('shows badge when no lastSeenVersion', () => {
  assert.strictEqual(shouldShowWhatsNewBadge('1.9.0', null), true);
});

test('hides badge when versions equal', () => {
  assert.strictEqual(shouldShowWhatsNewBadge('1.9.0', '1.9.0'), false);
});

test('shows badge when versions differ', () => {
  assert.strictEqual(shouldShowWhatsNewBadge('1.9.1', '1.9.0'), true);
});

test('hides badge when no current version', () => {
  assert.strictEqual(shouldShowWhatsNewBadge(undefined as any, '1.9.0'), false);
});

console.log('All welcome-logic tests passed');

