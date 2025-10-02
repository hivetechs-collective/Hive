import { TermBuffer } from '../src/terminal/core/buffer';

function assert(cond: any, msg: string) { if (!cond) throw new Error(msg); }

(async () => {
  const buf = new TermBuffer(() => {});
  buf.setSize(10, 3);
  assert(buf.cols === 10 && buf.rows === 3, 'Initial size 10x3');

  // Fill with some content then resize smaller
  buf.feed('abcdefg');
  buf.feed('\n');
  buf.feed('hijklmn');
  buf.setSize(5, 2);
  assert(buf.cols === 5 && buf.rows === 2, 'Resized to 5x2');
  assert(buf.lines.length === 2, 'Lines array matches rows');

  // Resize larger
  buf.setSize(8, 4);
  assert(buf.cols === 8 && buf.rows === 4, 'Resized to 8x4');
  assert(buf.lines.length === 4, 'Lines grew to new rows');

  console.log('[terminal-resize.test] OK');
})();

