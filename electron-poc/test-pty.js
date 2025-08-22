const pty = require('node-pty');

console.log('Testing node-pty...');

try {
  const shell = '/bin/zsh';
  const args = [];
  const cwd = '/Users/veronelazio';
  
  console.log(`Spawning: ${shell} with args: ${JSON.stringify(args)} in ${cwd}`);
  
  const ptyProcess = pty.spawn(shell, args, {
    name: 'xterm-256color',
    cols: 80,
    rows: 30,
    cwd: cwd,
    env: process.env
  });
  
  console.log(`Success! PID: ${ptyProcess.pid}`);
  
  ptyProcess.onData((data) => {
    console.log('PTY output:', data);
  });
  
  setTimeout(() => {
    ptyProcess.write('echo "Hello from PTY"\r');
  }, 100);
  
  setTimeout(() => {
    ptyProcess.kill();
    console.log('PTY killed');
    process.exit(0);
  }, 2000);
  
} catch (error) {
  console.error('Failed to spawn PTY:', error);
  console.error('Error details:', {
    message: error.message,
    code: error.code,
    errno: error.errno,
    syscall: error.syscall,
    stack: error.stack
  });
}