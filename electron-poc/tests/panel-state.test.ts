import assert from 'assert';
import { nextStateOnToggle, type CenterView, type PanelState } from '../src/utils/panel-state';

function test(name: string, fn: () => void) {
  try { fn(); console.log(`✓ ${name}`); } catch (e) { console.error(`✗ ${name}`); throw e; }
}

const S = (current: CenterView | null, last: CenterView | null): PanelState => ({ current, last });

test('toggle from Welcome to Settings and back', () => {
  let r = nextStateOnToggle(S('welcome', null), 'settings');
  assert.deepStrictEqual({ current: r.current, last: r.last, target: r.target }, { current: 'settings', last: 'welcome', target: 'settings' });

  r = nextStateOnToggle(S(r.current, r.last), 'settings');
  // Toggling off returns to last (welcome)
  assert.deepStrictEqual({ current: r.current, last: r.last, target: r.target }, { current: 'welcome', last: 'settings', target: 'welcome' });
});

test('switch between Memory and CLI Tools, toggling off returns to previous', () => {
  // Start from Welcome → Memory
  let r = nextStateOnToggle(S('welcome', null), 'memory');
  assert.strictEqual(r.current, 'memory');
  assert.strictEqual(r.last, 'welcome');

  // Memory → CLI Tools
  r = nextStateOnToggle(S(r.current, r.last), 'cli-tools');
  assert.strictEqual(r.current, 'cli-tools');
  assert.strictEqual(r.last, 'memory');

  // Toggle CLI Tools off → returns to Memory
  r = nextStateOnToggle(S(r.current, r.last), 'cli-tools');
  assert.strictEqual(r.current, 'memory');
  assert.strictEqual(r.last, 'cli-tools');
});

test('toggle Help off falls back to Welcome when no distinct last', () => {
  // Directly open Help from Welcome
  let r = nextStateOnToggle(S('welcome', null), 'help');
  assert.strictEqual(r.current, 'help');
  assert.strictEqual(r.last, 'welcome');

  // Toggle Help off → back to Welcome
  r = nextStateOnToggle(S(r.current, r.last), 'help');
  assert.strictEqual(r.current, 'welcome');
});

test('explicit close (null action) returns to last or Welcome', () => {
  let r = nextStateOnToggle(S('settings', 'help'), null);
  assert.strictEqual(r.current, 'help');

  r = nextStateOnToggle(S('memory', null), null);
  assert.strictEqual(r.current, 'welcome');
});

test('toggling Welcome off yields no-op (null target)', () => {
  const r = nextStateOnToggle(S('welcome', null), 'welcome');
  assert.strictEqual(r.target, null);
  assert.strictEqual(r.current, null);
});

console.log('All panel-state tests passed');

