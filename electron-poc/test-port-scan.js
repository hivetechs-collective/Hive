const net = require('net');

console.log('Testing port scanning...');

async function checkPort(port) {
  return new Promise((resolve) => {
    const server = net.createServer();
    const timeout = setTimeout(() => {
      console.log(`Port ${port} timed out`);
      server.close();
      resolve(false);
    }, 50);
    
    server.once('error', () => {
      clearTimeout(timeout);
      resolve(false);
    });
    
    server.once('listening', () => {
      clearTimeout(timeout);
      server.close();
      resolve(true);
    });
    
    server.listen(port, '127.0.0.1');
  });
}

async function test() {
  const ports = [3000, 3001, 3002, 7100, 7101];
  for (const port of ports) {
    const available = await checkPort(port);
    console.log(`Port ${port}: ${available ? 'available' : 'in use'}`);
  }
}

test().then(() => {
  console.log('Test complete');
  process.exit(0);
}).catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});