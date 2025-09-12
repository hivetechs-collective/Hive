import assert from 'assert';
import { isValidRepoUrl } from '../src/utils/clone-validate';

function test(name: string, fn: () => void) {
  try { fn(); console.log(`✓ ${name}`); } catch (e) { console.error(`✗ ${name}`); throw e; }
}

test('valid https URL without .git', () => {
  assert.strictEqual(isValidRepoUrl('https://github.com/org/repo'), true);
});

test('valid https URL with .git', () => {
  assert.strictEqual(isValidRepoUrl('https://github.com/org/repo.git'), true);
});

test('valid ssh URL', () => {
  assert.strictEqual(isValidRepoUrl('git@github.com:org/repo.git'), true);
});

test('invalid URLs', () => {
  ['github.com/org/repo', 'https://github.com', 'git@github.comorg/repo', ''].forEach(u => {
    assert.strictEqual(isValidRepoUrl(u), false);
  });
});

console.log('All clone validate tests passed');

