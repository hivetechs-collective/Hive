"use strict";
/**
 * Memory Service Entry Point
 * Can be run as a child process or standalone
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const server_1 = __importDefault(require("./server"));
const SafeLogger_1 = require("../utils/SafeLogger");
const port = parseInt(process.env.MEMORY_SERVICE_PORT || '3457');
SafeLogger_1.logger.info('[MemoryService] Starting Memory Service...');
SafeLogger_1.logger.info('[MemoryService] Port:', port);
SafeLogger_1.logger.info('[MemoryService] Database: via IPC to main process');
const server = new server_1.default(port);
// Start the server
server.start().then(() => {
    SafeLogger_1.logger.info('[MemoryService] Service started successfully');
    // Send ready signal to parent process if running as child
    if (process.send) {
        process.send({ type: 'ready', port });
    }
}).catch(error => {
    SafeLogger_1.logger.error('[MemoryService] Failed to start:', error);
    process.exit(1);
});
// Handle shutdown gracefully
process.on('SIGTERM', () => __awaiter(void 0, void 0, void 0, function* () {
    SafeLogger_1.logger.info('[MemoryService] Received SIGTERM, shutting down...');
    yield server.stop();
    process.exit(0);
}));
process.on('SIGINT', () => __awaiter(void 0, void 0, void 0, function* () {
    SafeLogger_1.logger.info('[MemoryService] Received SIGINT, shutting down...');
    yield server.stop();
    process.exit(0);
}));
// Handle messages from parent process
if (process.send) {
    process.on('message', (msg) => {
        if (msg.type === 'shutdown') {
            server.stop().then(() => {
                process.exit(0);
            });
        }
    });
}
exports.default = server;
//# sourceMappingURL=index.js.map