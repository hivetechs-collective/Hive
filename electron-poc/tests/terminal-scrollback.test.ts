import { TermBuffer } from '../src/terminal/core/buffer';

function assert(cond: any, msg: string) { if (!cond) throw new Error(msg); }

(async () => {
  const buf = new TermBuffer(() => {});
  buf.setSize(5, 3); // cols=5, rows=3
  (buf as any).scrollback = 4; // allow 4 extra lines beyond rows

  // Emit 8 lines; with rows=3 and scrollback=4, lines should cap at 7
  for (let i = 0; i < 8; i++) {
    buf.feed(`L${i}\n`);
  }
  const total = buf.lines.length;
  assert(total === 7, `Expected 7 lines (rows+scrollback=7), got ${total}`);
  // The visible bottom 3 lines should be L5, L6, '' (since last feed added a blank after \n)
  const all = buf.lines.map(l => l.cells.join('')).join('\n');
  // Ensure some recent content (L6) appears in the buffer
  assert(all.includes('L6'), `Expected recent content (L6) present in buffer`);
  console.log('[terminal-scrollback.test] OK');
})();
