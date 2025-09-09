import { BrowserWindow, app, ipcMain } from 'electron';
import * as path from 'path';
import { logger } from '../utils/SafeLogger';
import { PidTracker } from '../utils/PidTracker';
import { PortManager } from '../utils/PortManager';

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
        registerConsensusHandlers: () => void;
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
                logger.info('[Startup] About to register consensus handlers...');
                this.initFunctions.registerConsensusHandlers();  // Register consensus handlers AFTER database init
                logger.info('[Startup] Consensus handlers registered!')
                
                // REMOVED: WebSocket backend port handler - using DirectConsensusEngine now
                
                // Register Memory Service port handler (fully dynamic)
                ipcMain.handle('memory-service-port', async () => {
                    const memoryInfo = this.initFunctions.processManager.getProcessStatus('memory-service');
                    const port = memoryInfo?.port;
                    if (!port) {
                        logger.warn('[Startup] Memory Service port requested but not available');
                        throw new Error('Memory Service not running');
                    }
                    logger.info(`[Main] Memory Service port requested: ${port}`);
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
            required: true  // Memory Service is REQUIRED - core functionality
        },
        {
            id: 'consensusEngine',
            name: 'Direct Consensus Engine',
            init: async () => {
                logger.info('[Startup] Initializing Direct Consensus Engine...');
                // Simple initialization - no separate process (keep simple per architecture)
            },
            weight: 25,
            required: true   // Consensus is REQUIRED for consensus functionality
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
            logger.info('[StartupOrchestrator] Starting showSplashAndInitialize');
            
            // Create splash window
            logger.info('[StartupOrchestrator] Creating splash window...');
            this.createSplashWindow();
            
            // Pre-scan ports for optimized allocation (MUST complete before services start)
            logger.info('[StartupOrchestrator] Scanning available ports...');
            this.updateSplash(5, 'Scanning available ports...');
            await PortManager.initialize(); // Wait for port scan to complete FIRST
            logger.info('[StartupOrchestrator] Port scanning complete');
            
            // NOW initialize services with pre-scanned ports available
            logger.info('[StartupOrchestrator] Starting service initialization...');
            await this.initializeServices();
            logger.info('[StartupOrchestrator] Service initialization complete');
            
            // Final preparation
            this.updateSplash(95, 'Preparing workspace...');
            await this.delay(300);
            
            this.updateSplash(100, 'Ready');
            await this.delay(200);
            
            // Create and show main window
            logger.info('[StartupOrchestrator] Transitioning to main window...');
            await this.transitionToMain(createMainWindow);
            
            const totalTime = Date.now() - this.startTime;
            logger.info(`[Startup] Application ready in ${totalTime}ms`);
            
            return { success: true };
            
        } catch (error) {
            logger.error('[Startup] Initialization failed:', error);
            logger.error('[Startup] Error stack:', (error as Error).stack);
            this.showError(error as Error);
            return { 
                success: false, 
                error: error as Error 
            };
        }
    }
    
    private createSplashWindow(): void {
        try {
            // In production, startup files are in the unpacked directory
            const isPackaged = app.isPackaged;
            let preloadPath: string;
            let startupPath: string;
            
            logger.info(`[StartupOrchestrator] Creating splash window, isPackaged: ${isPackaged}`);
            
            if (isPackaged) {
                // Production: files are in .webpack/renderer/ inside the asar
                // Electron can load directly from asar, no need to unpack
                const appPath = app.getAppPath();
                preloadPath = path.join(appPath, '.webpack', 'renderer', 'startup-preload.js');
                startupPath = path.join(appPath, '.webpack', 'renderer', 'startup.html');
            } else {
                // Development: files are in .webpack/renderer/
                preloadPath = path.join(__dirname, '..', '..', 'startup-preload.js');
                startupPath = path.join(__dirname, '..', '..', 'startup.html');
            }
            
            logger.info(`[StartupOrchestrator] Splash preload path: ${preloadPath}`);
            logger.info(`[StartupOrchestrator] Splash HTML path: ${startupPath}`);
        
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
                preload: preloadPath
            }
        });
        
        this.splashWindow.loadFile(startupPath);
        
        // Prevent closing during startup
        this.splashWindow.on('close', (e) => {
            if (!this.mainWindow) {
                e.preventDefault();
            }
        });
            
            logger.info('[StartupOrchestrator] Splash window created successfully');
        } catch (error) {
            logger.error('[StartupOrchestrator] Failed to create splash window:', error);
            logger.error('[StartupOrchestrator] Error stack:', (error as Error).stack);
            throw error;
        }
    }
    
    private async initializeServices(): Promise<void> {
        let progress = 0;
        
        logger.info(`[StartupOrchestrator] Starting service initialization, ${this.requiredServices.length} services to start`);
        
        for (const service of this.requiredServices) {
            try {
                logger.info(`[StartupOrchestrator] Starting service: ${service.name} (id: ${service.id})`);
                
                // Update status
                this.updateSplash(progress, `Starting ${service.name}...`);
                
                // Initialize service
                const startTime = Date.now();
                logger.info(`[StartupOrchestrator] Calling init() for ${service.name}...`);
                await service.init();
                const duration = Date.now() - startTime;
                logger.info(`[StartupOrchestrator] ${service.name} init() completed in ${duration}ms`);
                
                // No timeout - just wait for ProcessManager to report success
                // The service.init() already waits for ProcessManager to confirm the service is ready
                
                // Update progress
                progress += service.weight;
                this.updateSplash(progress, `${service.name} ready`);
                logger.info(`[StartupOrchestrator] ${service.name} ready, progress: ${progress}%`);
                
                // Small delay for visual feedback
                await this.delay(100);
                
                logger.info(`[StartupOrchestrator] Moving to next service after ${service.name}...`);
                
            } catch (error) {
                logger.error(`[StartupOrchestrator] Service ${service.name} failed:`, error);
                if (service.required) {
                    throw new Error(`Failed to start ${service.name}: ${error}`);
                } else {
                    logger.warn(`[StartupOrchestrator] Optional service ${service.name} failed (continuing):`, error);
                    progress += service.weight; // Still add progress for optional services
                }
            }
        }
        
        logger.info('[StartupOrchestrator] All services initialized successfully');
    }
    
    private async startMemoryService(): Promise<void> {
        const processManager = this.initFunctions.processManager;
        
        logger.info('[StartupOrchestrator] Starting Memory Service initialization...');
        
        // Set up progress listener for memory service
        const progressHandler = (data: any) => {
            logger.info('[StartupOrchestrator] Memory Service progress event:', data);
            if (data.name === 'memory-service') {
                const basePercent = 45; // Start at 45% for memory service
                
                switch (data.status) {
                    case 'port-check':
                        logger.info(`[StartupOrchestrator] Memory Service checking port ${data.port}`);
                        this.updateSplash(basePercent + 5, `Memory Service on port ${data.port}...`);
                        break;
                    case 'ready':
                        logger.info('[StartupOrchestrator] Memory Service reported ready');
                        this.updateSplash(basePercent + 15, 'Memory Service ready');
                        break;
                    default:
                        logger.info(`[StartupOrchestrator] Memory Service status: ${data.status}`);
                }
            }
        };
        
        processManager.on('process:progress', progressHandler);
        
        try {
            logger.info('[StartupOrchestrator] Calling processManager.startProcess("memory-service")...');
            // Start the memory service through ProcessManager
            const started = await processManager.startProcess('memory-service');
            logger.info(`[StartupOrchestrator] Memory Service start result: ${started}`);
            
            if (!started) {
                logger.error('[StartupOrchestrator] Memory Service failed to start');
                throw new Error('Failed to start Memory Service');
            }
            
            logger.info('[StartupOrchestrator] Memory Service started successfully, returning from startMemoryService()');
        } catch (error) {
            logger.error('[StartupOrchestrator] Error starting Memory Service:', error);
            throw error;
        } finally {
            processManager.off('process:progress', progressHandler);
            logger.info('[StartupOrchestrator] Memory Service initialization complete');
        }
    }
    
    // REMOVED: startBackendServer() method - using DirectConsensusEngine instead of separate backend process
    // Following architecture principle: keep simple, no separate process needed
    
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