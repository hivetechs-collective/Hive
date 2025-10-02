import { TermBuffer } from '../src/terminal/core/buffer';

function assert(cond: any, msg: string) { if (!cond) throw new Error(msg); }

function rand(n: number) { return Math.floor(Math.random() * n); }

(async () => {
  const buf = new TermBuffer(() => {});
  buf.setSize(10, 5);
  const parts: string[] = [];
  const finals = 'ABCDEFGHJKLmno';
  const csis = ['\x1b[', '\x1b[?', '\x1b[>'];
  for (let i = 0; i < 200; i++) {
    const t = rand(6);
    if (t < 2) {
      // Printable chunk
      parts.push(String.fromCharCode(32 + rand(90)));
    } else if (t === 2) {
      // CSI with random params and final
      const pre = csis[rand(csis.length)];
      const params = Array.from({ length: rand(4) }, () => String(rand(200))).join(';');
      const fin = finals[rand(finals.length)];
      parts.push(pre + params + fin);
    } else if (t === 3) {
      // ESC single
      const escs = ['\x1b7','\x1b8','\x1bM','\x1bE'];
      parts.push(escs[rand(escs.length)]);
    } else if (t === 4) {
      // OSC noise either BEL or ST terminated
      if (rand(2) === 0) parts.push(`\x1b]1337;param=val\x07`);
      else parts.push(`\x1b]1;title\x1b\\`);
    } else {
      parts.push('\n');
    }
  }
  const payload = parts.join('');
  buf.feed(payload);
  // Just assert feed completed and buffer invariants hold
  assert(buf.lines.length >= buf.rows, 'Buffer has at least rows lines');
  for (const line of buf.lines) {
    assert(line.cells.length === buf.cols, 'cells len matches cols');
    assert(line.attrs.length === buf.cols, 'attrs len matches cols');
  }
  console.log('[terminal-fuzz.test] OK');
})();

