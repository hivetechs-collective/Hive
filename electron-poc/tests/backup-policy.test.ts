import assert from 'assert';
import { isBackupDue, computeRetentionDeletes } from '../src/utils/backup-policy';

function test(name: string, fn: () => void) {
  try { fn(); console.log(`✓ ${name}`); } catch (e) { console.error(`✗ ${name}`); throw e; }
}

const now = new Date('2025-09-12T12:00:00Z');

test('backup due when never backed up', () => {
  assert.strictEqual(isBackupDue(null, 'daily', now), true);
});

test('daily not due within same day', () => {
  assert.strictEqual(isBackupDue('2025-09-12T01:00:00Z', 'daily', now), false);
});

test('daily due after 1+ days', () => {
  assert.strictEqual(isBackupDue('2025-09-10T01:00:00Z', 'daily', now), true);
});

test('weekly due after 7 days', () => {
  assert.strictEqual(isBackupDue('2025-09-05T12:00:00Z', 'weekly', now), true);
});

test('weekly not due within same week', () => {
  assert.strictEqual(isBackupDue('2025-09-10T12:00:00Z', 'weekly', now), false);
});

test('retention trims oldest beyond limit', () => {
  const files = [
    { name: 'a.sqlite', mtimeMs: 1 },
    { name: 'b.sqlite', mtimeMs: 3 },
    { name: 'c.sqlite', mtimeMs: 2 },
  ];
  const deletes = computeRetentionDeletes(files, 2);
  assert.deepStrictEqual(deletes, ['a.sqlite']);
});

console.log('All backup policy tests passed');

