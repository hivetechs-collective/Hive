import assert from 'assert';
import { computeLayout } from '../src/utils/welcome-layout';

function test(name: string, fn: () => void) {
  try { fn(); console.log(`✓ ${name}`); } catch (e) { console.error(`✗ ${name}`); throw e; }
}

test('balanced with recents', () => {
  const l = computeLayout('balanced', true);
  assert.strictEqual(l.recentWidth, '70%');
  assert.strictEqual(l.startWidth, '15%');
  assert.strictEqual(l.learnWidth, '15%');
  assert.strictEqual(l.showLearn, true);
});

test('minimal with recents hides learn', () => {
  const l = computeLayout('minimal', true);
  assert.strictEqual(l.recentWidth, '80%');
  assert.strictEqual(l.showLearn, false);
});

test('full with recents', () => {
  const l = computeLayout('full', true);
  assert.strictEqual(l.recentWidth, '60%');
  assert.strictEqual(l.learnWidth, '20%');
});

test('no recents equal thirds', () => {
  const l = computeLayout('balanced', false);
  assert.strictEqual(l.recentWidth, '33.33%');
  assert.strictEqual(l.learnWidth, '33.33%');
  assert.strictEqual(l.showLearn, true);
});

console.log('All welcome-layout tests passed');

