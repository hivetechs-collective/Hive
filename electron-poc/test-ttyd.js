const { spawn } = require('child_process');

console.log('Testing ttyd spawn...');

const ttydPath = '/opt/homebrew/bin/ttyd';
const args = [
  '--port', '7777',
  '--interface', '127.0.0.1',
  '--max-clients', '1',
  '--once',
  '--writable',
  'bash'
];

console.log(`Spawning: ${ttydPath} ${args.join(' ')}`);

const ttyd = spawn(ttydPath, args, {
  env: { ...process.env, TERM: 'xterm-256color' },
  stdio: ['ignore', 'pipe', 'pipe']
});

ttyd.on('error', (error) => {
  console.error('Failed to start ttyd:', error);
});

ttyd.stdout.on('data', (data) => {
  console.log('ttyd stdout:', data.toString());
});

ttyd.stderr.on('data', (data) => {
  console.log('ttyd stderr:', data.toString());
});

ttyd.on('close', (code) => {
  console.log(`ttyd process exited with code ${code}`);
});

console.log('ttyd PID:', ttyd.pid);
console.log('Open http://localhost:7777 in your browser to test');

// Keep the process alive
setTimeout(() => {
  console.log('Killing ttyd after 30 seconds...');
  ttyd.kill();
}, 30000);