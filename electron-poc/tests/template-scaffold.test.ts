import assert from 'assert';
import * as fs from 'fs';
import * as path from 'path';
import { getScaffoldFiles, TemplateKind } from '../src/utils/template-scaffold';

function test(name: string, fn: () => Promise<void> | void) {
  Promise.resolve().then(fn).then(() => console.log(`✓ ${name}`)).catch((e) => { console.error(`✗ ${name}`); throw e; });
}

function withTempDir(prefix: string): { dir: string; cleanup: () => void } {
  const base = path.join(process.cwd(), '.tmp-template-test');
  if (!fs.existsSync(base)) fs.mkdirSync(base);
  const dir = path.join(base, `${prefix}-${Date.now()}`);
  fs.mkdirSync(dir);
  return { dir, cleanup: () => { fs.rmSync(dir, { recursive: true, force: true }); } };
}

async function materialize(files: ReturnType<typeof getScaffoldFiles>, root: string) {
  for (const f of files) {
    const dest = path.join(root, f.path);
    if (f.isDir) {
      if (!fs.existsSync(dest)) fs.mkdirSync(dest, { recursive: true });
    } else {
      const pdir = path.dirname(dest);
      if (!fs.existsSync(pdir)) fs.mkdirSync(pdir, { recursive: true });
      fs.writeFileSync(dest, f.content || '');
    }
  }
}

(['node','python','rust','empty'] as TemplateKind[]).forEach((tpl) => {
  test(`template scaffold: ${tpl}`, async () => {
    const { dir, cleanup } = withTempDir(tpl);
    try {
      const files = getScaffoldFiles(tpl, `proj-${tpl}`);
      await materialize(files, dir);
      // sanity checks
      if (tpl === 'node') {
        assert.ok(fs.existsSync(path.join(dir, 'package.json')));
        assert.ok(fs.existsSync(path.join(dir, 'index.js')));
      }
      if (tpl === 'python') {
        assert.ok(fs.existsSync(path.join(dir, 'main.py')));
      }
      if (tpl === 'rust') {
        assert.ok(fs.existsSync(path.join(dir, 'Cargo.toml')));
        assert.ok(fs.existsSync(path.join(dir, 'src/main.rs')));
      }
      assert.ok(fs.existsSync(path.join(dir, 'README.md')));
    } finally {
      cleanup();
    }
  });
});

console.log('All template scaffold tests scheduled');

