"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GitOperationQueue = void 0;
class GitOperationQueue {
    constructor() {
        this.queue = [];
        this.running = false;
        this.currentOperation = null;
        this.operationCounter = 0;
        console.log('[GitOperationQueue] Initialized');
    }
    async enqueue(type, fn) {
        return new Promise((resolve, reject) => {
            const operation = {
                id: `${type}-${++this.operationCounter}`,
                type,
                fn,
                resolve,
                reject,
                timestamp: Date.now(),
            };
            console.log(`[GitOperationQueue] Enqueuing operation: ${operation.id}`);
            this.queue.push(operation);
            // Start processing if not already running
            if (!this.running) {
                this.processQueue();
            }
        });
    }
    async processQueue() {
        if (this.running || this.queue.length === 0) {
            return;
        }
        this.running = true;
        while (this.queue.length > 0) {
            const operation = this.queue.shift();
            this.currentOperation = operation;
            console.log(`[GitOperationQueue] Processing operation: ${operation.id}`);
            const startTime = Date.now();
            try {
                const result = await operation.fn();
                const duration = Date.now() - startTime;
                console.log(`[GitOperationQueue] Operation ${operation.id} completed in ${duration}ms`);
                operation.resolve(result);
            }
            catch (error) {
                const duration = Date.now() - startTime;
                console.error(`[GitOperationQueue] Operation ${operation.id} failed after ${duration}ms:`, error);
                operation.reject(error);
            }
            this.currentOperation = null;
        }
        this.running = false;
        console.log('[GitOperationQueue] Queue processing complete');
    }
    getCurrentOperation() {
        return this.currentOperation;
    }
    getQueueLength() {
        return this.queue.length;
    }
    clearQueue() {
        console.log(`[GitOperationQueue] Clearing ${this.queue.length} pending operations`);
        // Reject all pending operations
        for (const operation of this.queue) {
            operation.reject(new Error('Operation cancelled - queue cleared'));
        }
        this.queue = [];
    }
    // Priority operations that can skip the queue (like status)
    async executePriority(fn) {
        // If nothing is running, execute immediately
        if (!this.running) {
            return fn();
        }
        // Otherwise, wait for current operation then execute
        return new Promise((resolve, reject) => {
            const checkAndExecute = async () => {
                if (!this.running) {
                    try {
                        const result = await fn();
                        resolve(result);
                    }
                    catch (error) {
                        reject(error);
                    }
                }
                else {
                    setTimeout(checkAndExecute, 100);
                }
            };
            checkAndExecute();
        });
    }
}
exports.GitOperationQueue = GitOperationQueue;
