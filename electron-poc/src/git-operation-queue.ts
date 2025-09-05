export type OperationType = 'status' | 'stage' | 'unstage' | 'commit' | 'push' | 'pull' | 'fetch' | 'sync' | 'branch' | 'checkout' | 'clean';

export interface Operation {
  id: string;
  type: OperationType;
  fn: () => Promise<any>;
  resolve: (value: any) => void;
  reject: (error: any) => void;
  timestamp: number;
}

export class GitOperationQueue {
  private queue: Operation[] = [];
  private running: boolean = false;
  private currentOperation: Operation | null = null;
  private operationCounter: number = 0;

  constructor() {
    console.log('[GitOperationQueue] Initialized');
  }

  async enqueue<T>(type: OperationType, fn: () => Promise<T>): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      const operation: Operation = {
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

  private async processQueue(): Promise<void> {
    if (this.running || this.queue.length === 0) {
      return;
    }

    this.running = true;

    while (this.queue.length > 0) {
      const operation = this.queue.shift()!;
      this.currentOperation = operation;

      console.log(`[GitOperationQueue] Processing operation: ${operation.id}`);
      const startTime = Date.now();

      try {
        const result = await operation.fn();
        const duration = Date.now() - startTime;
        console.log(`[GitOperationQueue] Operation ${operation.id} completed in ${duration}ms`);
        operation.resolve(result);
      } catch (error) {
        const duration = Date.now() - startTime;
        console.error(`[GitOperationQueue] Operation ${operation.id} failed after ${duration}ms:`, error);
        operation.reject(error);
      }

      this.currentOperation = null;
    }

    this.running = false;
    console.log('[GitOperationQueue] Queue processing complete');
  }

  getCurrentOperation(): Operation | null {
    return this.currentOperation;
  }

  getQueueLength(): number {
    return this.queue.length;
  }

  clearQueue(): void {
    console.log(`[GitOperationQueue] Clearing ${this.queue.length} pending operations`);
    
    // Reject all pending operations
    for (const operation of this.queue) {
      operation.reject(new Error('Operation cancelled - queue cleared'));
    }
    
    this.queue = [];
  }

  // Priority operations that can skip the queue (like status)
  async executePriority<T>(fn: () => Promise<T>): Promise<T> {
    // If nothing is running, execute immediately
    if (!this.running) {
      return fn();
    }

    // Otherwise, wait for current operation then execute
    return new Promise<T>((resolve, reject) => {
      const checkAndExecute = async () => {
        if (!this.running) {
          try {
            const result = await fn();
            resolve(result);
          } catch (error) {
            reject(error);
          }
        } else {
          setTimeout(checkAndExecute, 100);
        }
      };
      
      checkAndExecute();
    });
  }
}