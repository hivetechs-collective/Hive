import { BrowserWindow, app, ipcMain } from 'electron';
import * as path from 'path';
import { logger } from '../utils/SafeLogger';
import { PidTracker } from '../utils/PidTracker';

interface ServiceCheck {
    id: string;
    name: string;
    init: () => Promise<void>;
    verify?: () => Promise<boolean>;
    weight: number;
    required: boolean;
}

interface StartupResult {
    success: boolean;
    error?: Error;
    stage?: string;
}

export class StartupOrchestrator {
    private splashWindow: BrowserWindow | null = null;
    private mainWindow: BrowserWindow | null = null;
    private startTime: number = Date.now();
    private initFunctions: any;
    
    constructor(initFunctions: {
        initDatabase: () => void;
        initializeProcessManager: () => void;
        registerMemoryServiceHandlers: () => void;
        registerGitHandlers: () => void;
        registerFileSystemHandlers: () => void;
        registerDialogHandlers: () => void;
        registerSimpleCliToolHandlers: () => void;
        processManager: any;
    }) {
        this.initFunctions = initFunctions;
    }
    
    private requiredServices: ServiceCheck[] = [
        {
            id: 'database',
            name: 'Database',
            init: async () => {
                logger.info('[Startup] Initializing database...');
                await PidTracker.cleanupOrphans();
                this.initFunctions.initDatabase();
            },
            weight: 15,
            required: true
        },
        {
            id: 'processManager',
            name: 'Process Manager',
            init: async () => {
                logger.info('[Startup] Starting Process Manager...');
                this.initFunctions.initializeProcessManager();
            },
            weight: 10,
            required: true
        },
        {
            id: 'ipcHandlers',
            name: 'IPC Handlers',
            init: async () => {
                logger.info('[Startup] Registering IPC handlers...');
                this.initFunctions.registerMemoryServiceHandlers();
                this.initFunctions.registerGitHandlers();
                this.initFunctions.registerFileSystemHandlers();
                this.initFunctions.registerDialogHandlers();
                this.initFunctions.registerSimpleCliToolHandlers();
                
                // Register WebSocket backend port handler
                ipcMain.handle('websocket-backend-port', async () => {
                    const backendInfo = this.initFunctions.processManager.getProcessStatus('websocket-backend');
                    const port = backendInfo?.port || 8765;
                    logger.info(`[Main] WebSocket backend port requested: ${port}`);
                    return port;
                });
            },
            weight: 10,
            required: true
        },
        {
            id: 'memoryService',
            name: 'Memory Service',
            init: async () => {
                logger.info('[Startup] Launching Memory Service...');
                await this.startMemoryService();
            },
            weight: 20,
            required: false
        },
        {
            id: 'backendServer',
            name: 'Backend Server & Consensus Engine',
            init: async () => {
                logger.info('[Startup] Starting Backend Server...');
                await this.startBackendServer();
            },
            weight: 25,
            required: true
        },
        {
            id: 'cliTools',
            name: 'AI CLI Tools',
            init: async () => {
                logger.info('[Startup] Detecting AI CLI tools...');
                await this.detectTools();
            },
            weight: 15,
            required: false
        }
    ];
    
    async showSplashAndInitialize(createMainWindow: (show: boolean) => BrowserWindow): Promise<StartupResult> {
        try {
            // Create splash window
            this.createSplashWindow();
            
            // Initialize all services
            await this.initializeServices();
            
            // Final preparation
            this.updateSplash(95, 'Preparing workspace...');
            await this.delay(300);
            
            this.updateSplash(100, 'Ready');
            await this.delay(200);
            
            // Create and show main window
            await this.transitionToMain(createMainWindow);
            
            const totalTime = Date.now() - this.startTime;
            logger.info(`[Startup] Application ready in ${totalTime}ms`);
            
            return { success: true };
            
        } catch (error) {
            logger.error('[Startup] Initialization failed:', error);
            this.showError(error as Error);
            return { 
                success: false, 
                error: error as Error 
            };
        }
    }
    
    private createSplashWindow(): void {
        this.splashWindow = new BrowserWindow({
            width: 600,
            height: 500,
            frame: false,
            center: true,
            resizable: false,
            backgroundColor: '#0E1414',
            alwaysOnTop: true,
            skipTaskbar: true,
            webPreferences: {
                nodeIntegration: false,
                contextIsolation: true,
                preload: path.join(__dirname, '..', '..', 'startup-preload.js')
            }
        });
        
        const startupPath = path.join(__dirname, '..', '..', 'startup.html');
        this.splashWindow.loadFile(startupPath);
        
        // Prevent closing during startup
        this.splashWindow.on('close', (e) => {
            if (!this.mainWindow) {
                e.preventDefault();
            }
        });
    }
    
    private async initializeServices(): Promise<void> {
        let progress = 0;
        
        for (const service of this.requiredServices) {
            try {
                // Update status
                this.updateSplash(progress, `Starting ${service.name}...`);
                
                // Initialize service
                await service.init();
                
                // No timeout - just wait for ProcessManager to report success
                // The service.init() already waits for ProcessManager to confirm the service is ready
                
                // Update progress
                progress += service.weight;
                this.updateSplash(progress, `${service.name} ready`);
                
                // Small delay for visual feedback
                await this.delay(100);
                
            } catch (error) {
                if (service.required) {
                    throw new Error(`Failed to start ${service.name}: ${error}`);
                } else {
                    logger.warn(`[Startup] Optional service ${service.name} failed:`, error);
                    progress += service.weight; // Still add progress for optional services
                }
            }
        }
    }
    
    private async startMemoryService(): Promise<void> {
        const processManager = this.initFunctions.processManager;
        
        // Set up progress listener for memory service
        const progressHandler = (data: any) => {
            if (data.name === 'memory-service') {
                const basePercent = 45; // Start at 45% for memory service
                
                switch (data.status) {
                    case 'port-check':
                        this.updateSplash(basePercent + 5, `Memory Service on port ${data.port}...`);
                        break;
                    case 'ready':
                        this.updateSplash(basePercent + 15, 'Memory Service ready');
                        break;
                }
            }
        };
        
        processManager.on('process:progress', progressHandler);
        
        try {
            // Start the memory service through ProcessManager
            const started = await processManager.startProcess('memory-service');
            if (!started) {
                throw new Error('Failed to start Memory Service');
            }
        } finally {
            processManager.off('process:progress', progressHandler);
        }
    }
    
    private async startBackendServer(): Promise<void> {
        // Listen for backend server progress updates
        const processManager = this.initFunctions.processManager;
        
        // Set up progress listener for detailed updates
        const progressHandler = (data: any) => {
            if (data.name === 'websocket-backend') {
                // Display real-time status from ProcessManager
                if (data.message) {
                    this.updateSplash(75, data.message);
                }
                
                // Update based on specific events
                switch (data.status) {
                    case 'starting':
                        this.updateSplash(70, 'Starting backend server...');
                        break;
                    case 'initializing':
                        this.updateSplash(75, 'Backend server initializing...');
                        break;
                    case 'database':
                        this.updateSplash(78, 'Connecting to database...');
                        break;
                    case 'consensus':
                        this.updateSplash(80, 'Initializing consensus engine...');
                        break;
                    case 'models':
                        this.updateSplash(85, 'Syncing AI models from OpenRouter...');
                        break;
                    case 'ai-helpers':
                        this.updateSplash(88, 'Loading AI helpers...');
                        break;
                    case 'port-check':
                        this.updateSplash(90, `Verifying port ${data.port}...`);
                        break;
                    case 'waiting':
                        // Just show we're still working, no timeout
                        this.updateSplash(92, 'Waiting for services to be ready...');
                        break;
                    case 'ready':
                        this.updateSplash(95, `Backend ready on port ${data.port}`);
                        break;
                }
            }
        };
        
        processManager.on('process:progress', progressHandler);
        
        try {
            // Start the backend server - ProcessManager will wait as long as needed
            const started = await processManager.startProcess('websocket-backend');
            if (!started) {
                throw new Error('Failed to start Backend Server');
            }
        } finally {
            // Clean up listener
            processManager.off('process:progress', progressHandler);
        }
    }
    
    private async detectTools(): Promise<void> {
        // Detect installed AI CLI tools - this is handled via IPC now
        logger.info('[Startup] Tool detection will happen after UI loads');
    }
    
    private async checkHealth(url: string): Promise<boolean> {
        try {
            const response = await fetch(url);
            return response.ok;
        } catch {
            return false;
        }
    }
    
    private async checkWebSocket(url: string): Promise<boolean> {
        return new Promise((resolve) => {
            try {
                const WebSocket = require('ws');
                const ws = new WebSocket(url);
                
                ws.on('open', () => {
                    ws.close();
                    resolve(true);
                });
                
                ws.on('error', (err: any) => {
                    // Ignore connection errors during startup
                    resolve(false);
                });
                
                // Timeout after 3 seconds per attempt
                setTimeout(() => {
                    try {
                        ws.close();
                    } catch {
                        // Ignore close errors
                    }
                    resolve(false);
                }, 3000);
            } catch {
                resolve(false);
            }
        });
    }
    
    
    private updateSplash(percent: number, status: string): void {
        if (this.splashWindow && !this.splashWindow.isDestroyed()) {
            this.splashWindow.webContents.send('startup-progress', {
                percent: Math.min(100, Math.round(percent)),
                status
            });
        }
    }
    
    private showError(error: Error): void {
        if (this.splashWindow && !this.splashWindow.isDestroyed()) {
            this.splashWindow.webContents.send('startup-error', {
                message: error.message
            });
        }
    }
    
    private async transitionToMain(createMainWindow: (show: boolean) => BrowserWindow): Promise<void> {
        // Create main window but don't show it yet
        this.mainWindow = createMainWindow(false);
        
        // Wait for main window to be ready
        await new Promise<void>((resolve) => {
            if (this.mainWindow) {
                this.mainWindow.once('ready-to-show', () => resolve());
            }
        });
        
        // Close splash and show main
        if (this.splashWindow && !this.splashWindow.isDestroyed()) {
            this.splashWindow.destroy();
            this.splashWindow = null;
        }
        
        if (this.mainWindow) {
            this.mainWindow.show();
            this.mainWindow.focus();
        }
    }
    
    private delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
    
    cleanup(): void {
        if (this.splashWindow && !this.splashWindow.isDestroyed()) {
            this.splashWindow.destroy();
        }
    }
}

export default StartupOrchestrator;