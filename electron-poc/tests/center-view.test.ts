import assert from 'assert';
import { applyCenterView } from '../src/utils/center-view';
import type { CenterView, PanelState } from '../src/utils/center-view';

function test(name: string, fn: () => void) {
  try { fn(); console.log(`✓ ${name}`); } catch (e) { console.error(`✗ ${name}`); throw e; }
}

const S = (current: CenterView | null, last: CenterView | null): PanelState => ({ current, last });

test('idempotent focus keeps current and targets same view', () => {
  const state = S('settings', 'welcome');
  const r = applyCenterView(state, 'settings');
  assert.deepStrictEqual({ current: r.current, last: r.last, target: r.target }, { current: 'settings', last: 'welcome', target: 'settings' });
});

test('toggle off returns to last when different', () => {
  const r = applyCenterView(S('memory', 'welcome'), 'memory');
  assert.deepStrictEqual({ current: r.current, last: r.last, target: r.target }, { current: 'welcome', last: 'memory', target: 'welcome' });
});

test('toggle off help falls back to welcome', () => {
  const r1 = applyCenterView(S('welcome', null), 'help');
  assert.strictEqual(r1.current, 'help');
  const r2 = applyCenterView(S(r1.current, r1.last), 'help');
  assert.strictEqual(r2.current, 'welcome');
});

test('close (null) respects last or welcome', () => {
  const r1 = applyCenterView(S('cli-tools', 'memory'), null);
  assert.strictEqual(r1.current, 'memory');

  const r2 = applyCenterView(S('analytics', null), null);
  assert.strictEqual(r2.current, 'welcome');
});

test('close from welcome yields null target', () => {
  const r = applyCenterView(S('welcome', null), null);
  assert.strictEqual(r.target, null);
  assert.strictEqual(r.last, 'welcome');
});

console.log('All center-view tests passed');

