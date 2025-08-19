/**
 * Memory Service Entry Point
 * Can be run as a child process or standalone
 */

import MemoryServiceServer from './server';

const port = parseInt(process.env.MEMORY_SERVICE_PORT || '3457');

console.log('[MemoryService] Starting Memory Service...');
console.log('[MemoryService] Port:', port);
console.log('[MemoryService] Database: via IPC to main process');

const server = new MemoryServiceServer(port);

// Start the server
server.start().then(() => {
  console.log('[MemoryService] Service started successfully');
  
  // Send ready signal to parent process if running as child
  if (process.send) {
    process.send({ type: 'ready', port });
  }
}).catch(error => {
  console.error('[MemoryService] Failed to start:', error);
  process.exit(1);
});

// Handle shutdown gracefully
process.on('SIGTERM', async () => {
  console.log('[MemoryService] Received SIGTERM, shutting down...');
  await server.stop();
  process.exit(0);
});

process.on('SIGINT', async () => {
  console.log('[MemoryService] Received SIGINT, shutting down...');
  await server.stop();
  process.exit(0);
});

// Handle messages from parent process
if (process.send) {
  process.on('message', (msg: any) => {
    if (msg.type === 'shutdown') {
      server.stop().then(() => {
        process.exit(0);
      });
    }
  });
}

export default server;