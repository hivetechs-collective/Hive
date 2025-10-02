import { TermBuffer } from '../src/terminal/core/buffer';

function assert(cond: any, msg: string) { if (!cond) throw new Error(msg); }

(async () => {
  const buf = new TermBuffer(() => {});
  buf.setSize(20, 2);
  // OSC 0;title BEL should be ignored
  buf.feed('Hello');
  buf.feed('\x1b]0;malicious title\x07');
  buf.feed(' World');
  const line = buf.lines[0].cells.join('');
  assert(line.startsWith('Hello World'), `Expected OSC ignored, got: ${JSON.stringify(line)}`);

  // OSC terminated by ST (ESC \\) should be ignored
  const buf2 = new TermBuffer(() => {});
  buf2.setSize(20, 2);
  buf2.feed('A');
  buf2.feed('\x1b]133;ignored payload\x1b\\');
  buf2.feed('B');
  const line2 = buf2.lines[0].cells.join('');
  assert(line2.startsWith('AB'), `Expected OSC ST ignored, got: ${JSON.stringify(line2)}`);
  console.log('[terminal-security-osc.test] OK');
})();

