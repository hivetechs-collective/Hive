/**
 * Memory Service Entry Point
 * Can be run as a child process or standalone
 */

import MemoryServiceServer from './server';

import { logger } from '../utils/SafeLogger';

logger.info('[MemoryService] Process started, checking environment...');
logger.info('[MemoryService] Environment:', {
  MEMORY_SERVICE_PORT: process.env.MEMORY_SERVICE_PORT,
  PORT: process.env.PORT,
  IS_CHILD: !!process.send
});

// Port MUST be provided by ProcessManager - no hardcoded fallback
const port = parseInt(process.env.MEMORY_SERVICE_PORT || process.env.PORT || '0');
if (!port) {
  logger.error('[MemoryService] No port provided! PORT or MEMORY_SERVICE_PORT must be set');
  process.exit(1);
}

logger.info('[MemoryService] Starting Memory Service...');
logger.info('[MemoryService] Port:', port);
logger.info('[MemoryService] Database: via IPC to main process');
logger.info('[MemoryService] Creating server instance...');

const server = new MemoryServiceServer(port);

logger.info('[MemoryService] Server instance created, calling start()...');

// Start the server
server.start().then(() => {
  logger.info('[MemoryService] Service started successfully');
  logger.info('[MemoryService] Server listening on port:', port);
  
  // Send ready signal to parent process if running as child
  if (process.send) {
    logger.info('[MemoryService] Sending ready signal to parent process...');
    process.send({ type: 'ready', port });
    logger.info('[MemoryService] Ready signal sent');
  } else {
    logger.info('[MemoryService] Running standalone (no parent process)');
  }
}).catch(error => {
  logger.error('[MemoryService] Failed to start:', error);
  logger.error('[MemoryService] Error details:', error.stack || error);
  process.exit(1);
});

// Handle shutdown gracefully
process.on('SIGTERM', async () => {
  logger.info('[MemoryService] Received SIGTERM, shutting down...');
  await server.stop();
  process.exit(0);
});

process.on('SIGINT', async () => {
  logger.info('[MemoryService] Received SIGINT, shutting down...');
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