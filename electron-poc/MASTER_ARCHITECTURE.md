# 🏗️ Hive Consensus - Master Architecture Document

## Table of Contents
1. [System Overview](#system-overview)
2. [Core Components](#core-components)
3. [Process Architecture](#process-architecture)
4. [Data Architecture](#data-architecture)
5. [Communication Layers](#communication-layers)
6. [User Interface Architecture](#user-interface-architecture)
7. [Memory Service Infrastructure](#memory-service-infrastructure)
8. [Git Integration Architecture](#git-integration-architecture)
9. [Consensus Engine Architecture](#consensus-engine-architecture)
10. [Security & Authentication](#security--authentication)
11. [Performance & Optimization](#performance--optimization)
12. [Development & Deployment](#development--deployment)
13. [CLI Tools Management](#cli-tools-management)
14. [AI Tools Launch Tracking & Database](#ai-tools-launch-tracking--database)
15. [CLI Tools Management UI](#cli-tools-management-ui)
16. [Future Enhancements](#future-enhancements)

---

## System Overview

### Purpose
Hive Consensus is an advanced AI-powered development environment that combines:
- **4-Stage Consensus AI Processing** (Generator → Refiner → Validator → Curator)
- **Universal Memory Infrastructure** (Memory-as-a-Service for AI tools)
- **VS Code-like Development Environment** in Electron
- **Deep Git Integration** with visual source control
- **Real-time Collaboration** between human and AI
- **Visual Startup Experience** with neural network animation (v1.8.0)

### Technology Stack
```
Frontend:
├── Electron (Desktop App Framework)
├── TypeScript (Primary Language)
├── HTML/CSS (UI Rendering)
├── WebSockets (Real-time Communication)
└── Monaco Editor (Code Editing)

Backend:
├── Node.js (Runtime)
├── Express (API Server for Memory Service)
├── SQLite (Local Database)
├── Cloudflare D1 (Remote Sync)
└── OpenRouter API (AI Model Access)

Infrastructure:
├── ProcessManager (Child Process Management)
├── PortManager (Port Conflict Resolution)
├── IPC (Inter-Process Communication)
└── File System Watchers
```

---

## Core Components

### 1. Main Process (Electron Main)
**Location**: `src/index.ts`
**Responsibilities**:
- Application lifecycle management
- Window creation and management
- IPC handler registration
- Database initialization
- Process orchestration
- Menu system
- File system operations

**Key Features**:
- Manages SQLite database connection
- Handles all file I/O operations
- Spawns and manages child processes
- Provides secure bridge to renderer

### 2. Renderer Process (UI)
**Location**: `src/renderer.ts`
**Responsibilities**:
- User interface rendering
- Event handling
- State management
- API communication
- Real-time updates

**Key Components**:
```typescript
├── Chat Interface (Consensus interaction)
├── File Explorer (Project navigation)
├── Editor Tabs (Multi-file editing)
├── Git UI (Source control)
├── Memory Dashboard (UMI visualization)
├── Analytics View (Usage statistics)
└── Settings Modal (Configuration)
```

### 3. Memory Service (Child Process)
**Location**: `src/memory-service/`
**Port**: FULLY DYNAMIC - NO HARDCODED PORTS
**Responsibilities**:
- Memory-as-a-Service API
- External tool integration
- Query processing
- Learning contribution
- Statistics tracking

**Port Allocation**:
- Receives port from ProcessManager via environment variable
- NEVER has fallback ports
- Exits immediately if no port provided
- Clients discover port via IPC

---

## Process Architecture

### Zero-Fallback Port Management Philosophy

**CORE PRINCIPLES**:
1. **NO HARDCODED PORTS** - Every port is dynamically allocated
2. **NO FALLBACK PORTS** - Services fail fast if port not allocated
3. **NO SAMPLE/DEFAULT PORTS** - Not even in comments or documentation
4. **FAIL FAST** - Services exit immediately without proper port
5. **IPC DISCOVERY** - All port discovery happens via IPC channels
6. **CONFIGURATION-DRIVEN** - Port ranges from config/environment only
7. **SYSTEM-ADAPTIVE** - Discovers what's available, doesn't assume
8. **NO MAGIC NUMBERS** - Even examples use variables/config references

**WHY THIS MATTERS**:
- **Portability**: App works on any system without port conflicts
- **Scalability**: Can run multiple instances with different configs
- **Security**: No predictable ports for attackers to target
- **Reliability**: Adapts to system constraints automatically
- **Maintainability**: Port ranges changed without code modifications

### Process Hierarchy (Fully Dynamic)
```
Electron Main Process (Orchestrator)
├── ProcessManager (Control Tower)
│   ├── Process Lifecycle Management
│   ├── Port Allocation via PortManager
│   ├── Health Monitoring
│   └── Crash Recovery
├── PortManager (Intelligence Layer)
│   ├── Pre-scan Port Pools
│   ├── Dynamic Allocation
│   ├── Ephemeral Fallback
│   └── Port Release & Cleanup
├── Memory Service (Node.js Child Process)
│   ├── Express Server (Dynamic Port from Pool)
│   ├── WebSocket Server
│   └── IPC Channel to Main
├── WebSocket Backend Server (Rust Binary)
│   ├── Dynamic Port from Pool
│   ├── Consensus Engine with AI Helpers
│   └── Deferred Initialization Architecture
└── File Watchers
    └── Git Status Monitor
```

### ProcessManager & PortManager Integration

#### The Control Tower Pattern
ProcessManager acts as the central control tower for all child processes, while PortManager provides intelligent port allocation services. This separation of concerns ensures clean architecture and maximum flexibility.

```typescript
// ProcessManager - The Control Tower
class ProcessManager extends EventEmitter {
  private processes: Map<string, ProcessInfo> = new Map();
  private configs: Map<string, ProcessConfig> = new Map();
  
  async startProcess(name: string): Promise<boolean> {
    const config = this.configs.get(name);
    const info = this.processes.get(name);
    
    // Step 1: Port Allocation (if needed)
    if (config.port !== undefined) {
      // Delegate to PortManager - no port numbers!
      const port = await PortManager.allocatePortForService(name);
      info.port = port;
      
      // Pass to service via environment
      env.PORT = port.toString();
      env[`${name.toUpperCase().replace('-', '_')}_PORT`] = port.toString();
    }
    
    // Step 2: Process Spawning
    const childProcess = this.spawnProcess(config, env);
    
    // Step 3: Monitoring & Recovery
    this.monitorProcess(name, childProcess);
  }
}
```

#### Dynamic Service Discovery (NO FALLBACKS)
Services discover their ports and peer services dynamically through IPC:

```typescript
// Service discovers its own port from environment - NO FALLBACK
const port = parseInt(process.env.PORT || process.env.MEMORY_SERVICE_PORT || '0');
if (!port) {
  logger.error('No port provided! Exiting...');
  process.exit(1);  // FAIL FAST - NO FALLBACK
}

// Renderer discovers service ports via IPC - NO FALLBACK
try {
  const memoryPort = await window.api.invoke('memory-service-port');
  if (!memoryPort) throw new Error('Memory service not running');
  // Use port...
} catch (error) {
  // Show error to user - NO FALLBACK CONNECTION ATTEMPT
  showError('Memory Service not available');
}
```

### ProcessManager - Central Control Tower (v3.0.0 - Zero Fallback)
**Location**: `src/utils/ProcessManager.ts`
**Initialization**: Created as singleton at app start in `src/index.ts`

The ProcessManager serves as the single source of truth for all process and port management across the application. It acts as the "control tower" coordinating all services, ports, and inter-process communication.

#### Architecture Principles
- **SINGLE INSTANCE**: Only ONE ProcessManager instance exists, created at app initialization
- **CENTRALIZED PORT MANAGEMENT**: All port allocations go through ProcessManager's PortManager
- **NO DUPLICATE INSTANCES**: All components receive the shared ProcessManager instance
- **EVENT-DRIVEN COORDINATION**: Real-time status updates via EventEmitter pattern
- **ZERO PORT ASSUMPTIONS**: ProcessManager never assumes port availability
- **STRICT SEQUENCING**: Dependencies initialized before dependents
- **FAIL-FAST PHILOSOPHY**: No retries with fallback ports

#### ProcessManager-PortManager Coordination Protocol

```typescript
class ProcessManager {
  // ProcessManager delegates ALL port decisions to PortManager
  async startProcess(name: string): Promise<boolean> {
    const config = this.configs.get(name);
    
    // Step 1: Request port from PortManager (no numbers!)
    let port: number | undefined;
    if (config.requiresPort) {
      port = await PortManager.allocatePortForService(name);
      if (!port) {
        logger.error(`[ProcessManager] No port available for ${name}`);
        this.emit('process-failed', { name, reason: 'no-port' });
        return false;  // FAIL - Don't try fallbacks
      }
    }
    
    // Step 2: Start process with allocated port
    const env = {
      ...process.env,
      ...(port && {
        PORT: port.toString(),
        [`${name.toUpperCase().replace(/-/g, '_')}_PORT`]: port.toString()
      })
    };
    
    // Step 3: Spawn and monitor
    const child = spawn(config.command, config.args, { env });
    
    // Step 4: Track allocation
    this.processes.set(name, {
      pid: child.pid,
      port,
      status: 'starting',
      startTime: Date.now()
    });
    
    return true;
  }
  
  // When process dies, release port immediately
  private handleProcessExit(name: string) {
    const info = this.processes.get(name);
    if (info?.port) {
      PortManager.releasePort(name);  // Port returns to pool
    }
    this.processes.delete(name);
    this.emit('process-stopped', { name });
  }
}
```

#### Key Components

##### 1. ProcessManager Singleton
```typescript
// Created once at app start (src/index.ts:39)
const processManager = new ProcessManager();

// Passed to all components that need it:
- StartupOrchestrator receives it in constructor
- Terminal handlers receive it via registerTerminalHandlers(mainWindow, processManager)
- Memory Service uses it for port allocation
- WebSocket Backend uses it for lifecycle management
```

##### 2. PortManager Integration
**Location**: `src/utils/PortManager.ts`

PortManager is a static utility class used by ProcessManager to:
- Track ALL allocated ports globally in a Map
- Check port availability both physically and in allocation registry
- Prevent port conflicts between services
- Automatically find next available port when conflicts occur

```typescript
class PortManager {
  private static allocatedPorts: Map<string, number> = new Map();
  
  static async allocatePort(config: PortConfig): Promise<number> {
    // 1. Check if port is already allocated to ANY service
    const isPortAllocatedToAnother = Array.from(this.allocatedPorts.values()).includes(port);
    
    // 2. Check if port is physically available
    if (!isPortAllocatedToAnother && await this.isPortAvailable(port)) {
      // 3. Allocate and track the port
      this.allocatedPorts.set(serviceName, port);
      return port;
    }
    
    // 4. Find next available port (7100, 7101, 7102, etc.)
    // Continues until finding both unallocated AND available port
  }
}
```

##### 3. TTYD Terminal Management
**Location**: `src/services/TTYDManager.ts`

TTYDManager receives the shared ProcessManager instance and uses PortManager for port allocation:
- Each terminal gets a unique port (7100, 7101, 7102, etc.)
- Port conflicts are automatically resolved
- Failed terminals properly release their ports

```typescript
// Terminal handler registration with shared ProcessManager
registerTerminalHandlers(mainWindow: BrowserWindow, processManager: ProcessManager) {
  // Initialize TTYDManager with shared ProcessManager
  ttydManager = new TTYDManager(processManager);
}
```

#### Process Lifecycle Management

##### Service Registration (NO PORT NUMBERS)
```typescript
processManager.registerProcess('memory-service', {
  scriptPath: memoryServicePath,
  // NO PORT SPECIFIED - PortManager handles it
  env: { /* environment variables */ }
});
```

##### Service Startup Flow
1. StartupOrchestrator requests service start
2. ProcessManager allocates port via PortManager
3. Process spawned with allocated port
4. ProcessManager monitors port availability
5. Status events emitted to StartupOrchestrator
6. Visual progress updated in splash screen

### PortManager - Intelligent Dynamic Port Allocation (v3.0.0)
**Location**: `src/utils/PortManager.ts`

#### Core Design Principles
1. **NO HARDCODED PORTS ANYWHERE** - Not even in examples or comments
2. **PRE-SCAN OPTIMIZATION** - All ports scanned at startup, not on-demand
3. **POOL-BASED ALLOCATION** - Pre-validated pools for instant allocation
4. **ZERO FALLBACKS** - Services fail if no port available
5. **STRICT ISOLATION** - Services never know about other services' ports

#### Architecture Overview

```typescript
class PortManager {
  // Port ranges loaded from configuration - NEVER hardcoded
  private static readonly PORT_RANGES: PortRange[] = this.loadPortRanges();
  
  // Pre-scanned available ports organized by service pool
  private static availablePortPool: Map<string, AvailablePort[]> = new Map();
  private static allocatedPorts: Map<string, AllocatedPort> = new Map();
  private static scanComplete: boolean = false;
  
  // Load from environment or config file - NO HARDCODED VALUES
  private static loadPortRanges(): PortRange[] {
    // In production: Load from config file or environment
    // Ranges are DISCOVERED, not assumed
    return ConfigLoader.getPortRanges() || this.discoverAvailableRanges();
  }
  
  // Discover available ranges by scanning system
  private static async discoverAvailableRanges(): Promise<PortRange[]> {
    // Scan common ranges and find available blocks
    // Create pools based on what's actually available
    // NO ASSUMPTIONS about specific port numbers
  }
}
```

#### Configuration-Driven Port Management

```yaml
# config/ports.yaml - Externalized configuration
port_ranges:
  memory_service:
    scan_start: ${MEMORY_PORT_START}  # From environment
    scan_end: ${MEMORY_PORT_END}      # From environment
    pool_size: 20                     # How many to pre-scan
    
  backend_service:
    scan_start: ${BACKEND_PORT_START}
    scan_end: ${BACKEND_PORT_END}
    pool_size: 10
    
  terminals:
    scan_start: ${TERMINAL_PORT_START}
    scan_end: ${TERMINAL_PORT_END}
    pool_size: 50
```

#### Service-Agnostic Port Allocation

```typescript
// Services never know their port ranges
// They just request a port for their service type
class MemoryService {
  constructor() {
    // Service doesn't know or care about port ranges
    const port = process.env.PORT || process.env.MEMORY_SERVICE_PORT;
    if (!port) {
      logger.error('No port provided by ProcessManager!');
      process.exit(1);
    }
    // Service uses whatever port was allocated
    this.startServer(parseInt(port));
  }
}
```

#### Startup Pre-Scan Process (Dynamic Discovery)

```typescript
// Called ONCE at app startup BEFORE any service initialization
await PortManager.initialize();

// What happens:
1. Load port range configuration (or discover)
2. Parallel scan of discovered ranges (not hardcoded)
3. Each port checked with adaptive timeout
4. Available ports added to service pools
5. Scan completes quickly for any number of ports
6. Services get INSTANT allocation from pre-validated pools

// The scan adapts to the system:
- On developer machine: May find different ranges available
- In production: Uses environment-specific configuration
- In containers: Adapts to container port mappings
- NO HARDCODED ASSUMPTIONS
```

#### Service Port Allocation Flow

```typescript
// When ProcessManager starts a service:
const port = await PortManager.allocatePortForService('memory-service');

// What happens:
1. Check if scan is complete (should be)
2. Get pool for service type
3. Pop first available port from pool
4. Mark as allocated with timestamp
5. Return port INSTANTLY (no network check needed)
6. If no ports available: FAIL FAST - NO FALLBACK
```

#### Port Release & Cleanup

```typescript
// When service stops/crashes:
PortManager.releasePort('memory-service');

// What happens:
1. Find allocated port for service
2. Return to available pool
3. Clear allocation record
4. Port immediately available for reuse
```

#### Zero-Fallback Philosophy in Practice

```typescript
// ❌ WRONG - NEVER DO THIS:
const port = process.env.PORT || 3457;  // HARDCODED FALLBACK
const ws = new WebSocket('ws://localhost:3457');  // HARDCODED PORT
const backendUrl = 'http://localhost:8765';  // HARDCODED URL

// ✅ RIGHT - FAIL FAST WITH DYNAMIC DISCOVERY:
const port = parseInt(process.env.PORT || '0');
if (!port) {
  logger.error('[Service] No port provided! Cannot start.');
  process.exit(1);  // FAIL IMMEDIATELY - NO GUESSING
}

// ✅ RIGHT - IPC DISCOVERY:
const memoryPort = await window.api.invoke('memory-service-port');
if (!memoryPort) throw new Error('Memory service not available');
const ws = new WebSocket(`ws://localhost:${memoryPort}`);

// ✅ RIGHT - ENVIRONMENT CONFIGURATION:
const portRange = {
  start: parseInt(process.env.SERVICE_PORT_START || '0'),
  end: parseInt(process.env.SERVICE_PORT_END || '0')
};
if (!portRange.start || !portRange.end) {
  throw new Error('Port range configuration missing');
}
```

#### IPC Port Discovery Protocol

```typescript
// Renderer needs Memory Service port:
const port = await window.api.invoke('memory-service-port');

// Main process handler:
ipcMain.handle('memory-service-port', async () => {
  const info = processManager.getProcessStatus('memory-service');
  if (!info?.port) {
    throw new Error('Memory Service not running');  // NO FALLBACK
  }
  return info.port;
});

// Renderer handles failure:
try {
  const port = await window.api.invoke('memory-service-port');
  connectToService(port);
} catch (error) {
  showServiceUnavailable();  // NO FALLBACK CONNECTION
}
```

##### Port Conflict Resolution
When pool is exhausted:
1. Log critical error
2. Show user error dialog
3. Service fails to start
4. NO FALLBACK TO RANDOM PORTS
5. NO SCANNING FOR NEW PORTS
6. Admin must resolve port conflicts

### Startup Orchestrator System (v2.0.0 - Event-Driven Architecture)
**Location**: `src/startup/StartupOrchestrator.ts`

The StartupOrchestrator provides a sophisticated visual loading experience with real-time progress reporting from ProcessManager, ensuring all services are fully initialized before displaying the main application.

#### Core Philosophy: No Timeouts, Only Real Status
- **NO ARBITRARY TIMEOUTS**: System waits as long as needed for services to be ready
- **EVENT-DRIVEN PROGRESS**: ProcessManager reports actual status, not estimated times
- **CONTINUOUS MONITORING**: Real-time updates flow from services to visual display
- **GRACEFUL DEGRADATION**: Optional services can fail without blocking startup

#### Architectural Components

##### 1. StartupOrchestrator Class
**Location**: `src/startup/StartupOrchestrator.ts`

```typescript
class StartupOrchestrator {
  private splashWindow: BrowserWindow | null = null;
  private mainWindow: BrowserWindow | null = null;
  private startTime: number = Date.now();
  private initFunctions: {
    initDatabase: () => void;
    initializeProcessManager: () => void;
    registerMemoryServiceHandlers: () => void;
    registerGitHandlers: () => void;
    registerFileSystemHandlers: () => void;
    registerDialogHandlers: () => void;
    registerSimpleCliToolHandlers: () => void;
    processManager: ProcessManager;
  };
  
  // Service definitions with NO verify functions - ProcessManager handles everything
  private requiredServices: ServiceCheck[] = [
    { id: 'database', name: 'Database', init: async () => {...}, weight: 15, required: true },
    { id: 'processManager', name: 'Process Manager', init: async () => {...}, weight: 10, required: true },
    { id: 'ipcHandlers', name: 'IPC Handlers', init: async () => {...}, weight: 10, required: true },
    { id: 'memoryService', name: 'Memory Service', init: async () => {...}, weight: 20, required: false },
    { id: 'backendServer', name: 'Backend Server & Consensus Engine', init: async () => {...}, weight: 25, required: true },
    { id: 'cliTools', name: 'AI CLI Tools', init: async () => {...}, weight: 15, required: false }
  ];
}
```

##### 2. Visual Display Architecture

###### Splash Window Configuration
```typescript
splashWindow = new BrowserWindow({
  width: 600,
  height: 500,
  frame: false,           // No window chrome
  center: true,          // Center on screen
  resizable: false,      // Fixed size
  backgroundColor: '#0E1414',  // Match app theme
  alwaysOnTop: true,     // Stay above other windows
  skipTaskbar: true,     // Don't show in taskbar
  webPreferences: {
    nodeIntegration: false,
    contextIsolation: true,
    preload: path.join(__dirname, '..', '..', 'startup-preload.js')
  }
});
```

###### Visual Layout (Fixed Positioning)
```css
.startup-container {
  width: 600px;
  max-height: 500px;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);  /* Perfect centering */
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-evenly;
  padding: 20px;
}

/* Prevents text from appearing off-screen */
.status-text {
  min-height: 20px;
  flex-shrink: 0;
  width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
```

##### 3. ProcessManager Integration

###### Event-Driven Progress Reporting
```typescript
// ProcessManager emits detailed progress events
processManager.emit('process:progress', {
  name: 'websocket-backend',
  status: 'initializing' | 'database' | 'consensus' | 'models' | 'ai-helpers' | 'waiting' | 'ready',
  message: 'Human-readable status message',
  port: 8765,
  percent?: number  // Optional progress percentage
});

// StartupOrchestrator listens and updates display
const progressHandler = (data: any) => {
  if (data.name === 'websocket-backend') {
    switch (data.status) {
      case 'starting':
        this.updateSplash(70, 'Starting backend server...');
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
      case 'waiting':
        this.updateSplash(92, 'Waiting for services to be ready...');
        break;
      case 'ready':
        this.updateSplash(95, `Backend ready on port ${data.port}`);
        break;
    }
  }
};
```

###### No-Timeout Service Verification
```typescript
// OLD (removed): Timeout-based verification
// const verified = await this.waitForService(service.verify, timeout);

// NEW: ProcessManager handles all verification internally
await this.startBackendServer();  // Returns only when TRULY ready

// ProcessManager's infinite loop until ready
while (!portReady) {
  attempts++;
  portReady = await PortManager.waitForService(port, checkInterval);
  
  if (portReady) {
    this.emit('process:progress', { name, status: 'ready', message: `Service ready on port ${port}`, port });
    break;
  }
  
  // Report progress every 2.5 seconds
  if (attempts % 10 === 0) {
    const elapsed = attempts * checkInterval;
    this.emit('process:progress', {
      name,
      status: 'waiting',
      message: `Waiting for service to start... (${Math.round(elapsed/1000)}s)`,
      port
    });
  }
}
```

##### 4. Neural Network Animation System

###### Animation Architecture
```javascript
class StartupNeuralNetwork {
  constructor(canvas) {
    this.neurons = [];     // Array of neuron objects
    this.connections = []; // Array of connection objects
    this.pulses = [];     // Active signal pulses
  }
  
  initializeNetwork() {
    // Creates 4-layer network matching consensus engine stages
    // Generator → Refiner → Validator → Curator
    const layers = [
      { count: 4, y: 0.2 },   // Input layer
      { count: 6, y: 0.4 },   // Hidden layer 1
      { count: 6, y: 0.6 },   // Hidden layer 2
      { count: 3, y: 0.8 }    // Output layer
    ];
  }
  
  updateProgress(percent) {
    // Progressively illuminates neurons based on startup progress
    const activeNeurons = Math.floor((this.neurons.length * percent) / 100);
    this.neurons.forEach((neuron, i) => {
      neuron.brightness = i < activeNeurons ? 
        0.3 + (0.7 * (percent / 100)) : 0.1;
    });
  }
}
```

##### 5. Dynamic Port Allocation

###### Port Management - Production Zero-Fallback Pattern
```typescript
// StartupOrchestrator NEVER uses fallback ports
verify: async () => {
  // Get dynamic port from ProcessManager - NO FALLBACK
  const info = this.initFunctions.processManager.getProcessStatus('websocket-backend');
  if (!info?.port) {
    throw new Error('Backend service has no allocated port');
  }
  // Only proceed with properly allocated port
  return this.checkHealth(`http://localhost:${info.port}/health`);
}

// ProcessManager allocates from pre-scanned pools
const port = await PortManager.allocatePortForService(name);
if (!port) {
  throw new Error(`No ports available for service: ${name}`);
}
// NO FALLBACK PORTS - Service fails if pool exhausted
```

###### Port Detection Strategy (Fail-Fast Design)
```typescript
// New isPortListening method - connects to check if port is ready
private static async isPortListening(port: number): Promise<boolean> {
  return new Promise((resolve) => {
    const client = new net.Socket();
    const timeout = setTimeout(() => {
      client.destroy();
      resolve(false);
    }, 100);
    
    client.once('connect', () => {
      clearTimeout(timeout);
      client.destroy();
      resolve(true);  // Port is listening!
    });
    
    client.once('error', () => {
      clearTimeout(timeout);
      resolve(false);
    });
    
    client.connect(port, 'localhost');
  });
}
```

#### Service Initialization Flow

1. **Database Initialization** (15% weight)
   - Clean up orphaned processes via PidTracker
   - Initialize SQLite connection
   - No network calls, always fast

2. **Process Manager Setup** (10% weight)
   - Register service configurations
   - Initialize port management
   - Set up event emitters

3. **IPC Handler Registration** (10% weight)
   - Memory Service handlers
   - Git integration handlers
   - File system handlers
   - Dialog handlers
   - CLI tool handlers
   - WebSocket backend port handler

4. **Memory Service Launch** (20% weight - Optional)
   - Dynamic port allocation (3457-3560 range)
   - Express server initialization
   - WebSocket server setup
   - Can fail without blocking startup

5. **Backend Server Launch** (25% weight - Required)
   - Dynamic port allocation (8765-8865 range)
   - Database connection
   - Consensus engine initialization
   - Model syncing from OpenRouter
   - AI helper ecosystem setup
   - Python subprocess for ML models

6. **CLI Tools Detection** (15% weight - Optional)
   - Detect installed AI CLI tools
   - Version checking
   - Path resolution

#### Progress Reporting Chain

```
Backend Server Process → Console Output
                      ↓
         ProcessManager (monitors output)
                      ↓
         Emits 'process:progress' events
                      ↓
         StartupOrchestrator (listening)
                      ↓
         Updates splash window via IPC
                      ↓
         Visual feedback to user
```

#### Error Handling Philosophy

- **No Timeout Failures**: Services never fail due to arbitrary time limits
- **Required vs Optional**: Only required services can block startup
- **Graceful Degradation**: App launches even if optional services fail
- **User Feedback**: Clear error messages displayed on splash screen
- **Auto-Recovery**: ProcessManager handles service restarts automatically

#### Performance Characteristics

- **Typical Startup**: 3-5 seconds with all services
- **Slow Network**: May take 10-30 seconds for model syncing
- **No Upper Limit**: System waits indefinitely if needed
- **User Can Cancel**: Close splash window to abort startup

#### Files Structure
```
electron-poc/
├── startup.html                    # Splash screen HTML (fixed positioning)
├── startup-neural.js               # Neural network animation engine
├── startup-preload.js              # IPC bridge for progress updates
├── src/
│   └── startup/
│       └── StartupOrchestrator.ts # Main orchestration logic
└── src/utils/
    ├── ProcessManager.ts           # Event-driven process management
    └── PortManager.ts              # Dynamic port allocation
```

#### Key Improvements in v2.0.0

1. **Removed ALL Timeouts**: No `waitForService` with timeout parameters
2. **Event-Driven Updates**: Real-time progress from ProcessManager
3. **Fixed Visual Positioning**: Text always visible on screen
4. **Dynamic Port Discovery**: No hardcoded ports anywhere
5. **Continuous Monitoring**: ProcessManager reports until ready
6. **Better Error Recovery**: Services can restart without user intervention
7. **Professional UX**: Smooth, informative, never stuck

#### Terminal Panel Visual Fix (Resolved Issues)

##### The 9-Row Terminal Problem (SOLVED)
- **Issue**: TTYD terminals would get stuck at 9 rows when window minimized/maximized
- **Root Cause**: WebView reload attempts failed with ERR_FAILED, terminal size set server-side
- **Solution**: Removed all resize handlers and webview reloading, using fixed flexbox layout

##### CSS Architecture for Terminal Panels
```css
/* Fixed width panel - no manual dragging */
.isolated-terminal-panel {
  flex: 0 0 450px;  /* No grow, no shrink, fixed 450px */
  transition: flex 0.2s ease;
}

/* Auto-expand when center collapsed */
.isolated-terminal-panel.expanded {
  flex: 1 1 auto;  /* Take all available space */
}
```

This architecture ensures the startup experience is smooth, informative, and reliable - taking exactly as long as needed with continuous visual feedback.

### Enhanced ProcessManager System (2025 Production Architecture)
**Location**: `src/utils/ProcessManager.ts`

**Core Philosophy**: 
- **Parallel Everything**: All services start simultaneously (2025 best practice)
- **Zero Blocking**: No service waits for another to start
- **Dynamic Ports Only**: Never retry same port, always find next available
- **Critical Path Protection**: Consensus engine never blocked by other services

**Key Improvements**:

#### 1. Process Registration & Configuration
```typescript
interface ProcessConfig {
  name: string;                    // Unique process identifier
  scriptPath: string;              // Path to executable/script
  args?: string[];                 // Command line arguments
  env?: NodeJS.ProcessEnv;         // Environment variables
  port?: number;                   // Preferred port (not required)
  alternativePorts?: number[];     // Large range for dynamic allocation
  autoRestart?: boolean;           // Enable crash recovery
  maxRestarts?: number;            // Retry limit (default: 5)
  restartDelay?: number;           // Ms between restarts (default: 3000)
  healthCheckUrl?: string;         // HTTP endpoint for health monitoring
  healthCheckInterval?: number;    // Ms between health checks
  stdio?: StdioOptions;            // Critical for binary processes
  detached?: boolean;              // Run independently of parent
  priority?: 'critical' | 'high' | 'normal'; // Startup priority
}
```

#### 2. Dynamic Port Management System
```typescript
// PortManager ensures no port conflicts (src/utils/PortManager.ts)
class PortManager {
  // Finds next available port automatically
  static async allocatePort(config: PortConfig): Promise<number> {
    // Try preferred port first
    if (await isPortAvailable(preferredPort)) return preferredPort;
    
    // Scan up to 100 ports ahead
    for (let port = preferredPort + 1; port < preferredPort + 100; port++) {
      if (await isPortAvailable(port)) return port;
    }
  }
}

// Frontend discovers ports dynamically via IPC
const backendPort = await window.backendAPI.getBackendPort();
const wsUrl = `ws://127.0.0.1:${backendPort}/ws`;
```

#### 3. Stdio Configuration (Critical for AI Helpers)
```typescript
// For binary processes with Python subprocesses (AI Helpers)
stdio: 'inherit'  // REQUIRED - Allows subprocess communication

// For Node.js processes with IPC
stdio: 'pipe'     // Default for fork()

// NEVER use for processes with subprocesses:
stdio: ['ignore', 'pipe', 'pipe']  // Breaks AI Helper communication
```

#### 3. Parallel Startup Architecture
```typescript
async startAllProcesses(): Promise<void> {
  const startupPromises: Promise<ProcessResult>[] = [];
  
  // Start ALL processes simultaneously
  for (const [name, config] of this.processConfigs) {
    startupPromises.push(this.startProcessAsync(name, config));
  }
  
  // Wait for all to complete/fail
  const results = await Promise.allSettled(startupPromises);
  
  // Log results but don't block on failures
  results.forEach((result, index) => {
    if (result.status === 'rejected') {
      console.error(`Process ${index} failed:`, result.reason);
    }
  });
}
```

#### 4. Dynamic Port Allocation (No Retries)
```typescript
async allocatePort(config: PortConfig): Promise<number> {
  let currentPort = config.preferredPort || config.alternativePorts[0];
  const maxPort = currentPort + 100; // Scan up to 100 ports ahead
  
  // Never retry same port - always find next available
  while (currentPort < maxPort) {
    if (await this.isPortAvailable(currentPort)) {
      return currentPort;
    }
    currentPort++;
  }
  
  throw new Error(`No available ports in range ${config.preferredPort}-${maxPort}`);
}

// Port ranges for each service
const PORT_RANGES = {
  'memory-service': { 
    preferred: 3457, 
    range: [3457, 3560]  // 100+ alternatives
  },
  'websocket-backend': { 
    preferred: 8765, 
    range: [8765, 8865]  // 100+ alternatives
  },
  'analytics-service': {
    preferred: 4567,
    range: [4567, 4667]  // 100+ alternatives
  }
};
```

#### 5. Process Type Detection & Handling
```typescript
detectProcessType(scriptPath: string): ProcessType {
  const ext = path.extname(scriptPath);
  
  if (ext === '.ts') return 'typescript';
  if (ext === '.js') return 'javascript';
  if (ext === '.rs' || !ext) return 'binary';  // Rust binaries
  
  return 'unknown';
}

async spawnProcess(config: ProcessConfig): Promise<ChildProcess> {
  const type = this.detectProcessType(config.scriptPath);
  
  switch (type) {
    case 'binary':
      // Critical: Use 'inherit' for processes with subprocesses
      return spawn(config.scriptPath, config.args, {
        stdio: 'inherit',  // Allows AI Helper communication
        env: { ...process.env, ...config.env },
        detached: false
      });
      
    case 'typescript':
      // Use fork with ts-node for IPC support
      return fork(config.scriptPath, config.args, {
        execArgv: ['-r', 'ts-node/register'],
        env: config.env,
        silent: false
      });
      
    default:
      return fork(config.scriptPath, config.args, {
        env: config.env,
        silent: false
      });
  }
}
```

#### 6. IPC Ready Message Handling (Critical Fix)
```typescript
// FIXED: Race condition where message handler intercepted ready signal
// Solution: Create ready promise BEFORE setting up message handlers

let readyResolver: ((value: boolean) => void) | null = null;
let readyTimeout: NodeJS.Timeout | null = null;

// Create ready promise first
const readyPromise = (config.scriptPath.endsWith('.ts') || config.scriptPath.endsWith('.js')) 
  ? new Promise<boolean>((resolve) => {
      readyResolver = resolve;
      readyTimeout = setTimeout(() => {
        logger.info(`Timeout waiting for ${name} ready signal`);
        resolve(false);
      }, 15000);
    })
  : null;

// THEN set up message handlers
childProcess.on('message', (msg: any) => {
  // Handle ready message first if waiting for it
  if (readyResolver && msg.type === 'ready') {
    if (readyTimeout) clearTimeout(readyTimeout);
    readyResolver(true);
    readyResolver = null; // Prevent double resolution
  }
  // Then handle normally
  this.handleProcessMessage(name, msg);
});

// Wait for ready signal
if (readyPromise) {
  isReady = await readyPromise;
}
```

#### 7. Enhanced Status Reporting
```typescript
interface ProcessStatus {
  name: string;
  state: 'stopped' | 'starting' | 'running' | 'crashed' | 'unhealthy';
  pid?: number;
  port?: number;
  uptime?: number;
  restartCount: number;
  lastError?: string;
  memoryUsage?: number;
  cpuUsage?: number;
}

// Comprehensive status methods
getFullStatus(): {
  processes: ProcessStatus[];
  allocatedPorts: Map<string, number>;
  summary: {
    total: number;
    running: number;
    crashed: number;
    unhealthy: number;
  };
}

debugProcess(name: string): Promise<{
  logs: string[];
  environment: NodeJS.ProcessEnv;
  connections: number;
  threads: number;
}>

logStatus(): void {
  console.log('╭─────────────────────────────────────────╮');
  console.log('│         PROCESS MANAGER STATUS         │');
  console.log('├─────────────────────────────────────────┤');
  for (const [name, info] of this.processes) {
    console.log(`│ ${name.padEnd(20)} │ ${info.state.padEnd(10)} │`);
  }
  console.log('╰─────────────────────────────────────────╯');
}
```

#### 7. Startup Optimization Techniques
```typescript
// 1. Deferred Initialization (Backend)
// Backend binds to port immediately, initializes consensus later
async startBackend(): Promise<void> {
  // Start server immediately
  const server = await this.bindToPort(port);
  
  // Defer heavy initialization
  setTimeout(() => {
    this.initializeConsensusEngine();
  }, 1000);
}

// 2. Parallel Service Discovery
// Don't wait for services to be ready before starting others
async startServices(): Promise<void> {
  const services = ['memory', 'analytics', 'consensus'];
  
  // Start all at once
  await Promise.all(services.map(s => this.startService(s)));
  
  // Check readiness in background
  this.monitorReadiness(services);
}

// 3. Non-Blocking Health Checks
// Health checks run in background, don't block startup
scheduleHealthChecks(): void {
  setInterval(async () => {
    for (const [name, config] of this.processConfigs) {
      // Non-blocking check
      this.checkHealthAsync(name, config).catch(err => {
        console.warn(`Health check failed for ${name}:`, err);
      });
    }
  }, 30000);
}
```

#### 8. AI Helper Integration Requirements
```typescript
// Critical configuration for AI Helper subprocess communication
const CONSENSUS_CONFIG = {
  name: 'websocket-backend',
  scriptPath: '/path/to/hive-backend-server-enhanced',
  stdio: 'inherit',  // MUST be 'inherit' for Python subprocess
  env: {
    RUST_LOG: 'info',
    PORT: '8765',
    PYTHONUNBUFFERED: '1',  // Critical for Python output
    TRANSFORMERS_OFFLINE: '0',
    HF_HOME: '~/.hive/models'
  },
  priority: 'critical',
  initTimeout: 30000  // 30s timeout for AI Helper init
};
```

#### 9. Process Crash Recovery Protocol
```typescript
async handleProcessCrash(name: string): Promise<void> {
  const info = this.processes.get(name);
  if (!info) return;
  
  // Release port immediately (no blocking)
  if (info.port) {
    PortManager.releasePort(name);
    info.port = undefined;
  }
  
  // Check restart eligibility
  if (info.restartCount < info.maxRestarts) {
    info.restartCount++;
    
    // Find new port (never reuse crashed port)
    const newPort = await this.allocateNewPort(name);
    
    // Restart with new port
    await this.startProcess(name, { ...info.config, port: newPort });
  }
}
```

#### 10. Event System (Enhanced)
```typescript
// New events for better monitoring
processManager.on('port:allocated', (name, port) => {})
processManager.on('port:released', (name, port) => {})
processManager.on('startup:parallel', (services) => {})
processManager.on('consensus:initializing', () => {})
processManager.on('consensus:ready', () => {})
processManager.on('ai-helpers:timeout', () => {})
```

### Advanced PortManager System
**Location**: `src/utils/PortManager.ts`

**2025 Best Practices**:
- **No Retries**: If port in use, immediately try next
- **Large Ranges**: 100+ alternative ports per service
- **Smart Scanning**: Efficient port availability checking
- **No Process Killing**: Never kill existing processes
- **Parallel Allocation**: Allocate ports for all services at once

### Parallel Startup Timeline
```
T+0ms    : App Ready Event
T+1ms    : ProcessManager initialized
T+2ms    : Start all processes in parallel
           ├── Memory Service → Port scan 3457-3560
           ├── Backend Server → Port scan 8765-8865  
           └── Analytics (future) → Port scan 4567-4667
T+100ms  : Port allocations complete
T+150ms  : Processes spawned with allocated ports
T+500ms  : Backend bound to port, serving health endpoint
T+1000ms : Memory Service Express server ready
T+1500ms : Backend starts consensus initialization (deferred)
T+5000ms : Full system operational
```

### Critical Implementation Details

#### Binary Process Communication Fix
The most critical fix was changing stdio configuration for binary processes:
```typescript
// BROKEN - Prevents AI Helper Python subprocess communication
stdio: ['ignore', 'pipe', 'pipe']

// FIXED - Allows full subprocess communication
stdio: 'inherit'
```
This single change resolved the "AI Helpers required for mode detection" error by allowing the Rust backend's Python subprocesses to communicate properly.

#### Port Allocation Strategy
```typescript
// Old (Broken) - Retry same port with delays
async allocatePort(port: number): Promise<number> {
  for (let i = 0; i < 50; i++) {
    if (await isAvailable(port)) return port;
    await sleep(1000); // Wait and retry
  }
  throw new Error('Port unavailable after 50 attempts');
}

// New (Fixed) - Instant next port finding
async allocatePort(preferred: number): Promise<number> {
  let port = preferred;
  while (port < preferred + 100) {
    if (await isAvailable(port)) return port;
    port++; // Immediately try next
  }
  throw new Error('No ports available in range');
}
```

#### Consensus Engine Initialization
```rust
// Backend Server - Deferred initialization pattern
async fn main() {
  // 1. Bind to port immediately (fast)
  let server = bind_server(port).await;
  
  // 2. Start serving health endpoint
  tokio::spawn(health_endpoint());
  
  // 3. Defer consensus initialization
  tokio::spawn(async move {
    sleep(Duration::from_secs(1)).await;
    
    // Initialize with 30s timeout for AI Helpers
    match timeout(Duration::from_secs(30), 
                  ConsensusEngine::new()).await {
      Ok(Ok(engine)) => {
        *CONSENSUS_ENGINE.write().await = Some(engine);
        info!("Consensus ready");
      }
      Err(_) => warn!("AI Helper init timeout")
    }
  });
  
  // 4. Server starts immediately
  server.serve().await;
}
```

### Integration with Main Architecture

#### 1. Memory Service Integration
```typescript
processManager.registerProcess({
  name: 'memory-service',
  scriptPath: 'src/memory-service/index.ts',
  port: 3457,
  alternativePorts: [3458, 3459, 3460],
  autoRestart: true,
  healthCheckUrl: 'http://localhost:{port}/health',
  healthCheckInterval: 30000
});
```

#### 2. WebSocket Backend Integration
```typescript
processManager.registerProcess({
  name: 'websocket-backend',
  scriptPath: '/path/to/websocket-server',
  port: 8765,
  alternativePorts: [8766, 8767, 8768],
  autoRestart: true,
  healthCheckUrl: 'http://localhost:{port}/health',
  healthCheckInterval: 30000
});
```

#### 3. AI Helper Python Subprocess Architecture
```rust
// In src/ai_helpers/python_models.rs
pub struct PythonModelService {
  process: Arc<Mutex<Option<Child>>>,
  stdin: Arc<Mutex<Option<ChildStdin>>>,
  response_handlers: Arc<RwLock<HashMap<String, Sender<Response>>>>
}

impl PythonModelService {
  async fn start(&self) -> Result<()> {
    let mut cmd = Command::new("python3");
    cmd.arg("model_service.py")
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .kill_on_drop(true)  // Critical: Clean subprocess termination
       .env("PYTHONUNBUFFERED", "1")  // Critical: Real-time output
       .env("TRANSFORMERS_OFFLINE", "0")
       .env("HF_HOME", model_cache_dir);
    
    let mut child = cmd.spawn()?;
    
    // Capture stderr for debugging
    let stderr = child.stderr.take().unwrap();
    tokio::spawn(async move {
      let reader = BufReader::new(stderr);
      let mut lines = reader.lines();
      while let Ok(Some(line)) = lines.next_line().await {
        tracing::warn!("Python stderr: {}", line);
      }
    });
    
    // Handle stdout responses
    let stdout = child.stdout.take().unwrap();
    tokio::spawn(async move {
      let reader = BufReader::new(stdout);
      let mut lines = reader.lines();
      while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(response) = serde_json::from_str(&line) {
          // Route response to waiting handler
          self.route_response(response).await;
        }
      }
    });
  }
}
```

**Critical Requirements**:
- Parent process MUST use `stdio: 'inherit'` in ProcessManager
- Python subprocess MUST set `PYTHONUNBUFFERED=1`
- Use `kill_on_drop(true)` to prevent orphan processes
- Capture stderr separately for debugging
- Add timeout wrappers for initialization

#### 4. Benefits of Unified Process Management
- **Single Point of Control**: All processes managed through one system
- **Consistent Lifecycle**: Same startup/shutdown/restart logic for all
- **Unified Monitoring**: Single dashboard for all process health
- **Simplified Debugging**: Centralized logging and error handling
- **Production Ready**: Built-in recovery and health monitoring
- **User Experience**: Appears as single cohesive application
- **2025 Performance**: Parallel startup, dynamic ports, zero blocking

### Process Cleanup & Termination Architecture

#### Overview
A robust multi-layered cleanup system ensures no orphaned processes or port conflicts across development and production environments.

#### Components

**1. PidTracker System** (`src/utils/PidTracker.ts`)
- Tracks all spawned process PIDs to disk (`/tmp/hive-electron-poc.pids`)
- Cleans up orphaned processes from previous runs on startup
- Removes PID tracking when processes terminate normally
- Critical for development where apps may be force-killed

**2. Unified Cleanup Function** (`src/index.ts`)
```typescript
async function performCleanup(reason: string) {
  if (isCleaningUp) return;  // Prevent duplicate cleanup
  isCleaningUp = true;
  
  // Clean up all terminals first
  cleanupTerminals();
  
  // Stop memory service if running
  await processManager.stopProcess('memory-service');
  
  // Clean up all other processes
  await processManager.cleanup();
}
```

**3. Signal Handlers**
- `before-quit`: Normal app closure
- `SIGINT`: Ctrl+C termination
- `SIGTERM`: System shutdown
- `uncaughtException`: Crash recovery
- `unhandledRejection`: Promise error recovery

**4. Development Cleanup Script** (`scripts/cleanup-dev.sh`)
```bash
# Kills lingering processes and frees ports
./scripts/cleanup-dev.sh

# Automatically kills:
# - ttyd processes (terminal servers)
# - memory-service processes
# - backend-server processes
# - Processes using ports 7100-7110, 3457
```

#### Cleanup Flow

**Normal Shutdown**:
```
User Quits App
    ↓
app.on('before-quit')
    ↓
performCleanup('before-quit')
    ↓
├── cleanupTerminals()
├── stopProcess('memory-service')
├── processManager.cleanup()
└── PidTracker removes PIDs
    ↓
app.exit(0)
```

**Force Termination Recovery**:
```
App killed with `kill -9`
    ↓
PIDs remain in /tmp/hive-electron-poc.pids
    ↓
Next app start
    ↓
app.on('ready')
    ↓
PidTracker.cleanupOrphans()
    ↓
├── Read PID file
├── Check each PID if running
├── SIGTERM to running processes
├── SIGKILL if still running after 1s
└── Clear PID file
```

#### Port Management During Cleanup

**Port Release Flow**:
```typescript
// When process stops/crashes:
1. Process termination detected
2. PidTracker.removePid(pid)
3. PortManager.releasePort(name)
4. Port available for reallocation
```

**TTYD Terminal Cleanup**:
- Each terminal tracked with unique ID
- PID tracked via PidTracker
- Port released via PortManager
- Process killed on tab close or app quit

#### Common Issues & Solutions

**Issue**: Ports still in use after crash
**Solution**: Run `./scripts/cleanup-dev.sh` or wait for auto-cleanup on next start

**Issue**: Duplicate `before-quit` handlers
**Solution**: Single unified cleanup function prevents conflicts

**Issue**: Orphaned Python processes
**Solution**: `kill_on_drop(true)` in Rust + PidTracker ensures cleanup

---

## Python Runtime & AI Helpers Architecture

### Overview
The Electron app bundles a complete Python runtime with all ML dependencies to ensure AI Helpers work without requiring users to install Python or any packages. This is critical for production deployment.

### Architecture Philosophy
- **Self-Contained**: Everything needed ships with the app
- **No System Dependencies**: Users don't need Python installed
- **Platform-Agnostic**: Same approach works across macOS/Windows/Linux
- **Production-Ready**: Works on clean systems out of the box

### Directory Structure
```
electron-poc/
├── resources/
│   └── python-runtime/
│       ├── venv/                    # Python virtual environment
│       │   ├── bin/
│       │   │   └── python3          # Python executable
│       │   └── lib/
│       │       └── python3.13/
│       │           └── site-packages/
│       │               ├── numpy/
│       │               ├── torch/
│       │               ├── transformers/
│       │               └── sentence_transformers/
│       └── models/
│           └── model_service.py     # AI Helper service script
```

### Implementation Details

#### 1. Environment Variable Configuration
The Electron main process passes Python paths to the Rust backend via environment variables:
```typescript
// In electron-poc/src/index.ts
const bundledPythonPath = '/path/to/venv/bin/python3';
const bundledModelScript = path.join(app.getAppPath(), 'resources', 'python-runtime', 'models', 'model_service.py');

processManager.registerProcess({
  name: 'websocket-backend',
  env: {
    HIVE_BUNDLED_PYTHON: bundledPythonPath,
    HIVE_BUNDLED_MODEL_SCRIPT: bundledModelScript
  }
});
```

#### 2. Rust Backend Python Detection
The Rust backend checks for bundled Python before falling back to system Python:
```rust
// In src/ai_helpers/python_models.rs
let python_path = if let Ok(bundled_python) = std::env::var("HIVE_BUNDLED_PYTHON") {
    // Production: Use bundled Python from Electron
    tracing::info!("Using bundled Python from Electron: {}", bundled_python);
    bundled_python
} else if let Ok(current_dir) = std::env::current_dir() {
    // Development: Check for venv
    let venv_python = current_dir.join("venv").join("bin").join("python3");
    if venv_python.exists() {
        venv_python.to_string_lossy().to_string()
    } else {
        "python3".to_string()
    }
} else {
    "python3".to_string()
};
```

#### 3. Python Subprocess Management
```rust
// Critical configuration for subprocess communication
let mut cmd = Command::new(&self.config.python_path);
cmd.arg(&self.config.service_script)
   .stdin(Stdio::piped())
   .stdout(Stdio::piped())
   .stderr(Stdio::piped())
   .kill_on_drop(true)              // Clean termination
   .env("PYTHONUNBUFFERED", "1")    // Real-time output
   .env("TRANSFORMERS_OFFLINE", "0")
   .env("HF_HOME", model_cache_dir);
```

### Package Dependencies
The bundled Python includes these critical packages:
- **numpy 2.3.1**: Numerical computing foundation
- **torch 2.7.1**: PyTorch for neural networks
- **transformers 4.53.2**: Hugging Face transformers
- **sentence-transformers 5.0.0**: Sentence embeddings

### Production Deployment Strategy

#### Current Implementation (Development)
- Uses Python venv with symlinks to system Python
- Requires Python 3.13 installed on the system
- Works for development and testing

#### Target Implementation (Production)
1. **Portable Python Distribution**
   - Use py2app (macOS), py2exe (Windows), or PyInstaller
   - Creates standalone Python without system dependencies
   - No symlinks, fully self-contained

2. **Alternative: Binary Compilation**
   - Use Nuitka or PyOxidizer to compile Python to native binary
   - Single executable file, no Python runtime needed
   - Best performance and smallest size

3. **Ultimate Goal: Pure Rust**
   - Rewrite Python AI Helpers in Rust
   - Use candle, tch, or ort for ML operations
   - Single binary, optimal performance

### Process Communication Flow
```
Electron Main Process
    ↓ (Environment Variables)
ProcessManager
    ↓ (spawn with stdio: 'inherit')
Rust Backend Process
    ↓ (HIVE_BUNDLED_PYTHON env var)
Python Subprocess (AI Helpers)
    ↓ (JSON over stdin/stdout)
Model Service (transformers, torch)
```

### Critical Requirements
1. **Stdio Inheritance**: Binary processes MUST use `stdio: 'inherit'` for Python subprocess communication
2. **Environment Variables**: Pass `PYTHONUNBUFFERED=1` for real-time output
3. **Path Resolution**: Use absolute paths for Python executable and scripts
4. **Error Handling**: 30-second timeout on AI Helper initialization
5. **Process Cleanup**: Use `kill_on_drop(true)` to prevent orphan processes

### Known Issues & Solutions

#### Issue: Python Not Found
**Symptom**: "Failed to spawn Python model service"
**Solution**: Ensure HIVE_BUNDLED_PYTHON points to valid Python executable

#### Issue: Module Import Errors
**Symptom**: "ModuleNotFoundError: No module named 'numpy'"
**Solution**: Verify all packages are installed in the bundled venv

#### Issue: Subprocess Communication Failure
**Symptom**: "AI Helpers required for mode detection"
**Solution**: Ensure ProcessManager uses `stdio: 'inherit'` for binary processes

### Testing the Bundled Python
```bash
# Verify Python executable
$HIVE_BUNDLED_PYTHON --version

# Test package imports
$HIVE_BUNDLED_PYTHON -c "import numpy, torch, transformers; print('All packages loaded')"

# Run model service directly
$HIVE_BUNDLED_PYTHON $HIVE_BUNDLED_MODEL_SCRIPT --model-cache-dir ~/.hive/models
```

---

## Data Architecture

### Database Schema (SQLite)
**Location**: `~/.hive/hive-ai.db`

#### Core Tables

##### 1. Users Table
```sql
users (
  id TEXT PRIMARY KEY,
  email TEXT UNIQUE,
  license_key TEXT,
  tier TEXT DEFAULT 'FREE',
  created_at TEXT,
  updated_at TEXT
)
```

##### 2. Conversations Table
```sql
conversations (
  id TEXT PRIMARY KEY,
  user_id TEXT,
  title TEXT,
  model_used TEXT,
  timestamp TEXT,
  metadata TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id)
)
```

##### 3. Messages Table
```sql
messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  content TEXT NOT NULL,
  role TEXT NOT NULL,
  stage TEXT,
  model_used TEXT,
  timestamp TEXT,
  tokens_used INTEGER,
  cost REAL,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id)
)
```

##### 4. Conversation Usage Table
```sql
conversation_usage (
  conversation_id TEXT NOT NULL UNIQUE,
  user_id TEXT,
  timestamp TEXT,
  message_count INTEGER DEFAULT 0,
  total_tokens INTEGER DEFAULT 0,
  total_cost REAL DEFAULT 0.0,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id)
)
```

##### 5. Configuration Table
```sql
configurations (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  encrypted BOOLEAN DEFAULT 0,
  user_id TEXT,
  created_at TEXT,
  updated_at TEXT
)
```

### Data Flow
```
User Input → 
  Renderer Process → 
    IPC to Main → 
      Database Write → 
        D1 Sync (if online) → 
          Memory Service Update → 
            Dashboard Refresh
```

---

## Communication Layers

### 1. IPC (Inter-Process Communication)
**Main ↔ Renderer**:
```typescript
// Renderer → Main (via preload)
electronAPI.runConsensus(prompt, options)
electronAPI.saveConversation(data)
electronAPI.getUsageCount()

// Main → Renderer
webContents.send('consensus-update', data)
webContents.send('file-changed', filepath)
```

**Main ↔ Memory Service**:
```typescript
// Memory Service → Main
process.send({ type: 'db-query', sql, params })
process.send({ type: 'ready', port })

// Main → Memory Service  
childProcess.send({ type: 'db-result', data })
```

### 2. HTTP/REST APIs

#### Memory Service API (Port 3457)
```
GET  /health                 - Service health check
POST /api/v1/memory/register - Register new tool
POST /api/v1/memory/query    - Query memories
POST /api/v1/memory/contribute - Contribute learning
GET  /api/v1/memory/stats    - Get statistics
GET  /api/v1/memory/tools    - List connected tools
GET  /api/v1/memory/activity - Get activity stream
```

#### Cloudflare D1 Sync API
```
POST /api/validate-license    - License validation
POST /api/sync-conversation   - Sync conversation data
GET  /api/analytics           - Fetch usage analytics
```

### 3. WebSocket Connections
**Memory Dashboard ↔ Memory Service**:
- Real-time statistics updates
- Activity stream broadcasting
- Connected tools monitoring

---

## User Interface Architecture

### File Explorer Component
**Implementation**: `src/vscode-explorer-exact.ts`

#### Architecture
- **Tree Structure**: Recursive TreeNode interface with parent/child relationships
- **Virtual Rendering**: Only renders visible nodes for performance
- **Lazy Loading**: Directory contents loaded on demand
- **State Management**: Tracks expanded/collapsed nodes
- **Event System**: Drag & drop, click, double-click handlers

#### Features
- **File Operations**:
  - Create new files/folders
  - Rename via F2 key
  - Delete via Delete key
  - Move via drag & drop
- **Visual Features**:
  - VS Code exact styling
  - File type icons with colors
  - Indentation levels
  - Chevron indicators for folders
  - Selection highlighting
- **Git Integration**:
  - Status badges (M, A, D, U, R) with colors
  - Color-coded filenames based on status
  - Event-driven refresh (no polling)
  - In-place decoration updates
  - Integration with GitDecorationProvider

#### Performance Optimizations
- **No Polling**: Event-driven updates only (no intervals)
- **In-Place Updates**: Git decorations update without tree rebuild via `updateGitDecorationsInPlace()`
- **Scroll Preservation**: Maintains scroll position during renders using `requestAnimationFrame()`
- **Debounced Operations**: 500ms delay for Git refresh on typing
- **Lazy Loading**: Directory contents loaded on-demand
- **Cached Lookups**: Git status cached in Map structure
- **DOM Fragment Rendering**: Batched DOM updates for performance
- **Efficient Re-renders**: Only updates changed content areas, preserves tree expansion state

### File Menu System
**Implementation**: `src/index.ts` (main process)

#### Menu Structure
```typescript
File Menu
├── New File (Ctrl/Cmd+N)
├── Open File (Ctrl/Cmd+O)
├── Open Folder (Ctrl/Cmd+K Ctrl/Cmd+O)
├── ─────────────
├── Save (Ctrl/Cmd+S)
├── Save As (Ctrl/Cmd+Shift+S)
├── ─────────────
├── Auto Save [Toggle] ✓
├── ─────────────
├── Close Tab (Ctrl/Cmd+W)
├── Close All Tabs
├── Close Folder
└── Exit (Ctrl/Cmd+Q)
```

#### Auto-Save Feature
- **Toggle Option**: Checkbox in File menu (checked state persists)
- **Default Delay**: 1000ms after last change
- **Implementation**: Debounced save on content changes via `autoSaveTimeout`
- **Visual Feedback**: Dirty indicator (orange dot #FFA500) on unsaved tabs
- **Persistence**: Saves editor state before closing
- **IPC Event**: `menu-toggle-auto-save` toggles the feature

#### IPC Communication
- Main process sends menu events via IPC
- Renderer listens via `electronAPI.onMenu*` handlers
- Bidirectional communication for dialogs
- Type-safe interfaces in `window.d.ts`

### Editor Tabs Component
**Implementation**: `src/editor-tabs.ts`

#### Architecture
- **Tab Management**: Array of tab objects with unique IDs
- **Monaco Integration**: One Monaco editor instance per tab
- **Model Management**: Separate text models for each file
- **State Tracking**: Dirty state, active tab, file paths

#### Features
- **Multi-file Editing**:
  - Unlimited open tabs
  - Tab switching with preserved state
  - Unsaved changes indicator (orange dot - #FFA500)
  - Close button per tab
- **Auto-Save System**:
  - Configurable delay (default 1000ms)
  - Debounced save on content changes
  - Toggle via File menu (checkbox state)
  - Visual feedback for save operations
  - Persists auto-save preference
- **Git Integration**:
  - Debounced refresh on content changes (500ms)
  - Immediate refresh after save operations
  - Updates Explorer decorations in-place
  - Triggers SCM view refresh on save
  - Orange dot indicator (#FFA500) for unsaved changes
  - Properly updates both window.scmView and window.fileExplorer
- **File Operations**:
  - Save (Ctrl/Cmd+S)
  - Save As (not yet implemented)
  - Close tab (X button or Ctrl/Cmd+W)
  - Close all tabs

#### Event Handling
- Content change detection via Monaco API
- File save triggers via keyboard shortcuts
- Menu command handlers
- Tab click/close events
- Drag & drop support (planned)

### Component Hierarchy
```
App Root (renderer.ts)
├── Header Bar
│   ├── App Title
│   └── Window Controls
├── Activity Bar (Left Edge)
│   ├── Explorer
│   ├── Source Control
│   ├── Divider
│   ├── Analytics
│   ├── Memory
│   ├── CLI Tools
│   ├── Divider
│   ├── AI Quick Launch Icons (6 tools)
│   │   ├── Claude Code
│   │   ├── Gemini CLI
│   │   ├── Grok
│   │   ├── Qwen Code
│   │   ├── OpenAI Codex
│   │   └── Cline
│   ├── Divider
│   └── Settings (Fixed at bottom)
├── Sidebar Panel (320px width)
│   ├── File Explorer (scrollable)
│   ├── Source Control View (scrollable)
│   │   ├── Header (fixed)
│   │   ├── Commit Input (fixed)
│   │   ├── File Groups (scrollable)
│   │   │   ├── Staged Changes
│   │   │   ├── Changes
│   │   │   └── Untracked
│   │   ├── Git Graph (300px fixed)
│   │   └── Status Bar (fixed)
│   ├── Settings Panel
│   └── CLI Tools Panel
├── Main Content Area
│   ├── Editor Tabs
│   │   ├── Code Editor (Monaco)
│   │   ├── Git Diff View
│   │   └── Memory Dashboard
│   └── Terminal Section (Hidden - replaced by System Log in TTYD)
├── Isolated Terminal Panel (Resizable)
│   ├── System Log Tab
│   └── Terminal Tabs
├── Consensus Panel (Right Side, Resizable)
│   ├── Neural Consciousness
│   ├── Progress Bars
│   └── Chat Interface
├── Status Bar
│   ├── Connection Status
│   ├── Usage Counter
│   └── Model Selection
└── Modals
    ├── Settings Modal
    └── Analytics Modal
```

### State Management
- **No framework** - Vanilla TypeScript with DOM manipulation
- **Event-driven** updates via IPC and WebSockets
- **Local storage** for UI preferences
- **Database** for persistent application state

### Panel System Architecture

#### Complete DOM Structure and Layout System

##### DOM Hierarchy
The application's DOM (Document Object Model) is a tree structure that defines the entire user interface. All panels, buttons, and interactive elements are nodes in this tree, created dynamically by JavaScript when the application starts.

```html
<body>
  <div class="hive-consensus-gui">
    <!-- Title Bar (Fixed Height: 30px) -->
    <div class="title-bar">
      <div class="title-bar-left"></div>
      <div class="title-bar-center">
        <img src="logo" /> Hive Consensus
      </div>
      <div class="title-bar-right"></div>
    </div>
    
    <!-- Main Content Area (Flexbox Container) -->
    <div class="main-content">
      <!-- Left Sidebar (Fixed + Collapsible) -->
      <div class="sidebar" id="left-sidebar">
        <div class="activity-bar-unified">         <!-- 48px wide -->
          <button class="activity-btn">...</button> <!-- Explorer, Git, etc -->
          <div class="ai-cli-icons-section">       <!-- AI Tool Icons -->
          <div class="sidebar-bottom-section">     <!-- Settings at bottom -->
        </div>
        <div class="sidebar-panel">                <!-- 260px collapsible -->
          <div id="explorer-sidebar">...</div>
          <div id="git-sidebar">...</div>
        </div>
      </div>
      
      <!-- Center Area (Flexible Width) -->
      <div class="center-area" id="center-area">
        <div class="editor-area">...</div>
        <div id="analytics-panel">...</div>
        <div class="terminal-section" style="display:none">...</div>
      </div>
      
      <!-- TTYD Terminal Panel (Resizable: 200-1200px) -->
      <div class="isolated-terminal-panel" id="isolated-terminal-panel">
        <button class="panel-collapse-btn">−</button>
        <div class="resize-handle vertical-resize"></div>
        <div class="isolated-terminal-header">...</div>
        <div class="isolated-terminal-content">...</div>
      </div>
      
      <!-- Consensus Panel (Resizable: 300-800px) -->
      <div class="consensus-chat-panel" id="consensus-chat">
        <button class="panel-collapse-btn">−</button>
        <div class="resize-handle vertical-resize"></div>
        <div id="neural-consciousness-container">...</div>
        <div class="progress-section">...</div>
        <div class="chat-area">...</div>
      </div>
    </div>
    
    <!-- Status Bar (Fixed Height: 22px) -->
    <div class="status-bar">...</div>
  </div>
</body>
```

##### Panel Types and Specifications

**1. Fixed Panels**
- **Activity Bar**: 48px width, never resizes
- **Title Bar**: 30px height, spans full width
- **Status Bar**: 22px height, spans full width

**2. Collapsible Panels**
- **Sidebar Panel**: 260px default width, can collapse to 0
  - Toggled by activity buttons
  - Contains Explorer, Git, Settings, CLI Tools views
  - Only one view visible at a time

- **TTYD Terminal Panel**: Can collapse to 40px (v1.7.5)
  - Toggle button shows + when collapsed, − when expanded
  - Maintains user-defined width when expanded
  - Auto-expands with `expand-to-fill` class when center collapses
  
- **Consensus Panel**: Can collapse to 40px
  - Toggle button shows + when collapsed, − when expanded
  - Maintains user-defined width when expanded
  
- **Center Area**: Can collapse to 40px (v1.7.5)
  - Toggle button shows + when collapsed, − when expanded
  - Adjacent panels auto-expand to fill space when collapsed

**3. Fixed-Size Flexbox Panels (v1.8.0 - No Manual Resizing)**
- **TTYD Terminal Panel**: 
  - Fixed width: 450px (flexbox: `flex: 0 0 450px`)
  - Expands automatically when center panel collapses (class: `expanded`)
  - No resize handles to prevent terminal size issues
  - Maintains terminal size stability
  
- **Consensus Panel**:
  - Initial width: 400px
  - Min width: 300px
  - Max width: 800px
  - Resize handle on left edge (still manually resizable)

**4. Flexible Panels**
- **Center Area**: Takes remaining space (when not collapsed)
  - Uses `flex: 1` to fill available space
  - Minimum width: 200px (reduced from 400px for flexibility)
  - Can be manually collapsed to 40px via toggle button
  - Contains editor tabs, analysis reports, and settings views

##### Resize Mechanism Implementation

**Drag Handle System**
Each resizable panel has a 4px wide vertical drag handle positioned on its left edge:

```javascript
// Resize handle setup (lines 3875-3920 in renderer.ts)
const isolatedTerminalResize = document.getElementById('isolated-terminal-resize');
let isResizing = false;
let startX = 0;
let startWidth = 0;

// Mouse down: Start resize
isolatedTerminalResize.addEventListener('mousedown', (e) => {
    isResizing = true;
    startX = e.clientX;
    startWidth = parseInt(window.getComputedStyle(panel).width, 10);
    document.body.style.cursor = 'ew-resize';
});

// Mouse move: Calculate new width
document.addEventListener('mousemove', (e) => {
    if (!isResizing) return;
    
    const deltaX = startX - e.clientX;  // Negative when dragging left
    const newWidth = startWidth + deltaX; // Increases when dragging left
    
    // Apply constraints
    const constrainedWidth = Math.min(Math.max(newWidth, MIN), MAX);
    
    // Check center area minimum
    if (centerAreaWouldBeTooSmall) return;
    
    panel.style.width = constrainedWidth + 'px';
});
```

**Resize Formula Explanation**
- `deltaX = startX - e.clientX`: Calculates mouse movement
  - Negative value when dragging left (expanding panel)
  - Positive value when dragging right (shrinking panel)
- `newWidth = startWidth + deltaX`: Updates panel width
  - Panel grows when dragging left
  - Panel shrinks when dragging right

**Center Area Protection**
The center area has a protected minimum width of 400px to ensure usability:

```javascript
// Calculate remaining space for center area
const windowWidth = window.innerWidth;
const sidebarWidth = document.getElementById('left-sidebar')?.offsetWidth || 0;
const terminalWidth = newWidth; // Proposed new width
const consensusWidth = document.getElementById('consensus-chat')?.offsetWidth || 0;
const remainingWidth = windowWidth - sidebarWidth - terminalWidth - consensusWidth;

// Only apply resize if center area maintains minimum
if (remainingWidth >= 400) {
    panel.style.width = newWidth + 'px';
}
```

##### Layout Strategy and CSS

**Flexbox Layout**
The main container uses flexbox for horizontal layout:

```css
.main-content {
    display: flex;
    flex-direction: row;
    height: calc(100vh - 52px); /* Full height minus title and status bars */
}

/* Fixed width panels */
.sidebar {
    flex-shrink: 0;
    width: 308px; /* 48px activity bar + 260px sidebar panel */
}

/* Flexible center area */
.center-area {
    flex: 1; /* Takes remaining space */
    min-width: 400px; /* Protected minimum */
    overflow: hidden;
}

/* Resizable panels - fixed width, no flex */
.isolated-terminal-panel,
.consensus-chat-panel {
    flex-shrink: 0;
    flex-grow: 0;
    position: relative;
}
```

**Z-Index Layering**
```css
.resize-handle { z-index: 10; }      /* Above content */
.panel-collapse-btn { z-index: 1000; } /* Above everything */
.activity-tooltip { z-index: 10000; }  /* Tooltips on top */
```

##### Panel Collapse/Expand Behavior

**Collapse States**
- **Collapsed Width**: 40px (shows only collapse button)
- **Expanded Width**: Returns to previous width before collapse
- **Animation**: No transition (instant for performance)

**Collapse Implementation**
```javascript
toggleButton.addEventListener('click', () => {
    const isCollapsed = panel.classList.contains('collapsed');
    if (isCollapsed) {
        panel.classList.remove('collapsed');
        panel.style.width = '400px'; // Or saved width
        toggleButton.textContent = '−';
    } else {
        panel.classList.add('collapsed');
        panel.style.width = '40px';
        toggleButton.textContent = '+';
    }
});
```

##### Responsive Behavior

**Window Resize Handling**
The layout automatically adjusts when the window is resized:
- Fixed panels maintain their sizes
- Resizable panels maintain their absolute widths
- Center area flexes to fill remaining space
- If center area would be < 400px, rightmost panels may be auto-collapsed

**Minimum Window Width**
Recommended minimum: 1400px for all panels visible
- Left sidebar: 308px
- Center area: 400px (minimum)
- TTYD panel: 400px (default)
- Consensus panel: 400px (default)
- Total: 1508px

##### Terminal Panel Sizing Solution (v1.8.0)

**Problem Solved**: The "9-row terminal" issue where ttyd terminals would become stuck at 9 rows high when:
- Window was minimized/maximized
- Center panel was collapsed/expanded  
- Panel was manually resized via dragging

**Root Cause**: 
- ttyd sets PTY (pseudo-terminal) size server-side
- Webview reloads triggered by resize events would fail with ERR_FAILED (-2)
- Terminal would see tiny container during transitions and lock to 9 rows

**Solution Architecture**:
1. **Removed all manual resize handlers** - No dragging that triggers reloads
2. **Fixed flexbox layout** - TTYD panel has fixed 450px width
3. **Automatic expansion** - CSS class `expanded` makes panel fill space when center collapses
4. **No webview reloading** - Prevents terminal reset to default size
5. **Pure CSS transitions** - All size changes handled by flexbox, not JavaScript

**Implementation Details**:
```css
/* Fixed width panel */
.isolated-terminal-panel {
  flex: 0 0 450px;  /* No grow, no shrink, fixed 450px */
  transition: flex 0.2s ease;
}

/* Auto-expand when center collapsed */
.isolated-terminal-panel.expanded {
  flex: 1 1 auto;  /* Take all available space */
}
```

##### Panel Interaction Flows

**1. TTYD Terminal Panel (v1.8.0 - No Manual Resize)**
- Fixed at 450px width by default
- Automatically expands when center panel collapses
- No drag handles or manual resizing
- Terminal size remains stable throughout

**2. Switching Sidebar Views**
- User clicks activity button (Explorer, Git, etc.)
- Previous view hidden (display: none)
- New view shown (display: block)
- Sidebar panel expands if collapsed
- Content initializes if first time

**3. Collapsing Panels**
- User clicks collapse button (−)
- Panel width animates to 40px
- Button changes to (+)
- Center area expands to fill space
- Panel content hidden but maintained in DOM

##### Performance Considerations

**DOM Optimization**
- Panels use `display: none` when hidden (removes from render tree)
- Content virtualization for large lists (file explorer, git changes)
- Resize operations use RAF (requestAnimationFrame) for smoothness
- Event delegation for dynamic content

**Memory Management**
- Hidden panels maintain state but release render resources
- Terminal instances destroyed when tabs closed
- Event listeners properly cleaned up on panel destroy

##### Accessibility Features

**Keyboard Navigation**
- Tab order follows visual layout
- Escape key closes modals and panels
- Arrow keys navigate within panels
- F6 cycles between major panels

**ARIA Attributes**
```html
<button aria-label="Explorer" role="button" aria-pressed="true">
<div role="region" aria-label="File Explorer">
<div role="tree" aria-label="Project files">
```

##### Browser Compatibility

**Supported Browsers** (via Electron Chromium)
- Chrome/Chromium 100+
- All modern CSS features available
- No polyfills needed

**CSS Features Used**
- Flexbox for layout
- CSS Grid for complex components
- CSS Variables for theming
- calc() for dynamic sizing
- position: sticky for headers

### UI Components

#### Isolated Terminal Panel (TTYD Implementation v1.8.0)
**Location**: `src/components/TTYDTerminalPanel.ts`
- Completely isolated component with zero impact on rest of app
- Tab management system (System Log + dynamic terminals)
- Console output capture for System Log
- Full ttyd integration with real terminal emulation
- **Fixed-width flexbox layout (no manual resizing)** to prevent terminal size issues
- Collapse/expand functionality with automatic space filling

#### File Explorer
**Location**: `src/file-explorer.ts`
- Tree view of project structure
- Git status indicators
- Context menus
- Drag & drop support
- File watching integration

#### Editor Tabs
**Location**: `src/editor-tabs.ts`
- Multi-file editing
- Monaco editor integration
- Syntax highlighting
- Git diff visualization
- Unsaved changes tracking

#### Git Integration
**Location**: `src/vscode-scm-view.ts`
- Stage/unstage files
- Commit interface
- Push/pull/sync operations
- Branch management
- Diff visualization

#### Memory Dashboard
**Location**: `src/components/memory-dashboard.ts`
- Real-time statistics
- Connected tools display
- Activity stream
- Integration guide
- Export/import functionality

---

## Memory Service Infrastructure

### Architecture Overview
```
External AI Tools (Claude Code, Gemini, etc.)
         ↓
    HTTP/REST API (Dynamic Port)
         ↓
Memory Service (Express + HTTP Server)
         ↓
    IPC Channel
         ↓
   Main Process
         ↓
   SQLite Database
```

### Critical Implementation Details

#### Express Server Configuration
The Memory Service uses Express for API routing but **MUST** attach the Express app to the HTTP server:
```typescript
// CRITICAL: Attach Express app to HTTP server
this.server = http.createServer(this.app);  // NOT just http.createServer()
this.wss = new WebSocketServer({ server: this.server });
```

#### Dynamic Port Allocation
- Service starts on preferred port 3457
- If unavailable, ProcessManager allocates next available port (3458-3557)
- Port is communicated to main process via IPC
- Frontend discovers port dynamically via IPC handlers

### Service Capabilities

#### 1. Memory Queries
- Pattern matching against conversation history
- Thematic clustering
- Confidence scoring
- Context-aware responses

#### 2. Learning Contributions
- Accept knowledge from external tools
- Categorize by type and domain
- Track success/failure patterns
- Build collective intelligence

#### 3. Tool Registration
- Token-based authentication
- Usage tracking per tool
- Rate limiting
- Activity monitoring

### Statistics Tracking
```typescript
{
  totalMemories: 616,      // Total messages in database
  queriesToday: 6,         // Actual consensus queries from conversation_usage table
  contributionsToday: 5,   // Messages added today via consensus
  connectedTools: 0,       // Currently connected external AI tools
  hitRate: 92,            // Query success rate percentage
  avgResponseTime: 45     // Milliseconds (exponential moving average)
}
```

### Memory Service Implementation Details

#### Process Architecture
- **Type**: Child process managed by ProcessManager
- **Entry Point**: `src/memory-service/index.ts`
- **Server**: `src/memory-service/server.ts`
- **Port**: 3457 (with automatic fallback to 3458-3460)
- **IPC Communication**: Fork with ts-node for TypeScript support

#### Database Access Pattern
```typescript
// Child process sends query via IPC
process.send({
  type: 'db-query',
  id: queryId,
  sql: 'SELECT COUNT(*) FROM messages',
  params: []
});

// Main process handles query
handleMemoryServiceDbQuery(msg) {
  db.all(msg.sql, msg.params, (err, rows) => {
    childProcess.send({
      type: 'db-result',
      id: msg.id,
      data: rows,
      error: err
    });
  });
}
```

#### Critical Files & Functions
1. **Main Process** (`src/index.ts`):
   - `initializeProcessManager()` - Line 862: Registers Memory Service
   - `handleMemoryServiceDbQuery()` - Line 2420: Database query handler
   - `startMemoryService()` - Line 2436: Start service via ProcessManager
   - IPC handlers - Line 948: `memory-service-start`, `memory-service-stop`

2. **ProcessManager** (`src/utils/ProcessManager.ts`):
   - Port allocation using PortManager
   - Health check monitoring
   - Auto-restart on crash (max 5 attempts)
   - IPC message routing

3. **PortManager** (`src/utils/PortManager.ts`):
   - `allocatePort()` - Ensures port availability
   - `killProcessOnPort()` - Cleans up stuck processes
   - `waitForService()` - Verifies service startup

#### Common Issues & Solutions

##### Issue: Memory Service Not Starting
**Symptoms**: 
- "Failed to start Memory Service" in console
- Memory Dashboard shows "Service Inactive"

**Troubleshooting**:
1. Check port availability:
   ```bash
   lsof -i :3457
   # Kill if occupied: kill -9 [PID]
   ```

2. Check for duplicate IPC handlers:
   - Search for duplicate `registerMemoryServiceHandlers` functions
   - Ensure only one registration in main process

3. Verify ProcessManager registration:
   - Check `initializeProcessManager()` is called in app.whenReady()
   - Verify process config has correct script path

##### Issue: Database Query Timeouts
**Symptoms**:
- "Database query timeout" errors
- Stats not updating in dashboard

**Root Causes**:
- IPC channel not established
- Main process database not initialized
- Message handler not registered

**Solution**:
- Ensure `handleMemoryServiceDbQuery` is defined before process starts
- Check database initialization completes before starting Memory Service

##### Issue: Statistics Not Updating
**Symptoms**:
- Dashboard shows 0 for all metrics
- "Failed to get today's count" in logs

**SQL Queries Used**:
```sql
-- Total memories
SELECT COUNT(*) as count FROM messages

-- Messages added today
SELECT COUNT(*) as count FROM messages 
WHERE date(timestamp) = date('now')

-- Actual queries today (not estimated)
SELECT COUNT(*) as usage_count 
FROM conversation_usage 
WHERE date(timestamp, 'localtime') = date('now', 'localtime')
```

**Note**: `queriesToday` shows ACTUAL consensus queries from `conversation_usage` table, not approximations.

#### Service Lifecycle

1. **Startup Sequence**:
   ```
   App Ready → initializeProcessManager() → Register Memory Service
   → User clicks Memory → IPC: memory-service-start 
   → ProcessManager.startProcess() → Fork child process with ts-node
   → Child sends 'ready' message → Port allocated → Service running
   ```

2. **Shutdown Sequence**:
   ```
   User closes Memory tab OR App quits
   → IPC: memory-service-stop → Send 'shutdown' message to child
   → Graceful shutdown (2s timeout) → SIGTERM if needed
   → Port released → Process terminated
   ```

3. **Auto-Restart Logic**:
   - Max 5 restart attempts
   - 3 second delay between restarts
   - Exponential backoff on repeated failures
   - Port reallocation on each restart

#### Health Monitoring
- Health check endpoint: `http://localhost:3457/health`
- Checked every 30 seconds by ProcessManager
- Returns: `{ status, port, database, uptime }`
- Auto-restart triggered on health check failures

---

## Git Integration Architecture

### Git Operations
**Location**: `src/git/`

#### Core Functions
```typescript
- getStatus()      // Working tree status with ahead/behind counts
- stage(files)     // Stage changes
- unstage(files)   // Unstage changes
- commit(message)  // Create commit with auto-refresh
- push()          // Smart push with strategy analysis
- pull()          // Pull from remote
- sync()          // Pull + Push
- getBranches()   // List branches
- switchBranch()  // Change branch with modal dialog
- getRepoStats()   // Repository and push size analysis
- analyzeStrategy() // Smart push strategy recommendations
```

#### Smart Push System (Enterprise-Grade)
**Location**: `src/git-push-*.ts`

##### Strategy Analysis Engine
```typescript
// Analyzes repository characteristics for optimal push strategy
class GitPushStrategyAnalyzer {
  static analyzeRepository(stats, gitStatus): RepositoryAnalysis {
    // Calculates ACTUAL push size (not repo size)
    // Uses git rev-list to measure only unpushed objects
    // Provides intelligent recommendations based on:
    - Push size (prioritized over repo size)
    - Commit count
    - Branch status (new/existing/diverged)
    - Large file detection
    - GitHub 2GB pack limits
  }
}
```

##### Available Push Strategies
1. **Standard Push** - Regular git push for normal repos
2. **Smart Chunked Push** - Batches commits (50→25→10→5→1) for large pushes
3. **Force Push** - Replace remote branch (non-main branches only)
4. **Fresh Branch** - Create new branch to avoid history issues
5. **Squash & Push** - Combine commits to reduce size
6. **Create Bundle** - Export as file for manual upload
7. **Clean History First** - BFG cleanup for oversized repos

##### Custom Command Support
- **Cross-branch pushing**: Push local branch to different remote branch
- **Force with lease**: Safer force push with protection
- **Dry run validation**: Test commands before execution
- **User-defined commands**: Full flexibility for enterprise workflows

### Smart Push Dialog Architecture
**Location**: `src/git-push-dialog.ts`

#### Dialog Structure
```typescript
class GitPushDialog {
  // Comprehensive push strategy selection
  // Real-time repository analysis
  // Interactive option configuration
}
```

#### Key Features
1. **Repository Analysis Display**:
   - Shows actual push size (not repo size)
   - Commit count to be pushed
   - Branch status (new/existing/diverged)
   - Large file warnings
   - Risk assessment

2. **Strategy Cards**:
   - Visual cards for each strategy
   - Recommended badge on optimal choice
   - Pros/cons for each approach
   - Estimated time to complete
   - Requirements and warnings

3. **Advanced Options Panel**:
   - Force with Lease (safer force push)
   - Include Tags
   - Set Upstream
   - Dry Run (test without pushing)
   - Commit Limit (for chunked push)
   - Custom Command (full flexibility)
   - Atomic Push
   - Sign Push (GPG)
   - Thin Pack

4. **Custom Command Support**:
   - Full command input field
   - Supports cross-branch pushing
   - Example: `git push origin local:main`
   - Dry run validation before execution
   - IPC handler properly processes custom commands

5. **UI/UX Design**:
   - Dark theme matching VS Code
   - Smooth animations
   - Clear visual hierarchy
   - Responsive layout
   - Keyboard shortcuts (Escape to close)

### IPC Handlers for Git Operations
**Location**: `src/index.ts`

#### Critical Handlers
```typescript
// Repository statistics with push size calculation
ipcMain.handle('git-repo-stats', async (event, repoPath) => {
  // Calculates actual push size using git rev-list
  // Provides both repo size and push size
  // Passes gitStatus for context
});

// Smart push execution
ipcMain.handle('git-push', async (event, options) => {
  // Handles all push strategies
  // Supports custom commands
  // Implements chunked push logic
  // Returns detailed progress
});

// Dry run validation
ipcMain.handle('git-push-dry-run', async (event, options) => {
  // Tests push without execution
  // Properly handles custom commands
  // Shows what would be pushed
});
```

#### Bug Fixes Applied
1. **Custom Command in Dry Run**: Fixed handler to respect customCommand option
2. **Push Size Calculation**: Added git rev-list for accurate push size
3. **Shell Parameter**: Changed from `shell: true` to `shell: '/bin/bash'`

### Repository Cleanup & BFG Integration

#### Large Repository Management
**Problem**: Repositories exceeding GitHub's 2GB pack limits
**Solution**: BFG Repo Cleaner integration for history cleanup

##### BFG Usage Examples
```bash
# Remove large files from history
bfg --strip-blobs-bigger-than 100M

# Remove specific directories (e.g., build artifacts)
bfg --delete-folders target

# Clean up after BFG
git reflog expire --expire=now --all
git gc --prune=now --aggressive
```

##### Real-World Case Study
- **Initial Size**: 11GB repository
- **Issue**: Rust target directories in history
- **Solution**: `bfg --delete-folders target`
- **Result**: Reduced to 5.9GB (46% reduction)
- **Preserved**: All commit messages and history

### Authentication System
**Location**: `src/git/authentication/`

#### Askpass Helper
- Intercepts Git credential requests
- Shows authentication dialog
- Securely passes credentials
- Supports username/password and tokens

### Visual Interface

#### File Explorer Git Integration
**Implementation**: `src/vscode-explorer-exact.ts`
- **Real-time Git Status Indicators**:
  - Modified (M) - Orange color (#e2c08d)
  - Added (A) - Green color (#73c991)
  - Deleted (D) - Red color (#f48771)
  - Untracked (U) - Green with different shade
  - Renamed (R) - Blue color (#4fc3f7)
- **Visual Feedback**:
  - Badge indicators next to filenames
  - Color-coded file labels
  - Data attributes for CSS styling
  - Tooltips with Git status details
- **Auto-refresh on Changes**:
  - Debounced refresh (500ms) on editor changes
  - Immediate refresh after save operations
  - Git watcher for external changes

#### Source Control Panel
**Implementation**: `src/vscode-scm-view.ts`

##### Layout Architecture
The Source Control panel uses a sophisticated flexbox layout to ensure all content remains visible and accessible:

```typescript
// Main container structure
<div class="scm-view" style="
  display: flex;
  flex-direction: column;
  height: calc(100vh - 22px);  // Accounts for bottom status bar
  overflow: hidden;
">
  <!-- Fixed sections at top -->
  <div class="scm-status-bar">    // Branch info & sync indicators
  <div class="scm-view-header">    // Toolbar with Git actions
  <div class="scm-input-container"> // Commit message input
  
  <!-- Scrollable content area -->
  <div style="flex: 1; overflow-y: auto;">
    <div class="scm-view-content">  // Resource groups
    <div id="git-graph-container">  // Commits section
  </div>
</div>
```

##### Key Design Features
1. **Branch Status Bar** (Fixed at top):
   - Displays current branch name with clickable branch switcher
   - Push/Pull indicators with click-to-action:
     - Blue badge (↑N) - commits ahead, click to push
     - Red badge (↓N) - commits behind, click to sync
   - Always visible (shows 0 when no commits)
   - Border-bottom for visual separation
   - `flex-shrink: 0` to prevent compression

2. **Resource Groups** (Independently scrollable):
   - Each section has `max-height: 200px` with `overflow-y: auto`
   - Three main groups:
     - **Staged Changes**: Files in index ready to commit
     - **Changes**: Modified tracked files (unstaged)
     - **Untracked**: New files not yet tracked by Git
   - Individual file actions (stage/unstage/discard)
   - Group actions (stage all, unstage all, discard all)
   - Proper handling of `working_dir` property from simple-git
   - Fallback support for `working` property
   - TypeScript interface updated to include `working_dir?: string`

3. **Commits Section** (Bottom scrollable):
   - Git Graph container with commit history
   - `max-height: 200px` with independent scrolling
   - Shows recent commits with messages and timestamps
   - 5px padding-bottom to ensure last item visibility

##### Scrolling Behavior
- **Main container**: Uses flexbox with `flex: 1` for scrollable area
- **Individual sections**: Each has its own scrollbar when content exceeds 200px
- **Parent container**: `#git-content` has `padding: 0` to eliminate gaps (fixed in CSS)
- **Overflow management**: Prevents content from extending below app's bottom bar
- **Height calculation**: `calc(100vh - 22px)` accounts for bottom status bar

##### Action Buttons
- **File-level actions**:
  - Stage/unstage individual files
  - Discard changes per file
  - Delete untracked files (trash icon)
  - Open diff view on click
- **Group-level actions**:
  - Stage all (only stages tracked files, excludes untracked)
  - Unstage all for staged section
  - Discard all changes in group
- **Global Git operations**:
  - Commit (Ctrl+Enter in message box)
  - Push (toolbar button or click ahead indicator)
  - Pull (toolbar button)
  - Sync (pull then push)
  - **Smart Push Button** - Launches intelligent push dialog
  - **Refresh Button** - Removed to reduce UI clutter (v1.8.2)
    - Click ahead (↑) → Launch Smart Push
    - Click behind (↓) → Fetch then Smart Push
    - Real-time updates after operations
- **Panel Refresh System** (Production-Grade):
  - `recreatePanel()` method for complete DOM refresh
  - Auto-triggered after commits and pushes
  - Preserves Git Graph view reference
  - DOM safety checks with auto-recovery
  - No polling - fully event-driven
- **UI Layout**:
  - Scrollable content area with flexbox layout
  - Fixed height sections (header, commit input, status bar)
  - Content area uses `flex: 1 1 auto` with `overflow-y: auto`
  - Git graph section reduced to 300px for more file space
  - Width increased to 320px for better button visibility
- **Event-driven refresh** (no polling intervals)
- **Refresh Triggers**:
  - File save operations
  - Git operations (commit, stage, unstage)
  - Manual refresh button (properly wired to window.scmView)
  - Folder changes
  - Editor content changes (debounced 500ms)

#### Git Decoration Provider
**Implementation**: `src/git-decoration-provider.ts`
- **Centralized Git status management**
- **Event-driven updates** (no polling)
- **Caches file status** in Map structure
- **Provides decoration data**:
  - Color coding for file labels
  - Status badges (M, A, D, U, R)
  - Tooltips with detailed status
- **Integrates with**:
  - File Explorer (decorations via `updateGitDecorationsInPlace()`)
  - Source Control view (file status)
  - Editor tabs (triggers refresh on save/change)
- **Update triggers**:
  - File saves (immediate)
  - Editor content changes (debounced 500ms)
  - Git operations
  - Manual refresh button
- **In-place DOM updates** without tree rebuilds
- **Handles both `working_dir` and `working` properties** from different Git libraries

### Smart Push System Architecture

#### Overview
**Purpose**: Intelligent git push handling for repositories of any size, with special focus on large repositories (>2GB) that exceed GitHub's limits.

#### Core Components

##### 1. Push Strategy Analyzer
**Location**: `src/git-push-strategy.ts`
**Responsibilities**:
- Analyzes repository characteristics (size, commit count, branch status)
- Determines optimal push strategy based on repository analysis
- Provides multiple strategy options with clear recommendations

**Strategy Types**:
```typescript
enum PushStrategy {
  REGULAR = 'regular',        // Standard git push
  CHUNKED = 'chunked',        // Break into batches for large repos
  FORCE = 'force',            // Force push (with warnings)
  FRESH_BRANCH = 'fresh-branch', // Create new branch
  SQUASH = 'squash',          // Squash commits first
  BUNDLE = 'bundle',          // Create bundle file
  CLEANUP_FIRST = 'cleanup-first' // Clean history before push
}
```

##### 2. Smart Push Dialog
**Location**: `src/git-push-dialog.ts`
**Features**:
- **Repository Statistics**: Shows repo size, commit count, branch status
- **AI Recommendations**: Displays best strategies for the specific situation
- **Push Options Section**: Common git flags accessible via checkboxes
- **Advanced Options**: Collapsible section for power users
- **No Confidence Scores**: Removed confusing percentages in favor of clear recommendations

**Dialog Structure**:
```
┌─────────────────────────────────────────────────┐
│ Smart Push - [Repository Stats]                 │
├─────────────────────────────────────────────────┤
│ 📊 RECOMMENDED STRATEGIES                       │
│ ┌─────────────────────────────────────────────┐│
│ │ [Strategy Cards with RECOMMENDED badge]     ││
│ │ - Icon, Name, Description                   ││
│ │ - Pros/Cons lists                          ││
│ │ - Estimated time                           ││
│ └─────────────────────────────────────────────┘│
│                                                 │
│ ⚙️ PUSH OPTIONS                                │
│ □ Force with lease (--force-with-lease)        │
│ □ Include tags (--tags)                        │
│ □ Set upstream (-u)                            │
│ □ Dry run first (--dry-run)                    │
│                                                 │
│ ▼ Advanced Options                             │
│   - Commit limit (HEAD~N)                      │
│   - Custom git command                         │
│   - Atomic, Sign, Thin pack                    │
└─────────────────────────────────────────────────┘
```

##### 3. Push Executor
**Location**: `src/git-push-executor.ts`
**Capabilities**:
- Executes selected strategy with user options
- Shows progress dialog during execution
- Handles failures with automatic fallbacks
- Supports dry run for previewing operations
- Collects options from UI and passes to appropriate handlers
- Routes custom commands through secure IPC

**Option Handling**:
```typescript
interface PushOptions {
  forceWithLease?: boolean;  // Safer force push
  includeTags?: boolean;      // Push tags with commits
  setUpstream?: boolean;      // Set tracking branch
  dryRun?: boolean;          // Preview without pushing
  commitLimit?: number;       // Push only last N commits
  customCommand?: string;     // Override with custom command
  atomic?: boolean;          // All refs must succeed
  signPush?: boolean;        // GPG sign the push
  thinPack?: boolean;        // Optimize pack size
}
```

##### 4. Git Consensus Advisor
**Location**: `src/git-consensus-advisor.ts`
**Purpose**: AI-powered strategy recommendations (future enhancement)
- Analyzes repository state
- Provides intelligent recommendations
- Explains reasoning for each strategy
- Lists potential risks

#### Push Implementation Details

##### Standard Push with Options
**Status**: ✅ Fully Implemented
- Executes `git push` with any combination of selected options
- Automatically falls back to chunked if size limit hit
- Supports all standard git push flags through `pushWithOptions()`
- Options dynamically build the git command

##### Smart Chunked Push
**Status**: ✅ Fully Implemented
**Algorithm**: (`src/git-chunked-push-main.ts`)
1. Analyzes repository size with `git count-objects -vH`
2. Gets unpushed commits with `git rev-list`
3. Breaks commits into manageable batches
4. Pushes batches sequentially (50, 25, 10, 5, 1)
5. Handles partial failures gracefully
6. Provides detailed progress feedback

**Implementation**: Uses `GitChunkedPushMain.pushInBatches()`
```typescript
// Progressive batch sizes for reliability
const batchSizes = [50, 25, 10, 5, 1];
// Each batch: git push origin HEAD~N:branch
// Handles "pack exceeds maximum allowed size" errors
```

##### Force Push Variants
**Status**: ✅ Fully Implemented
- **Force with Lease**: Default safer option via `pushForceWithLease()`
- **Force**: Full overwrite (requires explicit confirmation)
- **Custom Force Options**: Via `pushWithOptions({ forceWithLease: true })`

##### Special Operations
**Implementation Status**:
- ✅ **Fresh Branch**: Creates new branch, switches, and pushes
- ⚠️ **Squash & Push**: Partially implemented (doesn't actually squash yet)
- ℹ️ **Bundle Creation**: Shows instructions (manual process)
- ℹ️ **Cleanup First**: Provides BFG guidance (manual process)

#### IPC Communication

##### Main Process Handlers
**Location**: `src/index.ts`
**Status**: ✅ Fully Implemented

```typescript
// Basic push operations
ipcMain.handle('git-push', async () => {...});
ipcMain.handle('git-push-chunked', async () => {...});
ipcMain.handle('git-repo-stats', async () => {...});

// Advanced operations (all implemented)
ipcMain.handle('git-push-with-options', async (event, options) => {
  // Builds dynamic git command with all selected options
  // Supports: force-with-lease, tags, upstream, atomic, sign, thin, commit-limit
});
ipcMain.handle('git-push-force-lease', async () => {
  // Executes git push --force-with-lease
});
ipcMain.handle('git-push-custom', async (event, command) => {
  // Executes custom git push command with security validation
  // Validates command starts with "git push" for security
  // 10-minute timeout for large operations
});
ipcMain.handle('git-push-dry-run', async (event, options) => {
  // Executes push with --dry-run flag to preview
});
```

##### Renderer API
**Location**: `src/preload.ts`
**Status**: ✅ Fully Implemented

```typescript
gitAPI: {
  // Core operations
  push: () => ipcRenderer.invoke('git-push'),
  pushChunked: () => ipcRenderer.invoke('git-push-chunked'),
  getRepoStats: () => ipcRenderer.invoke('git-repo-stats'),
  
  // Advanced push operations (all implemented)
  pushWithOptions: (options: any) => 
    ipcRenderer.invoke('git-push-with-options', options),
  pushForceWithLease: () => 
    ipcRenderer.invoke('git-push-force-lease'),
  pushCustom: (command: string) => 
    ipcRenderer.invoke('git-push-custom', command),
  pushDryRun: (options?: any) => 
    ipcRenderer.invoke('git-push-dry-run', options)
}
```

#### User Experience Flow

1. **User clicks Push button** in SCM toolbar
2. **System analyzes repository**:
   - Gets size via `git count-objects -vH`
   - Counts commits ahead/behind
   - Checks branch tracking status
3. **Smart Push Dialog appears** with:
   - Repository statistics
   - Recommended strategies (no confusing percentages)
   - Push options checkboxes
   - Advanced options (collapsed)
4. **User selects strategy** and optionally customizes options
5. **Executor runs push** with progress feedback
6. **Success/failure handling** with clear messages

#### Large Repository Optimization

##### Problem
- GitHub has 2GB pack size limit
- Repositories >2GB fail with standard push
- Users need guidance for large repo handling

##### Solution
- **Automatic detection** of large repositories
- **Smart Chunked Push** as default for >2GB
- **Progressive batch sizes** (50→25→10→5→1)
- **Automatic fallback** from regular to chunked
- **Clear user communication** about what's happening

##### Performance Considerations
- Chunked push takes longer but ensures success
- Network interruptions handled with resume capability
- Progress feedback keeps users informed
- Partial success tracking for recovery

#### Current Implementation Status

##### ✅ Completed Features
1. **Push Options System**:
   - All common git push flags implemented
   - Dynamic command building based on UI selections
   - Security validation for custom commands
   - Dry run preview functionality

2. **Smart Push Dialog**:
   - Repository analysis and size detection
   - Strategy recommendations without confusing percentages
   - Push options section with checkboxes
   - Advanced options in collapsible section
   - Custom command support for power users

3. **Push Strategies**:
   - Standard push with all options
   - Smart chunked push for large repos
   - Force push with lease (safer)
   - Fresh branch creation and push
   - Dry run simulation

##### ⚠️ Partially Implemented
1. **Squash & Push**: UI present but needs proper `git reset --soft` implementation
2. **AI Recommendations**: Framework exists but not connected to Consensus Engine

##### 📋 Future Enhancements

1. **AI Integration**:
   - Connect GitConsensusAdvisor to actual Consensus Engine
   - Learn from user patterns and preferences
   - Predict optimal strategy based on repository history

2. **Advanced Git Features**:
   - Git LFS detection and automatic migration
   - Shallow clone recommendations for huge histories
   - Automated history cleanup with BFG integration
   - Push range support (HEAD~N:branch)

3. **UI/UX Improvements**:
   - Real-time command preview with syntax highlighting
   - Visual progress bars for chunked operations
   - Push operation history and statistics
   - Saved push configuration profiles
   - Inline help and tooltips for each option

4. **Performance Optimizations**:
   - Parallel chunk pushing for faster transfers
   - Resume capability for interrupted pushes
   - Background push with notification on completion

### Event-Driven Update Architecture

#### Philosophy
The application uses a **purely event-driven architecture** with no polling intervals. This ensures optimal performance and prevents unnecessary CPU usage.

#### Recent Performance Fixes (v1.9.0)
1. **Removed All Polling Intervals**:
   - ~~SCM view 2-second refresh~~ → Event-driven only
   - ~~GitDecorationProvider 2-second polling~~ → On-demand updates
   - Result: 0% idle CPU usage

2. **Fixed Explorer Constant Refresh Bug**:
   - Problem: Tree collapsed/expanded every 2 seconds
   - Solution: `updateGitDecorationsInPlace()` method
   - Result: Stable tree with preserved expansion state

3. **Fixed Scroll Reset Bug**:
   - Problem: Scroll position reset to top on every update
   - Solution: Save/restore scroll position in `render()`
   - Uses `requestAnimationFrame()` for smooth restoration

4. **Fixed Double Folder Dialog**:
   - Problem: AI tools showed second dialog after folder selection
   - Solution: `window.openFolder(path?)` accepts optional path
   - Result: Single unified folder management system

5. **Optimized Git Updates**:
   - Debounced refresh on typing (500ms)
   - Immediate refresh on save
   - In-place DOM updates for decorations
   - No tree rebuilds for status changes

6. **Fixed Source Control Untracked Files Display**:
   - Problem: Untracked files not showing in Source Control panel
   - Root cause: simple-git uses `working_dir` property, not `working`
   - Solution: Updated to check both `working_dir` and `working` properties
   - Result: All three sections (Staged, Changes, Untracked) display correctly

7. **Fixed Source Control Refresh Button**:
   - Problem: Refresh button not triggering updates
   - Root cause: window.scmView reference not consistently set
   - Solution: Ensure both window.gitUI and window.scmView are set when creating VSCodeSCMView
   - Result: Manual refresh button works properly

8. **Added Scrollable Source Control Panel**:
   - Problem: Can't see all files when many changes exist
   - Solution: Implemented proper flexbox layout with scrollable content area
   - Changes: Content area gets `flex: 1 1 auto`, other sections `flex-shrink: 0`
   - Result: Smooth scrolling through all file changes

9. **Fixed Stage All Behavior**:
   - Problem: Stage All was including untracked files
   - Solution: Filter to only stage tracked files with modifications
   - Result: Matches VS Code behavior - untracked files must be staged individually

#### Update Flow
```
User Action → Event Trigger → Targeted Update → DOM Manipulation
```

#### Key Principles
1. **No Polling**: Zero `setInterval` calls for UI updates
2. **Targeted Updates**: Only update what changed
3. **Scroll Preservation**: Maintain user position during updates
4. **Debounced Operations**: Prevent excessive updates while typing
5. **In-Place Updates**: Modify existing DOM rather than rebuild

#### Event Sources
- **File Operations**:
  - Save (Ctrl/Cmd+S) → Immediate Git refresh
  - Content change → Debounced refresh (500ms)
  - Auto-save → Triggered after 1000ms idle
- **Git Operations**:
  - Commit/Stage/Unstage → Full refresh
  - External changes → File watcher triggers
- **User Interactions**:
  - Folder expand/collapse → Render with scroll preservation
  - File selection → Direct DOM update
  - Tab switches → Monaco editor swap

#### Performance Features
- **updateGitDecorationsInPlace()**: Updates badges without tree rebuild
- **requestAnimationFrame()**: Ensures smooth scroll restoration
- **Document Fragments**: Batch DOM operations
- **Lazy Loading**: Load directory contents on-demand
- **Debounced Saves**: Prevent excessive file writes

---

## Consensus Engine Architecture

### 4-Stage Pipeline
```
1. Generator Stage
   ├── Input: User prompt
   ├── Model: Selected generator model
   └── Output: Initial response

2. Refiner Stage
   ├── Input: Generator output
   ├── Model: Selected refiner model
   └── Output: Enhanced response

3. Validator Stage
   ├── Input: Refined response
   ├── Model: Selected validator model
   └── Output: Validated response

4. Curator Stage
   ├── Input: Validated response
   ├── Model: Selected curator model
   └── Output: Final response
```

### Model Selection
- **323+ models** via OpenRouter
- **Direct mode** for simple queries (single model)
- **Full consensus** for complex queries (4 stages)
- **Custom profiles** for specialized workflows

### Streaming Architecture
- Token-by-token streaming
- Stage progress indicators
- Real-time UI updates
- Cost tracking per stage

---

## Security & Authentication

### License System
- **User ID**: UUID per installation
- **License Key**: For premium features
- **Tier System**: FREE, PRO, UNLIMITED
- **Daily Limits**: Based on tier
- **D1 Validation**: Server-side verification

### Data Security
- **Local Storage**: SQLite with file permissions
- **API Keys**: Encrypted in database
- **IPC Security**: Preload script sanitization
- **File Access**: Main process only
- **Git Credentials**: Secure askpass handling

### Trust System (Future)
- Directory-level permissions
- Explicit user consent for file access
- Audit logging
- Security event tracking

---

## Performance & Optimization

### Startup Optimization
- Lazy loading of components
- Deferred database queries
- Parallel initialization
- Cached configurations

### Memory Management
- Efficient IPC message passing
- Streaming for large files

### ResizeObserver Performance (v1.7.5)
**Problem**: ResizeObserver loop errors when callbacks trigger synchronous DOM changes
**Solution**: Use requestAnimationFrame to defer DOM updates

```typescript
// INCORRECT - Causes loop errors
const resizeObserver = new ResizeObserver(() => {
    updateNavigationArrows(); // Synchronous DOM changes
});

// CORRECT - Deferred updates
let resizeAnimationFrame: number | null = null;
const resizeObserver = new ResizeObserver(() => {
    if (resizeAnimationFrame !== null) {
        cancelAnimationFrame(resizeAnimationFrame);
    }
    resizeAnimationFrame = requestAnimationFrame(() => {
        updateNavigationArrows(); // Deferred DOM changes
        resizeAnimationFrame = null;
    });
});
```

**Benefits**:
- Eliminates "loop completed with undelivered notifications" errors
- Improves performance by batching DOM updates
- Prevents infinite resize loops
- Ensures smooth 60 FPS animations
- Pagination for lists
- Resource cleanup on unmount

### Database Optimization
- Indexed queries
- Batch operations
- Connection pooling (for child processes)
- Periodic vacuum

### UI Performance
- Virtual scrolling for large lists
- Debounced file watching
- Throttled updates
- Web Workers for heavy computations

---

## Development & Deployment

### Production Requirements (CRITICAL)

**Core Principles**:
- **NO hardcoded paths or ports** - Everything dynamic via ProcessManager
- **ALL binaries bundled** - No external dependencies
- **ProcessManager as control tower** - Manages all processes and ports
- **PortManager for allocation** - Dynamic port assignment with fallbacks

#### Production Build Process
```bash
# Pre-build: Compile Memory Service
node scripts/build-memory-service.js

# Pre-build: Copy backend binary
cp ../hive/target/release/hive-backend-server-enhanced binaries/

# Build production app
npm run make
```

#### Optimized Dynamic Port Architecture (Revolutionary Design)

##### Overview
Our port management system represents a paradigm shift from traditional hardcoded ports to a fully dynamic, environment-adaptive architecture. This design ensures zero startup delays, handles restricted environments, and works across countless user configurations.

##### Core Architecture Components

**1. PortManager - The Intelligence Layer**
```typescript
class PortManager {
  // Pre-scanned port pools for instant allocation
  private static availablePortPool: Map<string, AvailablePort[]> = new Map();
  private static allocatedPorts: Map<string, number> = new Map();
  
  // Service-specific port ranges (strategically chosen to avoid conflicts)
  private static readonly PORT_RANGES: PortRange[] = [
    // Memory Service: Mid-range ports (less likely to conflict)
    { name: 'memory-service', start: 3457, end: 3560, priority: 1 },
    { name: 'memory-service-alt', start: 14500, end: 14600, priority: 2 },
    
    // Backend Server: Higher range for WebSocket services
    { name: 'backend-server', start: 8765, end: 8865, priority: 1 },
    { name: 'backend-server-alt', start: 19000, end: 19100, priority: 2 },
    
    // Terminal Services: Dedicated range for TTYD instances
    { name: 'ttyd-terminals', start: 7100, end: 7200, priority: 1 },
    
    // Debug/Development: Separate range for debugging
    { name: 'debug-server', start: 9230, end: 9330, priority: 1 },
  ];
}
```

**2. ProcessManager - The Control Tower**
```typescript
// ProcessManager integration with PortManager
async startProcess(name: string, config: ProcessConfig): Promise<boolean> {
  // No hardcoded ports - just indicate if service needs one
  if (config.port !== undefined) {
    // Allocate port dynamically - no specific port numbers!
    const port = await PortManager.allocatePortForService(name);
    info.port = port;
    
    // Pass port to service via environment
    env.PORT = port.toString();
    if (name === 'memory-service') {
      env.MEMORY_SERVICE_PORT = port.toString();
    }
  }
}
```

##### Startup Sequence (Optimized for Speed)

**Phase 1: Parallel Port Scanning (200ms)**
```typescript
// StartupOrchestrator.ts
async showSplashAndInitialize() {
  // Step 1: Launch splash screen
  this.createSplashWindow();
  
  // Step 2: Pre-scan all port ranges IN PARALLEL with splash
  this.updateSplash(5, 'Scanning available ports...');
  const portScanPromise = PortManager.initialize();
  
  // Step 3: Initialize services while scan completes
  await this.initializeServices();
  
  // Step 4: Ensure scan is complete before service startup
  await portScanPromise; // Typically completes in ~200ms
}
```

**Phase 2: Service Registration (Zero Hardcoded Ports)**
```typescript
// No more hardcoded ports anywhere!
processManager.registerProcess({
  name: 'memory-service',
  scriptPath: memoryServicePath,
  env: {
    NODE_ENV: app.isPackaged ? 'production' : 'development',
    // NO PORT SPECIFIED HERE - fully dynamic!
  },
  port: 1, // Just a flag indicating "needs a port"
  // No alternativePorts needed - PortManager handles everything
});

processManager.registerProcess({
  name: 'websocket-backend',
  scriptPath: consensusBackendPath,
  env: {
    RUST_LOG: 'info',
    // NO PORT SPECIFIED HERE - fully dynamic!
  },
  port: 1, // Just a flag indicating "needs a port"
});
```

##### Port Allocation Algorithm (Intelligent & Fast)

**1. Pre-Scan Strategy**
```typescript
private static async performInitialScan(): Promise<void> {
  // Scan all ranges in parallel (not sequential!)
  const scanPromises = this.PORT_RANGES.map(range => this.scanRange(range));
  await Promise.all(scanPromises);
  
  // Each range scanned with:
  // - 50ms timeout per port check
  // - 10 ports checked in parallel per batch
  // - Stop after finding 10 available ports per range
  // Total time: ~200ms for all ranges
}
```

**2. Allocation Strategy**
```typescript
static async allocatePortForService(serviceName: string): Promise<number> {
  // Step 1: Check if already allocated (instant)
  if (this.allocatedPorts.has(serviceName)) {
    const existing = this.allocatedPorts.get(serviceName);
    if (await this.checkPortQuick(existing)) return existing;
  }
  
  // Step 2: Get from pre-scanned pool (instant)
  const pools = this.getPoolsForService(serviceName);
  for (const pool of pools) {
    const available = this.availablePortPool.get(pool);
    if (available?.length > 0) {
      const port = available.shift();
      return port; // Instant return!
    }
  }
  
  // Step 3: Fallback to ephemeral port (guaranteed available)
  return this.getEphemeralPort(); // OS-assigned port
}
```

##### IPC Communication for Dynamic Ports

**Main Process Handlers**
```typescript
// No fallback ports - services must be running!
ipcMain.handle('websocket-backend-port', async () => {
  const port = processManager.getProcessStatus('websocket-backend')?.port;
  if (!port) throw new Error('Backend not running');
  return port;
});

ipcMain.handle('memory-service-port', async () => {
  const port = processManager.getProcessStatus('memory-service')?.port;
  if (!port) throw new Error('Memory Service not running');
  return port;
});
```

**Renderer Process Usage**
```typescript
// Renderer dynamically discovers ports
const backendPort = await window.electronAPI.getBackendPort();
const ws = new WebSocket(`ws://localhost:${backendPort}/ws`);

const memoryPort = await window.electronAPI.getMemoryServicePort();
const response = await fetch(`http://localhost:${memoryPort}/api/v1/memory/stats`);
```

##### Multi-Environment Adaptability

**1. Restricted Corporate Environments**
- Scans only non-privileged ports (>1024)
- Avoids common corporate proxy ports
- Falls back to ephemeral ports if all ranges blocked

**2. Developer Machines**
- Avoids common development ports (3000, 8080, 5000, etc.)
- Separate ranges for each service type
- No conflicts with webpack-dev-server, create-react-app, etc.

**3. Production Deployments**
- Consistent port allocation across restarts
- Graceful handling of port squatters
- Automatic cleanup of orphaned processes

##### Performance Metrics

**Startup Performance**
```
Traditional (Sequential Port Checking):
- Check port 3457: 100ms (if blocked)
- Try alternative: 100ms (if blocked)
- Find available: 100ms
- Total: 300ms+ PER SERVICE

Our Design (Parallel Pre-Scan):
- Pre-scan all ranges: 200ms (TOTAL)
- Allocate port: <1ms (from pool)
- Total: 200ms for ALL SERVICES
```

**Memory Efficiency**
```
Port Pool Storage:
- ~60 ports pre-scanned
- 16 bytes per port entry
- Total: <1KB memory overhead
```

##### Failure Recovery

**1. Port Exhaustion**
```typescript
// If all pre-scanned ports taken (rare)
if (pool.length === 0) {
  // Request ephemeral port from OS
  const port = await this.getEphemeralPort();
  // OS guarantees an available port
  return port;
}
```

**2. Service Restart**
```typescript
// ProcessManager handles graceful restart
on('process:crashed', async (name) => {
  // Release old port
  PortManager.releasePort(name);
  
  // Get new port on restart
  const newPort = await PortManager.allocatePortForService(name);
  // Restart with new port
});
```

##### Configuration & Customization

**Custom Port Ranges (Enterprise)**
```typescript
// Can be configured via environment variables
MEMORY_SERVICE_PORTS=20000-20100
BACKEND_SERVER_PORTS=21000-21100

// Or via config file
{
  "portRanges": {
    "memory-service": { "start": 20000, "end": 20100 },
    "backend-server": { "start": 21000, "end": 21100 }
  }
}
```

##### Benefits of This Architecture

**1. Zero Startup Delays**
- No sequential port checking
- No retry loops
- Instant allocation from pool

**2. Universal Compatibility**
- Works in restricted environments
- No hardcoded ports anywhere
- Adapts to any system configuration

**3. Developer Experience**
- No port conflict errors
- No "port already in use" messages
- Automatic port management

**4. Production Reliability**
- Consistent behavior across environments
- Graceful degradation
- Self-healing on failures

**5. Scalability**
- Easy to add new services
- No port planning needed
- Supports unlimited service instances

##### Implementation Status

**✅ Completed**:
- PortManager with pre-scan algorithm (200ms parallel scan)
- ProcessManager integration as control tower
- Dynamic port allocation with zero hardcoded ports
- Ephemeral port fallback from OS
- IPC handlers for dynamic port discovery
- Memory Service webpack build integration
- Production build with dynamic ports
- Removed ALL hardcoded port references
- Service name-based allocation (no port numbers needed)

**✅ Key Achievements**:
- **Zero startup delays** from port conflicts
- **Universal compatibility** across all environments
- **No hardcoded ports** anywhere in codebase
- **Instant allocation** from pre-scanned pools
- **Self-healing** with automatic port reallocation
- **Production ready** with optimized build process

**📋 Future Enhancements**:
- Configuration file for custom port ranges
- Port persistence across restarts (optional)
- Analytics on port usage patterns
- Multi-instance support with unique ports

### Build System
```bash
# Development
npm start           # Electron Forge dev server

# Production
npm run premake     # Build Memory Service
npm run package     # Package for current platform
npm run make       # Create distributables
```

### Configuration Files
```
├── package.json           # Dependencies and scripts
├── forge.config.ts        # Electron Forge config
├── webpack.main.config.ts # Main process webpack
├── webpack.renderer.config.ts # Renderer webpack
├── tsconfig.json         # TypeScript configuration
└── .env                  # Environment variables
```

### Testing Strategy
- Unit tests for utilities
- Integration tests for IPC
- E2E tests for user workflows
- Performance benchmarks

### Distribution
- **macOS**: .dmg installer
- **Windows**: .exe installer
- **Linux**: .AppImage / .deb
- **Auto-updates**: Electron updater

---

## System Interconnections

### Data Flow Example: Running Consensus
```
1. User enters prompt in chat
2. Renderer sends IPC: 'run-consensus'
3. Main process receives IPC
4. Main queries database for history
5. Main calls OpenRouter API (4 stages)
6. Streaming responses sent to renderer
7. Final response saved to database
8. Database syncs to D1 (if online)
9. Memory Service queries updated DB
10. Dashboard reflects new statistics
11. Usage counter updates in status bar
```

### Component Dependencies
```
Main Process
├── Depends on: SQLite, Node.js APIs
├── Provides: File system, IPC handlers
└── Manages: Child processes, windows

Renderer Process
├── Depends on: Main process IPC
├── Provides: User interface
└── Manages: UI state, user input

Memory Service
├── Depends on: Main process (for DB)
├── Provides: REST API for tools
└── Manages: External integrations

Git Integration
├── Depends on: File system, git binary
├── Provides: Version control UI
└── Manages: Repository state
```

---

## Complete AI CLI Tools Integration Pattern

### Executive Summary
This section documents the complete architectural pattern for integrating AI CLI tools into Hive Consensus. Each tool follows the same comprehensive integration pattern, enabling seamless installation, configuration, updates, and terminal launching with full Memory Service integration via MCP protocol.

### Core Integration Components

#### 1. Tool Registry & Metadata
**Location**: `src/utils/AI_CLI_TOOLS_REGISTRY.md`

Each AI CLI tool requires:
```typescript
interface CliToolDefinition {
  id: string;                    // Unique identifier (e.g., 'claude-code')
  name: string;                  // Display name (e.g., 'Claude Code')
  packageName: string;           // NPM/pip package (e.g., '@anthropic-ai/claude-code')
  packageManager: 'npm' | 'pip'; // Installation method
  command: string;               // Terminal command (e.g., 'claude')
  versionCommand: string;        // Version check (e.g., 'claude --version')
  versionPattern: RegExp;        // Version extraction pattern
  docsUrl: string;              // Official documentation URL
  description: string;          // Tool description
  memoryServiceCompatible: boolean; // MCP support flag
  resumeSupport: boolean;       // Supports --resume flag
}
```

#### 2. Detection System
**Location**: `src/main/cli-tools/detector.ts`

```typescript
class CliToolDetector {
  async detectTool(toolId: string): Promise<ToolStatus> {
    // 1. Check if tool command exists in PATH
    const exists = await this.commandExists(toolId);
    
    // 2. Get version if installed
    const version = exists ? await this.getVersion(toolId) : null;
    
    // 3. Check Memory Service connection
    const memoryConnected = await this.checkMemoryServiceConnection(toolId);
    
    // 4. Get installation path
    const path = await this.getToolPath(toolId);
    
    return {
      installed: exists,
      version,
      path,
      memoryServiceConnected,
      updateAvailable: await this.checkForUpdates(toolId, version)
    };
  }
}
```

#### 3. IPC Communication Layer
**Location**: `src/index.ts` (Main Process Handlers)

```typescript
// Complete set of IPC handlers for each tool
ipcMain.handle('cli-tool-detect', async (_, toolId) => { /* Detection logic */ });
ipcMain.handle('cli-tool-install', async (_, toolId) => { /* Installation logic */ });
ipcMain.handle('cli-tool-update', async (_, toolId) => { /* Update logic */ });
ipcMain.handle('cli-tool-configure', async (_, toolId) => { /* MCP configuration */ });
ipcMain.handle('cli-tool-launch', async (_, toolId) => { /* Terminal launch */ });
ipcMain.handle('cli-tool-uninstall', async (_, toolId) => { /* Uninstallation */ });
```

#### 4. UI Component Architecture
**Location**: `src/renderer.ts`

```typescript
interface CliToolCardUI {
  // Visual states
  states: {
    notInstalled: { buttons: ['Install', 'Docs'], color: 'gray' };
    installed: { buttons: ['Launch', 'Details', 'Configure', 'Update'], color: 'green' };
    updating: { buttons: ['Cancel'], status: '⬆️ Updating...', color: 'orange' };
    configuring: { buttons: [], status: '⚙️ Configuring...', color: 'blue' };
    error: { buttons: ['Retry', 'Help'], status: '❌ Error', color: 'red' };
  };
  
  // Dynamic updates without full refresh
  updateStatus(status: string): void;
  updateVersion(version: string): void;
  updateMemoryConnection(connected: boolean): void;
}
```

### Complete Integration Flow

#### Phase 1: Detection & Display
```mermaid
graph TD
    A[App Startup] --> B[Detect All CLI Tools]
    B --> C{For Each Tool}
    C --> D[Check Installation]
    D --> E[Get Version]
    E --> F[Check Memory Connection]
    F --> G[Display Card in UI]
    G --> H{Tool Installed?}
    H -->|Yes| I[Show Launch/Configure/Update]
    H -->|No| J[Show Install Button]
```

#### Phase 2: Installation Flow
```typescript
async function installCliTool(toolId: string): Promise<InstallResult> {
  // 1. Pre-installation checks
  await checkPrerequisites(toolId);  // Node.js, Python, etc.
  
  // 2. Execute installation
  const command = getInstallCommand(toolId);
  await executeCommand(command);
  
  // 3. Verify installation
  const installed = await verifyInstallation(toolId);
  
  // 4. Auto-configure if Memory Service compatible
  if (tool.memoryServiceCompatible) {
    await configureMemoryService(toolId);
  }
  
  // 5. Update UI
  return { success: true, version: getVersion(toolId) };
}
```

#### Phase 3: Memory Service Configuration
```typescript
async function configureMemoryService(toolId: string): Promise<ConfigResult> {
  // 1. Register with Memory Service
  const token = await registerTool(toolId);
  
  // 2. Save configuration
  await saveConfig('~/.hive/cli-tools-config.json', { token });
  
  // 3. Create MCP wrapper
  await createMCPWrapper(toolId, token);
  
  // 4. Update tool's MCP config
  await updateToolMCPConfig(toolId);
  
  return { success: true, token };
}
```

#### Phase 4: Launch Integration
```typescript
async function launchCliTool(toolId: string): Promise<LaunchResult> {
  // 1. Check previous launches
  const previousLaunch = await checkPreviousLaunch(toolId, currentFolder);
  
  // 2. Determine command
  const command = previousLaunch ? 
    `${tool.command} --resume` : 
    tool.command;
  
  // 3. Create terminal
  const terminal = await createTerminal({
    command,
    cwd: currentFolder,
    title: tool.name
  });
  
  // 4. Update global context
  await updateGlobalFolder(currentFolder);
  
  return { success: true, terminalId: terminal.id };
}
```

#### Phase 5: Update Mechanism
```typescript
async function updateCliTool(toolId: string): Promise<UpdateResult> {
  // 1. Determine update command
  const updateCommand = tool.packageManager === 'npm' ?
    `npm update -g ${tool.packageName}` :
    `pip install --upgrade ${tool.packageName}`;
  
  // 2. Execute update
  await executeCommand(updateCommand);
  
  // 3. Get new version
  const newVersion = await getVersion(toolId);
  
  // 4. Re-configure if needed
  if (tool.memoryServiceCompatible) {
    await verifyMemoryConnection(toolId);
  }
  
  return { success: true, version: newVersion };
}
```

### Terminal Integration Pattern

#### TTYD Terminal Management
**Location**: `src/services/TTYDManager.ts`

```typescript
class TTYDManager {
  async createTerminal(options: TerminalOptions): Promise<Terminal> {
    // 1. Allocate port (7100-7200 range)
    const port = await this.allocatePort();
    
    // 2. Spawn ttyd process
    const process = spawn('ttyd', [
      '--port', port,
      '--interface', '127.0.0.1',
      '--writable',
      '--',
      '/bin/zsh', '-c', `sleep 0.5 && ${options.command}; exec /bin/zsh -i`
    ]);
    
    // 3. Track PID for cleanup
    PidTracker.recordPid(process.pid, `ttyd-${options.id}`);
    
    // 4. Return terminal info
    return {
      id: options.id,
      url: `http://localhost:${port}`,
      port,
      process
    };
  }
}
```

### Button Functionality Matrix (Updated August 2025)

| Button | Purpose | Visual State | IPC Handler | Result |
|--------|---------|--------------|-------------|---------|
| **Install** | Install + Configure | Blue, primary | `cli-tool-install` | Tool installed, Memory Service configured automatically |
| **Launch** | Open in terminal | Blue, primary | `cli-tool-launch` | Terminal with tool running (smart setup for Grok) |
| **Details** | Refresh status | Green, secondary | `cli-tool-detect` | Updated version/status |
| **Update** | Update to latest | Gray, secondary | `cli-tool-update` | Latest version installed |
| **Docs** | Open documentation | Gray, minimal | N/A (opens URL) | Browser with docs |

**Note**: Configure button removed in favor of automatic configuration during installation

### File System Organization

```
~/.hive/
├── cli-tools-config.json       # Tool configurations and tokens
├── memory-service-mcp-wrapper.js # MCP bridge for Memory Service
├── ai-tools.db                 # Launch history database
└── tools/                       # Local tool installations

~/.claude/
├── .mcp.json                   # MCP server configurations
├── config.json                 # Claude Code settings
└── settings.json               # Hooks and preferences

/tmp/
└── hive-electron-poc.pids      # Process tracking for cleanup
```

### Database Schema for Tool Tracking

```sql
-- Tool installations
CREATE TABLE tool_installations (
  tool_id TEXT PRIMARY KEY,
  version TEXT,
  installed_at TEXT,
  last_updated TEXT,
  memory_token TEXT
);

-- Launch history (for --resume support)
CREATE TABLE launch_history (
  id INTEGER PRIMARY KEY,
  tool_id TEXT,
  project_path TEXT,
  launched_at TEXT,
  launch_count INTEGER DEFAULT 1
);

-- Update tracking
CREATE TABLE update_checks (
  tool_id TEXT PRIMARY KEY,
  last_checked TEXT,
  latest_version TEXT,
  current_version TEXT,
  update_available BOOLEAN
);
```

### Error Handling Patterns

```typescript
// Comprehensive error handling for each operation
try {
  const result = await operation();
  return { success: true, data: result };
} catch (error) {
  // Permission errors
  if (error.code === 'EACCES') {
    return { success: false, error: 'Permission denied', suggestion: 'Run with sudo' };
  }
  
  // Network errors
  if (error.code === 'ENOTFOUND') {
    return { success: false, error: 'Network error', suggestion: 'Check connection' };
  }
  
  // Command not found
  if (error.message.includes('command not found')) {
    return { success: false, error: 'Prerequisite missing', suggestion: 'Install Node.js/Python' };
  }
  
  // Generic fallback
  return { success: false, error: error.message };
}
```

### Testing Strategy

#### Unit Tests
```typescript
describe('CliToolDetector', () => {
  it('should detect installed Claude Code', async () => {
    const status = await detector.detectTool('claude-code');
    expect(status.installed).toBe(true);
    expect(status.version).toMatch(/\d+\.\d+\.\d+/);
  });
});
```

#### Integration Tests
```typescript
describe('CLI Tool Integration', () => {
  it('should complete full lifecycle', async () => {
    // Install
    await ipcRenderer.invoke('cli-tool-install', 'claude-code');
    
    // Configure
    await ipcRenderer.invoke('cli-tool-configure', 'claude-code');
    
    // Launch
    const result = await ipcRenderer.invoke('cli-tool-launch', 'claude-code');
    expect(result.success).toBe(true);
    
    // Update
    await ipcRenderer.invoke('cli-tool-update', 'claude-code');
  });
});
```

### Performance Considerations

1. **Lazy Loading**: Tools detected on-demand, not all at startup
2. **Caching**: Version info cached for 5 minutes
3. **Non-blocking**: All operations async with UI feedback
4. **Debouncing**: Status refreshes debounced to prevent spam
5. **Parallel Operations**: Multiple tools can update simultaneously

### Security Model

1. **Token Isolation**: Each tool gets unique Memory Service token
2. **Command Validation**: Tool IDs validated against whitelist
3. **Path Sanitization**: All paths validated before execution
4. **Permission Checks**: Graceful degradation without sudo
5. **Audit Logging**: All operations logged for security review

### Architectural Principles for AI CLI Tools

**1. Separation of Concerns**:
   - **Hive's Role**: Installation, detection, launching, UI management
   - **Tool's Role**: Authentication, configuration, API keys, actual work
   - **Exception**: Cline (we manage its OpenRouter key for user convenience)

**2. Non-Invasive Integration**:
   - Tools work exactly as they do standalone
   - No modification of tool behavior
   - No parsing of tool output
   - Pure terminal hosting

**3. User Experience First**:
   - One-click installation with progress feedback
   - Automatic detection on startup
   - Clear visual states (installed, updating, error)
   - Immediate UI updates (cache clearing fix)
   - Sensible defaults (tools manage own config)

**4. Error Recovery**:
   - Graceful degradation without sudo
   - Clear error messages with suggestions
   - Retry mechanisms for transient failures
   - Process cleanup on app exit

**5. Performance Optimization**:
   - Lazy detection (on-demand, not at startup)
   - 5-minute cache for version info
   - Parallel operations where possible
   - Non-blocking async operations

### Replication Guide for New AI CLI Tools

#### CRITICAL: Pre-Implementation Research Phase

##### ⚠️ MANDATORY FIRST STEP: Read Existing Tool Documentation

Before implementing ANY new AI CLI tool, you MUST become an expert by studying our existing comprehensive documentation:

1. **READ Our Complete Local Documentation FIRST**
   ```
   docs/cli-tools/
   ├── claude-code.md     # ✅ 500+ lines - ALREADY WRITTEN
   ├── gemini-cli.md      # ✅ Complete docs - ALREADY WRITTEN
   ├── qwen-code.md       # ✅ Full guide - ALREADY WRITTEN
   ├── openai-codex.md    # ✅ Detailed docs - ALREADY WRITTEN
   ├── aider.md           # ✅ Comprehensive - ALREADY WRITTEN
   └── cline.md           # ✅ Full coverage - ALREADY WRITTEN
   ```
   
   **DO NOT START CODING WITHOUT READING THE DOCS!**
   
   Each document contains:
   - Installation methods and prerequisites
   - Authentication flows and API key setup
   - Command-line flags and options
   - Configuration file locations and formats
   - Advanced features (MCP, hooks, sessions)
   - Pricing and limitations
   - Troubleshooting guides
   - SDK integration examples

2. **Understand Unique Tool Characteristics**
   - **Authentication Methods**: OAuth, API keys, tokens, browser-based
   - **Command Variations**: Tool command vs package name differences
   - **Special Flags**: `--resume`, `--continue`, `--no-git`, etc.
   - **Configuration Files**: Tool-specific config locations and formats
   - **Environment Variables**: Required env vars for operation
   - **Prerequisites**: System dependencies, language runtimes
   - **Limitations**: Rate limits, free tiers, usage quotas

3. **Document Advanced Options**
   ```typescript
   interface ToolAdvancedOptions {
     // Claude Code specific
     claudeCode: {
       resumeSupport: true,
       mcp: true,
       hooks: true,
       customInstructions: true,
       contextWindow: 200000
     },
     
     // Gemini specific
     geminiCli: {
       freeQuota: 1000,  // requests per day
       requiresGoogleAuth: true,
       regionRestrictions: ['US', 'EU']
     },
     
     // Aider specific
     aider: {
       gitIntegration: 'deep',
       editFormat: 'diff' | 'whole',
       repoMapStyle: 'tree' | 'tags',
       voiceMode: true
     }
   }
   ```

4. **Research Tool-Specific Features**
   - **MCP Support**: Not all tools support Model Context Protocol
   - **Hooks System**: Tool-specific event hooks (PreToolUse, PostToolUse)
   - **Session Management**: Resume, continue, or stateless
   - **Context Management**: How each tool handles context limits
   - **Cost Structure**: Free tier limits, paid features
   - **Model Selection**: Available models and routing logic

5. **Test Tool Manually First**
   ```bash
   # Install and test manually before automation
   npm install -g @anthropic-ai/claude-code
   claude --version
   claude doctor  # Tool-specific diagnostics
   claude --help  # Understand all flags
   
   # Test advanced features
   claude --resume  # Session continuation
   claude mcp list  # MCP servers
   /hooks          # Hook configuration
   ```

##### Current AI CLI Tools Implementation Status

**Integrated Tools (As of August 2025)**:

| Tool | Package | Command | Version Detection | Special Handling | Status |
|------|---------|---------|-------------------|------------------|---------|
| **Claude Code** | `@anthropic-ai/claude-code` | `claude` | `/claude-code\/(\d+\.\d+\.\d+)/` | `--resume` support, MCP integration | ✅ Complete |
| **Gemini CLI** | `@google/gemini-cli` | `gemini` | `/Gemini CLI v(\d+\.\d+\.\d+)/` | Free tier (1000 req/day), No `--resume` | ✅ Complete |
| **Qwen Code** | `@alibaba/qwen-code` | `qwen` | `/(?:qwen\/\|v?)(\d+\.\d+\.\d+)/` | Self-managed auth | ✅ Complete |
| **OpenAI Codex** | `@openai/codex-cli` | `codex` | `/codex-cli (\d+\.\d+\.\d+)/` | Self-managed auth | ✅ Complete |
| **Cline** | `@yaegaki/cline-cli` | `cline-cli` | `/(\d+\.\d+\.\d+)/` | **Special: Uses Hive's OpenRouter API key** | ✅ Complete |
| **Grok CLI** | `@vibe-kit/grok-cli` | `grok` | `/(\d+\.\d+\.\d+)/` | MCP support, Morph Fast Apply (4500 tokens/sec) | ✅ Complete |

**Key Implementation Patterns**:

1. **Self-Managed Tools** (Default Pattern):
   - Claude Code, Gemini CLI, Qwen Code, OpenAI Codex, Grok CLI
   - Handle their own API keys and authentication
   - We just spawn them in terminals
   - Users configure directly within the tool

2. **Hive-Managed Tools** (Special Case):
   - **Cline only** - We manage its API configuration
   - Uses Hive's OpenRouter API key from database
   - Dynamically syncs configuration on every launch
   - Environment variables passed: `OPENAI_API_KEY`, `OPEN_ROUTER_API_KEY`, `OPENROUTER_API_KEY`

3. **Version Detection Nuances**:
   ```typescript
   // Each tool has unique version output format
   if (toolId === 'claude-code') {
     // Outputs: claude-code/1.0.86
     const match = output.match(/claude-code\/(\d+\.\d+\.\d+)/);
   } else if (toolId === 'gemini-cli') {
     // Outputs: Gemini CLI v2.0.0
     const match = output.match(/Gemini CLI v(\d+\.\d+\.\d+)/);
   } else if (toolId === 'qwen-code') {
     // Outputs: qwen/1.5.0 or v1.5.0
     const match = output.match(/(?:qwen\/|v?)(\d+\.\d+\.\d+)/);
   }
   ```

4. **Launch Command Patterns**:
   ```typescript
   // Claude Code supports resume
   if (toolId === 'claude-code') {
     command = hasBeenLaunched ? 'claude --resume' : 'claude';
   }
   // Gemini doesn't support resume
   else if (toolId === 'gemini-cli') {
     command = 'gemini';  // Always base command
   }
   // Cline needs special handling
   else if (toolId === 'cline') {
     command = 'cline-cli task';  // Interactive mode
     // Plus API key configuration...
   }
   ```

5. **Memory Service Integration**:
   - All tools can connect to Memory Service via MCP
   - Configuration creates MCP wrapper at `~/.hive/memory-service-mcp-wrapper.js`
   - Token stored in `~/.hive/cli-tools-config.json`

##### Example: Claude Code Deep Dive

From `docs/cli-tools/claude-code.md`, we learn:
- **MCP Servers**: Supports stdio, SSE, and HTTP transports
- **Hooks**: 8 different hook events for workflow customization
- **Authentication**: OAuth via browser OR API key
- **Special Commands**: `/memory`, `/hooks`, `/mcp`
- **Auto-Update**: Has built-in auto-update mechanism
- **Context**: 200K token window
- **Resume**: Supports `--resume` for session continuation

This knowledge informs our implementation:
```typescript
if (toolId === 'claude-code') {
  // Claude-specific implementation based on documentation
  config.supportsResume = true;
  config.hasMCP = true;
  config.hasHooks = true;
  config.authMethods = ['oauth', 'api-key'];
  config.versionCommand = 'claude --version';
  config.versionPattern = /claude-code\/(\d+\.\d+\.\d+)/;
  config.diagnosticCommand = 'claude doctor';
  config.configLocation = '~/.claude/config.json';
}
```

##### Lessons Learned from AI CLI Tools Integration

**1. UI Refresh Bug (Critical Fix)**:
   - **Problem**: After installation, UI showed "Not Installed" until app refresh
   - **Solution**: Clear detector cache after install/update
   ```typescript
   // CRITICAL FIX in src/index.ts
   logger.info(`[Main] Clearing detector cache for ${toolId} after successful install`);
   cliToolsDetector.clearCache(toolId);
   ```

**2. Cline API Key Management (Unique Pattern)**:
   - **Challenge**: Cline is provider-agnostic, users need to configure provider
   - **Solution**: Use Hive's existing OpenRouter API key
   - **Implementation**: 
     - Read key from database: `SELECT value FROM configurations WHERE key = 'openrouter_api_key'`
     - Write to `~/.cline_cli/cline_cli_settings.json`
     - Pass via environment variables on launch
   - **Bug Workaround**: Cline CLI looks for `OPENAI_API_KEY` even when configured for OpenRouter

**3. Terminal Launch Sequencing**:
   - **Issue**: Launching tool before folder context set causes confusion
   - **Fix**: Two-step process with delay
   ```typescript
   // FIRST: Update global folder context
   mainWindow.webContents.send('menu-open-folder', selectedPath);
   
   // THEN: After delay, launch terminal
   setTimeout(() => {
     mainWindow.webContents.send('launch-ai-tool-terminal', {...});
   }, 100);
   ```

**4. Version Detection Variations**:
   - Each tool outputs version differently
   - Generic regex `(\d+\.\d+\.\d+)` works for most
   - Special cases documented per tool

**5. TTYD vs xterm.js Decision**:
   - **xterm.js issues**: % characters, duplicate UI, cursor problems with TUI apps
   - **TTYD benefits**: Real terminals, perfect TUI support, low maintenance
   - **Result**: Complete migration to TTYD for all terminals

#### Implementation Workflow for Each New Tool

##### 📚 Step 0: BECOME AN EXPERT (Required Reading)
```bash
# STOP! Before writing any code:
cat docs/cli-tools/[tool-name].md  # READ ENTIRE DOCUMENT

# Example for Gemini:
cat docs/cli-tools/gemini-cli.md   # Understand Gemini's free tier, auth, limits
```

After reading, you should know:
- ✅ Package name vs command name
- ✅ Version command and output format
- ✅ Authentication requirements
- ✅ Special flags and options
- ✅ Configuration file locations
- ✅ MCP compatibility
- ✅ Unique features and limitations

##### Step 1: Add Tool Definition (ONLY after becoming an expert)
**File**: `src/utils/AI_CLI_TOOLS_REGISTRY.md`

```typescript
// Example: Adding Gemini CLI
{
  id: 'gemini-cli',
  name: 'Gemini CLI',
  packageName: '@google/gemini-cli',
  packageManager: 'npm',
  command: 'gemini',
  versionCommand: 'gemini --version',
  versionPattern: /gemini-cli\/(\d+\.\d+\.\d+)/,
  docsUrl: 'https://cloud.google.com/gemini/docs',
  description: 'Google\'s free AI coding assistant (1000 req/day)',
  memoryServiceCompatible: true,
  resumeSupport: false
}
```

##### Step 1.5: CRITICAL - Import Registry at Module Level
**File**: `src/index.ts` (TOP OF FILE)

```typescript
// ⚠️ CRITICAL: Import at the top of the file, NEVER use runtime require()
import { CLI_TOOLS_REGISTRY } from './shared/types/cli-tools';

// ❌ NEVER DO THIS (breaks after webpack bundling):
// const { CLI_TOOLS_REGISTRY } = require('./shared/types/cli-tools');
```

**Why this is critical:**
- Webpack bundles all modules at build time
- Runtime `require()` with relative paths fails after bundling
- The relative path `'./shared/types/cli-tools'` doesn't exist in webpack output
- This causes "Installation not yet implemented" or similar errors
- **LESSON LEARNED**: Claude Code was pre-installed, so Install button was never tested
- **SOLUTION**: Gemini CLI exposed this issue and led to the fix

### DEFINITIVE TEMPLATE - Use Gemini CLI Pattern

**IMPORTANT**: The Gemini CLI implementation is now our definitive template for all AI CLI tool integrations.

See `docs/cli-tools/GEMINI_TEMPLATE.md` for the complete working pattern that includes:
1. **Module-level imports** - Fixes webpack bundling issues
2. **All four buttons working** - Install, Update, Configure, Launch
3. **Dynamic tool IDs** - No hardcoded references
4. **MCP integration** - Proper wrapper generation
5. **Full testing coverage** - All functionality verified

#### Key Lessons from Gemini CLI Implementation

1. **Always test with fresh installs** - Don't assume pre-installed tools represent the full pattern
2. **Module-level imports are critical** - Runtime requires break after webpack bundling
3. **Use dynamic references everywhere** - Tool IDs should never be hardcoded
4. **Test all buttons before declaring complete** - Each button has unique requirements
5. **Document the working pattern immediately** - Future tools need the complete template
6. **UI must refresh after operations** - Add forceRefresh mechanism to update status
7. **Never leave placeholders** - Every handler must be fully implemented
8. **Test the complete flow** - Install → Configure → Launch → Update cycle
9. **Add progress indicators** - Users need feedback during operations
10. **Verify with actual commands** - Test that tools actually install and run

#### Critical Lessons from Qwen Code Implementation

1. **Package names don't match binary names!** - NPM package `@qwen-code/qwen-code` installs binary as `qwen`, not `qwen-code`
2. **Always verify actual binary after install** - Check `which <command>` to find real binary name
3. **Documentation can be wrong** - Qwen docs show `qwen-code` everywhere but reality is `qwen`
4. **Check package.json bin entry** - The definitive source for actual binary name
5. **Memory Service detection must be explicit** - Add each new tool to the detection list in detector.ts
6. **Version output varies wildly** - Qwen outputs just `0.0.8` with no prefix, unlike other tools
7. **Terminal display names must match binary** - Use actual command as key in displayNames map
8. **Full rebuild often required** - HMR doesn't always work, use `npm run make` for reliability
9. **Test immediately after changes** - Don't accumulate changes without verification
10. **Document quirks immediately** - Create `<TOOL>_QUIRKS.md` for each tool's peculiarities

#### Lessons from OpenAI Codex Implementation

1. **Clean implementation** - Package name matches binary name (`@openai/codex` → `codex`)
2. **Version format** - Outputs `codex-cli 0.23.0` with "codex-cli" prefix
3. **Memory Service bug confirmed** - After Configure, Details shows "Not connected" until Launch
4. **UI refresh bug persistent** - 4th tool with same issue, 100% reproducible
5. **All features work** - Install, Configure, Launch, Update all functional
6. **GPT-5 and o-series models** - Advanced model support documented
7. **Multimodal capabilities** - Supports images and diagrams as input
8. **ChatGPT integration** - Can use ChatGPT Plus subscription instead of API key

#### Memory Service Display Bug (Affects All Tools)

**Bug Description:**
After clicking "Configure" button and successfully connecting to Memory Service:
- Configuration file is updated correctly
- Connection is established
- BUT "Details" button shows "Memory: Not connected"
- Only after "Launch Terminal" does Details show "Memory: Connected ✓"

**Root Cause:**
The detector cache isn't cleared after configuration, so it returns stale data.

**Workaround:**
Launch the terminal once after configuration to force cache refresh.

**Potential Fix:**
```typescript
// In configureCliTool function, after successful configuration:
cliToolsDetector.clearCache(toolId);
await renderCliToolsPanel(true);
```

### COMPLETE AI CLI Tool Integration Pattern (Post-Gemini)

#### Critical Missing Steps We Discovered

##### 1. UI Refresh Mechanism (CRITICAL - Was Missing!)
**File**: `src/renderer.ts`

**⚠️ CRITICAL BUG THAT HAPPENS WITH EVERY SINGLE TOOL (Claude Code, Gemini CLI, Qwen Code, OpenAI Codex):**
The UI will show "installed successfully" in console but won't update visually even WITH forceRefresh! This is a CONSISTENT issue that occurs EVERY TIME!

**Symptoms (happens 100% of the time):**
- Console: "[INFO] [CLI Tools] <tool> installed successfully"
- Console: "[INFO] [CLI Tools] Rendering CLI Tools panel..."
- Console: "[INFO] [CLI Tools] Panel rendered successfully"
- UI: Still shows "Install" button - NOT updated!

**Root Cause:**
The detection runs immediately after install but the tool isn't in PATH yet, or there's a cache issue with the detector.

**REQUIRED WORKAROUND:**
User MUST restart the app after installation. This is NOT optional - it happens EVERY time!

**Potential Fixes to Implement:**
1. Add a delay before detection: `setTimeout(() => detectTool(toolId), 3000)`
2. Clear detector cache: `cliToolsDetector.clearCache(toolId)` before re-detecting
3. Force PATH refresh in detector before checking
4. Show a "Restart Required" message after successful install
5. Implement a manual "Refresh Status" button that works

```typescript
// Add forceRefresh parameter to renderCliToolsPanel
async function renderCliToolsPanel(forceRefresh: boolean = false) {
    const container = document.getElementById('cli-tools-container');
    if (container && (container.innerHTML.trim() === '' || forceRefresh)) {
        // Render panel
    }
}

// In install handler - MUST force refresh
if (result.success) {
    console.log(`[CLI Tools] ${toolId} installed successfully`);
    await renderCliToolsPanel(true); // Force refresh!
}

// In update handler - MUST force refresh
if (result.success) {
    setTimeout(async () => {
        await renderCliToolsPanel(true); // Force refresh!
    }, 1000);
}
```

**TROUBLESHOOTING UI NOT UPDATING:**
1. Check console - if it says "installed successfully" but UI doesn't change
2. Verify forceRefresh parameter is passed as `true`
3. Kill and restart the app (`npm start`) - hot reload doesn't always work
4. If still not working, do full rebuild: `npm run make` then `npm start`

##### 2. Complete Handler Implementation (No Placeholders!)
**File**: `src/index.ts`

```typescript
ipcMain.handle('cli-tool-install', async (_, toolId: string) => {
    console.log(`[Main] Installing CLI tool: ${toolId}`);
    
    // CRITICAL: Use imported registry, NOT runtime require!
    const toolConfig = CLI_TOOLS_REGISTRY[toolId];
    if (!toolConfig) {
        return { success: false, error: `Unknown tool: ${toolId}` };
    }
    
    if (!toolConfig.installCommand) {
        return { success: false, error: `Installation not available` };
    }
    
    try {
        // Show progress to user
        const { stdout, stderr } = await execAsync(toolConfig.installCommand, {
            env: { ...process.env, PATH: enhancedPath }
        });
        
        // Verify installation worked
        const versionResult = await execAsync(toolConfig.versionCommand, {
            env: { ...process.env, PATH: enhancedPath }
        });
        
        // Tool-specific version extraction
        let version = 'Unknown';
        if (toolId === 'gemini-cli') {
            const match = versionResult.stdout.match(/(?:gemini-cli\/|v?)(\d+\.\d+\.\d+)/);
            version = match ? match[1] : 'Unknown';
        }
        
        return { 
            success: true, 
            version,
            message: `${toolConfig.name} installed successfully`
        };
    } catch (error) {
        console.error(`[Main] Install error for ${toolId}:`, error);
        return { success: false, error: error.message };
    }
});
```

##### 3. Terminal Display Names (Was Missing!)
**File**: `src/terminal-ipc-handlers.ts`

```typescript
const TOOL_DISPLAY_NAMES: Record<string, string> = {
    'claude-code': 'Claude',
    'gemini-cli': 'Gemini',  // MUST add for each tool
    'qwen-code': 'Qwen',
    'aider': 'Aider',
    'cline': 'Cline'
};
```

##### 4. Progress Indicators (Essential UX)
**File**: `src/renderer.ts`

```typescript
async function installCliTool(toolId: string): Promise<void> {
    const card = document.querySelector(`[data-tool-id="${toolId}"]`);
    if (card) {
        const statusDiv = card.querySelector('.tool-status');
        if (statusDiv) {
            statusDiv.innerHTML = '⏳ Installing...'; // Show progress!
            statusDiv.style.color = '#FFA500';
        }
    }
    // ... rest of installation
}
```

##### 5. Testing Checklist (MUST DO ALL)
```bash
# 1. Uninstall tool first to test fresh install
npm uninstall -g @google/gemini-cli

# 2. Test Install button
# - Click Install
# - Verify "Installing..." appears
# - Verify panel refreshes after success
# - Verify status shows "Installed"

# 3. Test Configure button
# - Click Configure
# - Verify MCP wrapper created
# - Check ~/.hive/mcp/ for wrapper script

# 4. Test Launch button
# - Click Launch
# - Verify terminal opens
# - Verify tool name in terminal tab
# - Verify tool actually runs

# 5. Test Update button
# - Click Update
# - Verify "Updating..." appears
# - Verify panel refreshes
# - Verify version updates
```

### Gemini CLI - Complete Implementation Details

#### What Makes Gemini Special
- **FREE Tier**: 1000 requests/day at no cost
- **1M Token Context**: Massive context window
- **MCP Support**: Full Memory Service integration
- **Google Integration**: Works with Google Cloud
- **No API Key Required**: Works out of the box

#### Gemini-Specific Configuration
**File**: `src/shared/types/cli-tools.ts`

```typescript
'gemini-cli': {
    id: 'gemini-cli',
    name: 'Gemini CLI',
    description: 'Google\'s free AI coding assistant with 1M token context',
    command: 'gemini',
    installCommand: 'npm install -g @google/gemini-cli',
    updateCommand: 'npm update -g @google/gemini-cli',
    versionCommand: 'gemini --version',
    versionRegex: /(?:gemini-cli\/|v?)(\d+\.\d+\.\d+)/,
    docsUrl: 'https://cloud.google.com/gemini/docs/codeassist/gemini-cli',
    requiresNode: true
}
```

#### Gemini Version Detection Pattern
```typescript
// Gemini outputs: "gemini-cli/1.2.3" or "v1.2.3"
if (toolId === 'gemini-cli') {
    const match = versionResult.stdout.match(/(?:gemini-cli\/|v?)(\d+\.\d+\.\d+)/);
    version = match ? match[1] : 'Unknown';
}
```

#### Gemini Launch Command
```typescript
if (toolId === 'gemini-cli') {
    // Gemini uses --chat flag for interactive mode
    launchCommand = 'gemini --chat';
}
```

#### Gemini UI Card with FREE Badge
```typescript
const geminiStatus = await electronAPI.detectCliTool('gemini-cli');
gridContainer.appendChild(createCliToolCard({
    id: 'gemini-cli',
    name: 'Gemini CLI',
    description: 'Google\'s free AI coding assistant with 1M token context',
    status: geminiStatus,
    docsUrl: 'https://cloud.google.com/gemini/docs/codeassist/gemini-cli',
    badgeText: 'FREE',  // Highlight free tier
    badgeColor: '#28a745'  // Green for free
}));
```

### Systematic Approach for Remaining AI CLI Tools

#### Pre-Implementation Checklist
- [ ] Uninstall the tool if already installed (test fresh install)
- [ ] Read the tool's documentation in `docs/cli-tools/`
- [ ] Note the package manager (npm vs pip)
- [ ] **CRITICAL: Verify actual binary name after manual install:**
  ```bash
  # For NPM packages:
  npm install -g <package-name>
  npm list -g --depth=0  # Shows installed package
  cat $(npm root -g)/<package-name>/package.json | grep -A2 '"bin"'  # Shows binary mapping
  which <expected-command>  # Verify actual command
  
  # Example with Qwen Code (documentation vs reality):
  # Docs say: qwen-code --version
  # Reality after install:
  npm install -g @qwen-code/qwen-code
  cat $(npm root -g)/@qwen-code/qwen-code/package.json | grep -A2 '"bin"'
  # Shows: "bin": { "qwen": "dist/index.js" }  ← Binary is 'qwen', not 'qwen-code'!
  which qwen  # ✓ Found
  which qwen-code  # ✗ Not found
  ```
- [ ] Test version command with ACTUAL binary name
- [ ] Identify version output pattern (may be just numbers like "0.0.8")
- [ ] Check for special launch flags
- [ ] Document quirks in `docs/cli-tools/<TOOL>_QUIRKS.md`

#### Implementation Order (NEVER SKIP ANY!)
1. **Module Import** - Add to top of `src/index.ts`
2. **Tool Configuration** - Add to `CLI_TOOLS_REGISTRY`
3. **Install Handler** - Complete implementation, no placeholders
4. **Update Handler** - Tool-specific version checking
5. **Configure Handler** - MCP wrapper with dynamic tool ID
6. **Launch Handler** - Tool-specific command flags
7. **Terminal Names** - Add to `TOOL_DISPLAY_NAMES`
8. **UI Card** - Dynamic detection with badges
9. **UI Refresh** - Ensure forceRefresh works
10. **Test Everything** - All 4 buttons must work

#### Testing Protocol (MANDATORY)
```bash
# For each new tool:
1. npm uninstall -g [tool-package]  # Start fresh
2. npm start                         # Run app
3. Click Install → Verify completes and refreshes
4. Click Configure → Verify MCP wrapper created
5. Click Launch → Verify terminal opens with tool
6. Click Update → Verify checks for updates
7. Run test script: node test-[tool-name].js
```

#### Common Mistakes to Avoid
- ❌ Assuming pre-installed tools work like fresh installs
- ❌ Using runtime require() instead of module imports
- ❌ Forgetting to add forceRefresh to UI
- ❌ Leaving \"not yet implemented\" placeholders
- ❌ Hardcoding tool IDs anywhere
- ❌ Not testing all 4 buttons
- ❌ Skipping terminal display names
- ❌ Missing progress indicators

#### Next Tools to Implement (Use Gemini Template!)
1. **Qwen Code** - Use pip, open source
2. **Aider** - Git integration, pip install
3. **Cline** - Lightweight, npm package
4. **OpenAI Codex** - Requires API key setup

**REMEMBER**: Follow `docs/cli-tools/GEMINI_TEMPLATE.md` exactly!

##### Step 2: Update Package Mappings
**File**: `src/index.ts` (in install/update handlers)

```typescript
// Use the imported CLI_TOOLS_REGISTRY directly
const toolConfig = CLI_TOOLS_REGISTRY[toolId];
if (!toolConfig) {
  return { success: false, error: `Unknown tool: ${toolId}` };
}

// For install handler
const installCommand = toolConfig.installCommand;
// For update handler  
const updateCommand = toolConfig.updateCommand;
```

##### Step 3: Add Version Detection Logic
**File**: `src/index.ts` (update handler ~1470)

```typescript
if (toolId === 'gemini-cli') {
  const versionResult = await execAsync('gemini --version');
  version = versionResult.stdout.match(/gemini-cli\/(\d+\.\d+\.\d+)/)?.[1];
}
```

##### Step 4: Create Documentation
**File**: `docs/cli-tools/gemini-cli.md`

```markdown
# Gemini CLI Documentation

## Overview
[Tool description]

## Installation
npm install -g @google/gemini-cli

## Features
[List features]

## Memory Service Integration
[MCP compatibility details]
```

##### Step 5: Add UI Card
**File**: `src/renderer.ts` (renderCliToolsPanel function)

```typescript
gridContainer.appendChild(createCliToolCard({
  id: 'gemini-cli',
  name: 'Gemini CLI',
  description: 'Google\'s free AI coding assistant',
  status: await electronAPI.detectCliTool('gemini-cli'),
  docsUrl: 'https://cloud.google.com/gemini/docs',
  badgeText: 'FREE',
  badgeColor: '#28a745'
}));
```

##### Step 6: Implement Tool-Specific Logic
If the tool has unique requirements:

```typescript
// Special handling for tool-specific features
if (toolId === 'gemini-cli') {
  // Example: Gemini might need API key setup
  await setupGeminiApiKey();
  
  // Example: Special authentication flow
  await authenticateWithGoogle();
}
```

#### Testing Checklist for New Tools

- [ ] **Detection**: Tool correctly detected when installed
- [ ] **Version**: Version extraction works correctly
- [ ] **Installation**: Install button successfully installs tool
- [ ] **Configuration**: Memory Service connection established
- [ ] **Launch**: Tool launches in terminal with correct command
- [ ] **Update**: Update button fetches and installs latest version
- [ ] **Cleanup**: Tool processes cleaned up on app quit
- [ ] **Error Handling**: All error cases show helpful messages
- [ ] **UI State**: All visual states work correctly
- [ ] **Documentation**: Tool documented in docs/cli-tools/

#### Common Pitfalls to Avoid

1. **Version Pattern Mismatch**
   - Test version regex with actual output
   - Handle edge cases (beta versions, etc.)

2. **PATH Issues**
   - Always include common binary paths
   - Test on fresh system without tools in PATH

3. **Permission Errors**
   - Provide clear manual fallback commands
   - Never attempt automatic sudo escalation

4. **Memory Service Compatibility**
   - Not all tools support MCP
   - Gracefully handle non-compatible tools

5. **Terminal Command Variations**
   - Some tools use different commands than package name
   - Example: Package `@anthropic-ai/claude-code` → Command `claude`

#### Tool-Specific Considerations

##### NPM-based Tools
- Use `npm update -g` for updates
- Check global npm prefix for installations
- Handle npm permission issues gracefully

##### Python/pip-based Tools
- Use `pip install --upgrade` for updates
- Check both pip and pip3
- Handle virtual environment considerations

##### Binary Tools
- May need custom installation logic
- Version detection might be non-standard
- Update mechanism varies by tool

#### Monitoring & Analytics

Track usage for each tool:
```sql
CREATE TABLE tool_metrics (
  tool_id TEXT,
  action TEXT, -- 'install', 'launch', 'update', 'configure'
  timestamp TEXT,
  success BOOLEAN,
  error_message TEXT,
  duration_ms INTEGER
);
```

#### Future Enhancements Path

1. **Auto-Discovery**: Scan system for installed AI tools
2. **Plugin System**: Allow community tool additions
3. **Tool Marketplace**: Browse and install from catalog
4. **Sync Settings**: Share tool configs across devices
5. **Team Management**: Centralized tool deployment

---

## CLI Tools Management

### Overview
The CLI Tools Management system provides automated installation, updates, and integration for AI-powered CLI tools, with seamless Memory Service integration via MCP (Model Context Protocol). This system enables one-click installation, configuration, and updates for AI coding assistants, making them feel "out of the box" integrated without user configuration.

### Architecture
**Location**: `src/utils/CliToolsManager.ts`
**Purpose**: Manage lifecycle of external AI CLI tools with full Memory Service integration
**Integration**: Direct connection to Memory Service via REST API and MCP protocol
**Detection**: `src/utils/cli-tool-detector.ts` - Real-time tool detection and version checking
**Launch Tracking**: `src/services/AIToolsDatabase.ts` - Track tool launches per repository for intelligent resume

### Components

#### CliToolsManager Class
```typescript
class CliToolsManager extends EventEmitter {
  // Tool registry and status tracking
  private tools: Map<string, CliToolConfig>
  private status: Map<string, ToolStatus>
  
  // Lifecycle methods
  public async install(toolId: string): Promise<void>
  public async checkForUpdates(toolId: string): Promise<boolean>
  public async update(toolId: string): Promise<void>
  
  // Memory Service integration
  private async configureMemoryServiceIntegration(toolId: string)
}
```

### Supported Tools
1. **Claude Code CLI** (`@anthropic/claude-cli`)
   - Primary integration with Memory Service via MCP
   - Auto-configuration of memory endpoints
   - Token-based authentication
   - MCP config at `~/.claude/.mcp.json`

2. **Grok CLI** (`@vibe-kit/grok-cli`)
   - Full MCP support for Memory Service integration
   - Custom setup wizard for API key configuration
   - MCP config at `~/.grok/mcp-config.json`
   - User settings at `~/.grok/user-settings.json`
   - Supports Morph Fast Apply (4,500+ tokens/sec)

3. **Gemini CLI** (`@caretdev/gemini-cli`)
   - Memory Service integration via tokens
   - Auto-configuration support

4. **Qwen Code** (`@qwen-code/qwen-code`)
   - Memory Service integration
   - Token-based authentication

5. **OpenAI Codex** (`openai-codex`)
   - Python-based installation
   - API key configuration

6. **Cline** (`cline`)
   - OpenRouter API key configuration
   - Memory Service integration

### Installation Flow
```
1. Check tool prerequisites
2. Verify system dependencies
3. Execute installation command (npm/pip/gh)
4. Verify installation success
5. **Automatic Configuration Phase**:
   a. Register with Memory Service (generates unique token)
   b. Create MCP wrapper script at ~/.hive/memory-service-mcp-wrapper.js
   c. Update tool's MCP configuration:
      - Claude Code: ~/.claude/.mcp.json
      - Grok: ~/.grok/mcp-config.json
   d. For Cline: Set OpenRouter API key in ~/.cline/config.json
   e. For Grok: Launch interactive setup wizard if API key missing
6. Clear cache to trigger UI refresh
7. Save status to database and config
```

### Grok-Specific Setup Wizard

**Location**: `src/terminal-ipc-handlers.ts`

When Grok is launched without an API key, a custom interactive setup wizard is triggered:

1. **Wizard Script Generation**:
   - Creates temporary bash script in `/tmp`
   - Guides user through API key setup
   - Shows clear steps to obtain key from https://console.x.ai/team/default/api-keys
   - Displays API key during entry for verification (not hidden)
   - Shows partial key mask after entry for confirmation

2. **Terminal Launch**:
   ```typescript
   if (options.command === 'grok:setup') {
     // Create interactive setup script
     const scriptContent = `#!/bin/bash
     echo "🚀 Grok CLI Setup Wizard"
     echo "Please enter your API key (visible for verification):"
     read api_key
     # Save to user settings
     echo "{\"apiKey\": \"\$api_key\", ...}" > ~/.grok/user-settings.json
     `;
     // Launch in TTYD terminal for interactive experience
   }
   ```

3. **Post-Setup Configuration**:
   - Creates `~/.grok/user-settings.json` with API key
   - Generates `~/.grok/mcp-config.json` for Memory Service
   - Updates with dynamic port on each app startup

### Uninstall Flow
```
1. Show confirmation dialog to user
2. Map tool ID to package name
3. Execute uninstall command (npm uninstall -g / pip uninstall)
4. Clean up tool-specific configurations:
   a. Remove Cline config file (~/.cline/config.json)
   b. Preserve Grok API keys for potential reinstall
   c. Keep Memory Service registration for reuse
5. Clear tool from detection cache
6. Verify tool is no longer accessible
7. Update UI to show uninstalled state
```

### Memory Service Integration

#### MCP (Model Context Protocol) Integration
For Claude Code and compatible tools:

1. **Registration Flow**:
   - Tool registers with Memory Service API (`POST /api/v1/memory/register`)
   - Memory Service generates unique 64-byte token using crypto.randomBytes(32).toString('hex')
   - Token stored in `~/.hive/cli-tools-config.json`
   - Token immediately usable for authenticated API calls

2. **MCP Configuration**:
   - Automatically updates tool-specific MCP configs:
     - Claude Code: `~/.claude/.mcp.json`
     - Grok: `~/.grok/mcp-config.json`
   - Creates shared MCP wrapper script at `~/.hive/memory-service-mcp-wrapper.js`
   - Wrapper script uses `@modelcontextprotocol/sdk` for MCP server
   - Exposes two MCP tools:
     - `query_memory`: Search AI memory system for relevant learnings
     - `contribute_learning`: Add new learnings with type, category, content

3. **Authentication System**:
   - Bearer token authentication for all API calls
   - Per-tool tokens for isolation and security (each tool gets unique token)
   - Tokens persist across sessions in cli-tools-config.json
   - Memory Service tracks connected tools in-memory with Map<token, ToolInfo>
   - Authentication middleware validates token on each request:
     ```typescript
     const token = req.headers.authorization?.replace('Bearer ', '');
     if (!this.connectedTools.has(token)) {
       // Register new tool connection
       this.connectedTools.set(token, { id, name, connectedAt, ... });
     }
     ```

4. **API Endpoints**:
   ```
   POST /api/v1/memory/query      - Query memories with context
   POST /api/v1/memory/contribute - Contribute new learnings
   GET  /api/v1/memory/stats      - Get memory statistics
   GET  /api/v1/memory/tools      - List connected tools
   ```

#### Dynamic Port Handling for MCP

**Problem**: Memory Service uses dynamic port allocation (3457-3560) but MCP configs need the actual port.

**Solution**: Automatic MCP configuration updates on every app startup.

1. **Port Discovery Flow**:
   ```typescript
   // When Memory Service starts
   processManager.on('process:message', (name, msg) => {
     if (name === 'memory-service' && msg.type === 'ready') {
       memoryServicePort = msg.port;  // Actual allocated port
       updateAllMCPConfigurations(memoryServicePort);
     }
   });
   ```

2. **MCP Configuration Update Process** (`updateAllMCPConfigurations`):
   - Reads current tool tokens from `~/.hive/cli-tools-config.json`
   - Updates MCP wrapper fallback endpoint with actual port
   - Updates Claude Code MCP config with dynamic endpoint
   - Updates Grok MCP config with dynamic endpoint
   - Maintains existing authentication tokens

3. **Files Updated on Each Startup**:
   ```
   ~/.hive/memory-service-mcp-wrapper.js  - Fallback endpoint updated
   ~/.claude/.mcp.json                    - Environment variables updated
   ~/.grok/mcp-config.json               - Environment variables updated
   ```

4. **Grok-Specific MCP Configuration**:
   
   **IMPORTANT**: Grok is unique among our AI CLI tools - it uses its own MCP configuration file at `~/.grok/mcp-config.json` rather than reading from the shared `~/.hive/cli-tools-config.json`.
   
   ```json
   // ~/.grok/mcp-config.json
   {
     "servers": {
       "hive-memory-service": {
         "transport": "stdio",
         "command": "node",
         "args": ["/Users/.../.hive/memory-service-mcp-wrapper.js"],
         "env": {
           "MEMORY_SERVICE_ENDPOINT": "http://localhost:<dynamic-port>",
           "MEMORY_SERVICE_TOKEN": "<unique-token>"
         }
       }
     }
   }
   ```
   
   **Key Differences for Grok**:
   - Configuration location: `~/.grok/mcp-config.json` (not `~/.hive/cli-tools-config.json`)
   - Auto-creation: File is created on app startup if token exists in cli-tools-config.json
   - Detection: Memory service connection check must look in Grok's unique location
   - Token storage: Initially saved to cli-tools-config.json, then propagated to MCP config

5. **Port Allocation Strategy**:
   - Preferred port: 3457
   - Fallback range: 3458-3560
   - Never hardcoded in configurations
   - Always discovered at runtime

#### Implementation Details & Lessons Learned

**Critical Discovery**: Grok requires special handling throughout the system:

1. **Configuration Dual-Storage Pattern**:
   - Token initially stored in `~/.hive/cli-tools-config.json` (like other tools)
   - MCP config must be at `~/.grok/mcp-config.json` (Grok-specific)
   - Synchronization happens on every app startup

2. **Auto-Creation vs Update-Only**:
   ```typescript
   // In updateAllMCPConfigurations (src/index.ts)
   // Claude Code: Only updates if file exists
   if (fs.existsSync(claudeMcpPath)) { /* update */ }
   
   // Grok: Creates file if missing (critical difference!)
   if (grokToken) {
     // Always create/update, don't check existence first
     fs.writeFileSync(grokMcpPath, JSON.stringify(grokMcp, null, 2));
   }
   ```

3. **Detection Fallback Pattern**:
   ```typescript
   // In checkMemoryServiceConnection (src/main/cli-tools/detector.ts)
   if (toolId === 'grok') {
     // Primary: Check ~/.grok/mcp-config.json
     // Fallback: Check ~/.hive/cli-tools-config.json
     // This dual-check ensures detection works during transition
   }
   ```

4. **Tool Registry Update Required**:
   - Must add 'grok' to memory service check list in detector
   - Easy to miss because other tools were hardcoded in condition

**Key Architectural Decisions**:
- ✅ Keep token in cli-tools-config.json for centralized management
- ✅ Auto-create Grok's MCP config on startup for seamless experience
- ✅ Support both config locations in detector for robustness
- ✅ Log all MCP operations for debugging
- ❌ Don't assume all tools use same config pattern

### Database Integration
Uses existing `sync_metadata` table:
```sql
-- Tool update tracking
sync_type: 'claude_cli_update' | 'gh_copilot_cli_update' | etc.
intelligence_version: installed tool version
next_sync_due: next update check time
```

### IPC Handlers
```typescript
// Main process handlers (index.ts)
'cli-tool-detect': Detect if a specific tool is installed
'cli-tool-install': Install a specific tool with automatic configuration
'cli-tool-update': Update a specific tool to latest version
'cli-tool-configure': Configure Memory Service integration (deprecated - now automatic)
'cli-tool-uninstall': Uninstall a specific tool with cleanup
'cli-tools-detect-all': Detect all supported tools
'cli-tools-check-updates': Check for updates across all tools
'cli-tool-launch': Launch tool with smart configuration detection
```

### UI Implementation

#### Batch Operations (Top of Panel)
1. **Install All Tools Button** (Blue):
   - Installs all 6 AI CLI tools in sequence
   - Skips already installed tools automatically
   - Shows progress counter (e.g., "Installing 3 of 6...")
   - Automatically configures Memory Service for compatible tools
   - Configures Cline with OpenRouter API key from settings
   - Displays summary (e.g., "✅ Installed 4, skipped 2")

2. **Update All Tools Button** (Gray):
   - Updates ONLY installed tools to latest versions
   - Skips tools that are not installed
   - Shows progress during batch update
   - Displays update summary with version changes

#### Individual Tool Card Actions
1. **Details Button** (Green):
   - Refreshes tool status
   - Shows version, path, and Memory status
   - Restores full detail view after other actions

2. **Configure Button** (REMOVED - Now Automatic):
   - **DEPRECATED**: Configuration now happens automatically during installation
   - Memory Service registration occurs seamlessly after install
   - MCP configuration updates without user intervention
   - Cline receives OpenRouter API key from Hive settings automatically
   - Grok launches interactive setup wizard on first run if unconfigured

3. **Update Button** (Gray):
   - **Purpose**: Updates CLI tools to their latest versions
   - **Visual States**:
     - Default: Gray button with "Update" text
     - Updating: Shows "⬆️ Updating..." with orange status
     - Success: Displays "✅ Up to date" with green status
     - Error: Shows "❌ Update failed" with red status
   - **Implementation**: See [Update Button Architecture](#update-button-architecture) below

4. **Install Button** (Blue - for uninstalled tools):
   - Runs appropriate package manager (npm/pip)
   - Shows progress indicators
   - **Automatically configures Memory Service** after successful installation
   - **For Cline**: Sets OpenRouter API key from Hive settings
   - **For Grok**: Detects missing API key and launches setup wizard
   - Refreshes panel on completion with configuration status

5. **Uninstall Button** (Red - for installed tools):
   - Shows confirmation dialog before proceeding
   - Runs `npm uninstall -g <package>` or `pip uninstall <package>`
   - Clears tool from cache after uninstall
   - **Preserves user configurations** (e.g., Grok API keys)
   - Removes Cline config file if present
   - Verifies tool was successfully removed
   - Updates UI to show uninstalled state

### Configuration Storage
```
~/.hive/
├── cli-tools-config.json  # Tool status and versions
└── tools/                  # Tool installation directory
    ├── node_modules/       # Local npm installations
    └── ...
```

### Auto-Update System
- 24-hour update check interval
- Background checking on app startup
- Event emissions for update availability
- Non-blocking update downloads
- Batch update capability for all installed tools

### Smart Configuration System

#### Automatic Memory Service Integration
After any tool installation:
1. Tool is automatically registered with Memory Service
2. Unique authentication token generated (64-byte hex)
3. MCP wrapper script created/updated
4. Tool's MCP configuration updated
5. No user intervention required

#### Tool-Specific Configurations

**Cline OpenRouter Integration**:
- Automatically receives API key from Hive settings
- Creates `~/.cline/config.json` with OpenRouter credentials
- Enables access to 400+ AI models immediately

**Grok Setup Wizard**:
- Detects missing API key on launch
- Provides interactive terminal wizard
- Guides through X.AI account setup
- Saves credentials securely
- Falls back to manual configuration if needed

##### Grok Setup Wizard Implementation
**Location**: `src/terminal-ipc-handlers.ts` (lines 95-209)

The Grok setup wizard provides a seamless first-time configuration experience:

```typescript
// Detection logic in main process
if (!fs.existsSync(grokConfigPath) || !grokConfig.apiKey) {
  // Launch setup wizard instead of regular Grok
  command = 'grok:setup';
}

// Wizard script generation
const scriptContent = `#!/bin/bash
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "                 🚀 Grok CLI Setup Wizard"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Welcome to Grok CLI! Let's get you set up."
echo ""
echo "To use Grok, you need an API key from X.AI"
echo ""
echo "📝 Steps to get your API key:"
echo "   1. Visit https://console.x.ai/team/default/api-keys"
echo "   2. Sign in or create an account"
echo "   3. Click 'Create API key'"
echo "   4. Copy your new API key"
echo ""
read -p "Would you like to set up your API key now? (y/n): " response

if [[ "$response" =~ ^[Yy]$ ]]; then
  echo "Please enter your Grok API key:"
  read -s api_key
  
  # Use Python to safely merge with existing config
  python3 -c "
import json
import sys

try:
    with open('$HOME/.grok/user-settings.json', 'r') as f:
        settings = json.load(f)
except:
    settings = {}

settings['apiKey'] = '$api_key'
if 'baseURL' not in settings:
    settings['baseURL'] = 'https://api.x.ai/v1'
if 'defaultModel' not in settings:
    settings['defaultModel'] = 'grok-4-latest'

with open('$HOME/.grok/user-settings.json', 'w') as f:
    json.dump(settings, f, indent=2)
print('✅ API key added to existing configuration')
"
  
  echo ""
  echo "🎉 Setup complete! Launching Grok CLI..."
  sleep 2
  exec grok
fi
`;

// Write to temporary file and execute
const scriptPath = path.join(os.tmpdir(), `grok-setup-${Date.now()}.sh`);
fs.writeFileSync(scriptPath, scriptContent);
fs.chmodSync(scriptPath, '755');

// Launch in terminal with cleanup
actualCommand = `bash ${scriptPath}; rm -f ${scriptPath}`;
```

**Features**:
1. **Smart Detection**: Checks for existing `~/.grok/user-settings.json` and API key
2. **Interactive Flow**: User-friendly prompts with clear instructions
3. **Secure Input**: Uses `read -s` for password-style API key entry
4. **Config Preservation**: Merges new API key with existing settings using Python JSON
5. **Automatic Launch**: Starts Grok CLI after successful configuration
6. **Fallback Options**: Provides manual configuration instructions if user declines

### Supported Agentic Coding CLIs
Reference: `src/utils/AI_CLI_TOOLS_REGISTRY.md`

The system supports 6 carefully selected agentic coding CLIs that provide autonomous coding capabilities:

1. **Claude Code CLI** - Anthropic's terminal-native agent
   - Documentation: [`docs/cli-tools/claude-code.md`](docs/cli-tools/claude-code.md)
   - NPM: `@anthropic-ai/claude-code`
   - Version: 1.0.85+

2. **Gemini CLI** - Google's free-tier agentic assistant (1000 requests/day)
   - Documentation: [`docs/cli-tools/gemini-cli.md`](docs/cli-tools/gemini-cli.md)
   - NPM: `@google/gemini-cli`
   - Version: 0.1.18+

3. **Qwen Code** - Alibaba's open-source agent
   - Documentation: [`docs/cli-tools/qwen-code.md`](docs/cli-tools/qwen-code.md)
   - NPM: `@qwen-code/qwen-code`
   - Version: 0.0.7+

4. **OpenAI Codex CLI** - OpenAI's smart terminal assistant
   - Documentation: [`docs/cli-tools/openai-codex.md`](docs/cli-tools/openai-codex.md)
   - NPM: `@openai/codex`
   - Version: 0.22.0+

5. **Aider** - Git-integrated agentic editor
   - Documentation: [`docs/cli-tools/aider.md`](docs/cli-tools/aider.md)
   - PyPI: `aider-chat`
   - Installation: `pip install aider-chat`

6. **Cline** - Lightweight conversational agent
   - Documentation: [`docs/cli-tools/cline.md`](docs/cli-tools/cline.md)
   - NPM: `@yaegaki/cline-cli`
   - Version: 0.1.1+
   - **Auto-configuration**: Receives OpenRouter API key from Hive settings

7. **Grok CLI** - xAI's powerful terminal agent
   - Documentation: [`docs/grok-cli-documentation.md`](docs/grok-cli-documentation.md)
   - NPM: `@vibe-kit/grok-cli`
   - Version: 0.0.23+
   - **Smart Setup**: Interactive wizard for first-time API key configuration
   - **MCP Support**: Advanced Model Context Protocol integration

### Update Button Architecture

#### Overview
The Update Button provides seamless, one-click updates for installed CLI tools directly from the Hive Consensus UI. It handles both NPM and pip-based tools with comprehensive error handling and real-time status feedback.

#### Implementation Architecture

##### 1. Frontend (Renderer Process)
**Location**: `src/renderer.ts`

```typescript
async function updateCliTool(toolId: string): Promise<void> {
  // 1. Show updating status in UI
  const card = document.querySelector(`[data-tool-id="${toolId}"]`);
  card.querySelector('.tool-status').innerHTML = '⬆️ Updating...';
  
  // 2. Call IPC handler
  const result = await electronAPI.updateCliTool(toolId);
  
  // 3. Update UI based on result
  if (result.success) {
    statusDiv.innerHTML = '✅ Up to date';
    // Update version display without full refresh
    versionSpan.textContent = result.version;
  } else {
    statusDiv.innerHTML = '❌ Update failed';
  }
}
```

##### 2. IPC Bridge
**Location**: `src/preload.ts`

```typescript
updateCliTool: (toolId: string) => ipcRenderer.invoke('cli-tool-update', toolId)
```

##### 3. Main Process Handler
**Location**: `src/index.ts` (lines 1396-1518)

```typescript
ipcMain.handle('cli-tool-update', async (_, toolId: string) => {
  // Package mapping
  const npmPackages = {
    'claude-code': '@anthropic-ai/claude-code',
    'gemini-cli': '@google/gemini-cli',
    'qwen-code': '@qwen-code/qwen-code'
  };
  
  // Execute update command
  const updateCommand = `npm update -g ${packageName}`;
  const { stdout, stderr } = await execAsync(updateCommand, {
    env: { ...process.env, PATH: enhancedPath }
  });
  
  // Get updated version
  if (toolId === 'claude-code') {
    const versionResult = await execAsync('claude --version');
    version = versionResult.stdout.match(/claude-code\/(\d+\.\d+\.\d+)/)?.[1];
  }
  
  return { success: true, version, message };
});
```

#### Update Flow Sequence

```mermaid
sequenceDiagram
    participant User
    participant UI as Renderer (UI)
    participant IPC as IPC Bridge
    participant Main as Main Process
    participant NPM as Package Manager
    participant Tool as CLI Tool

    User->>UI: Click Update Button
    UI->>UI: Show "⬆️ Updating..."
    UI->>IPC: updateCliTool('claude-code')
    IPC->>Main: invoke('cli-tool-update')
    Main->>Main: Map toolId to package
    Main->>NPM: npm update -g @anthropic-ai/claude-code
    NPM->>NPM: Download latest version
    NPM->>Main: Update complete
    Main->>Tool: claude --version
    Tool->>Main: claude-code/1.0.87
    Main->>IPC: {success: true, version: "1.0.87"}
    IPC->>UI: Update result
    UI->>UI: Show "✅ Up to date"
    UI->>User: Display new version
```

#### Package Management Strategy

##### NPM-based Tools
- **Claude Code**: `@anthropic-ai/claude-code`
- **Gemini CLI**: `@google/gemini-cli`
- **Qwen Code**: `@qwen-code/qwen-code` (command: `qwen`)
- **OpenAI Codex**: `@openai/codex`
- **Cline**: `@yaegaki/cline-cli`
- **Grok CLI**: `@vibe-kit/grok-cli`

**Install Command**: `npm install -g <package-name>@latest`
**Update Command**: `npm update -g <package-name>`
**Uninstall Command**: `npm uninstall -g <package-name>`

##### Python/pip-based Tools
- **Aider**: `aider-chat`

**Update Command**: `pip install --upgrade aider-chat`

#### Version Detection Methods

##### 1. Claude Code (Special Case)
```typescript
// Uses custom version command
const versionResult = await execAsync('claude --version');
// Parses: "claude-code/1.0.86 darwin-arm64 node-v23.6.0"
const match = versionResult.stdout.match(/claude-code\/(\d+\.\d+\.\d+)/);
version = match ? match[1] : 'Unknown';
```

##### 2. Other NPM Tools
```typescript
// Uses npm list to get version
const listResult = await execAsync(`npm list -g ${packageName} --depth=0`);
const versionMatch = listResult.stdout.match(/@(\d+\.\d+\.\d+)/);
version = versionMatch ? versionMatch[1] : 'Unknown';
```

##### 3. Python Tools
```typescript
// Uses tool's --version flag
const versionResult = await execAsync('aider --version');
version = versionResult.stdout.trim().match(/\d+\.\d+\.\d+/)?.[0];
```

#### Error Handling

##### Permission Errors
```typescript
if (error.message?.includes('EACCES')) {
  return { 
    success: false, 
    error: 'Permission denied. Try running with elevated permissions or update manually with: ' + updateCommand 
  };
}
```

##### Missing Package Manager
```typescript
if (error.message?.includes('npm: command not found')) {
  return { 
    success: false, 
    error: 'npm not found. Please ensure Node.js and npm are installed.' 
  };
}
```

##### Network Errors
- Timeout handling via execAsync
- Retry logic for transient failures
- Clear error messages for offline scenarios

#### PATH Resolution

The update system ensures package managers can be found by enhancing PATH:

```typescript
const pathAdditions = [
  '/opt/homebrew/bin',    // Homebrew on M1 Macs
  '/usr/local/bin',        // Standard Unix
  '/usr/bin',              // System binaries
  '/bin'                   // Core binaries
];

const enhancedPath = `${pathAdditions.join(':')}:${process.env.PATH}`;
```

#### UI State Management

##### Visual States
1. **Default State**: Gray button, "Update" text
2. **Updating State**: Orange status, "⬆️ Updating..." text
3. **Success State**: Green status, "✅ Up to date" text
4. **Error State**: Red status, "❌ Update failed" text

##### Non-Blocking Updates
- UI remains responsive during update
- Status updates without full panel refresh
- Version number updates in-place
- Error messages display temporarily

#### Security Considerations

1. **Command Injection Prevention**
   - Tool IDs validated against whitelist
   - Package names mapped internally
   - No user input in shell commands

2. **Permission Management**
   - Graceful handling of permission errors
   - Clear instructions for manual updates
   - No automatic privilege escalation

3. **Version Verification**
   - Version extracted after update
   - Displayed to user for confirmation
   - Logged for audit trail

#### Performance Optimizations

1. **Async Execution**
   - Non-blocking child process execution
   - Promise-based error handling
   - Timeout protection (default 2 minutes)

2. **Minimal UI Updates**
   - Only affected elements updated
   - No full panel re-render
   - Smooth visual transitions

3. **Efficient Version Detection**
   - Tool-specific version commands
   - Cached PATH resolution
   - Single version check per update

#### Testing Considerations

##### Manual Testing
1. Click Update button for installed tool
2. Verify "Updating..." status appears
3. Wait for completion
4. Verify success message and new version
5. Check tool works with `claude --version`

##### Edge Cases
- Tool not installed → Update button shouldn't appear
- Network offline → Clear error message
- Permission denied → Helpful manual command
- Already latest version → Success message
- Corrupted installation → Error with reinstall suggestion

#### Future Enhancements

1. **Batch Updates**
   - Update all tools simultaneously
   - Progress bar for multiple updates
   - Rollback on failure

2. **Version Comparison**
   - Show current vs available version
   - Changelog preview
   - Update only if newer

3. **Automatic Updates**
   - Background update checks
   - User preference settings
   - Silent updates option

4. **Update Channels**
   - Stable vs beta channels
   - Version pinning
   - Downgrade capability

---

## AI Tools Launch Tracking & Database

### Overview
The AI Tools Launch Tracking system records which AI CLI tools have been launched in which repositories, enabling intelligent features like resume detection, usage analytics, and per-repository tool preferences. This system uses the unified SQLite database (`~/.hive/hive-ai.db`) shared with the main application.

### Architecture

#### AIToolsDatabase Service
**Location**: `src/services/AIToolsDatabase.ts`
**Purpose**: Track and manage AI tool launch history per repository
**Database**: Uses the unified `hive-ai.db` SQLite database
**Integration**: Singleton service integrated with main process

#### Database Schema

##### Table: `ai_tool_launches`
```sql
CREATE TABLE IF NOT EXISTS ai_tool_launches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tool_id TEXT NOT NULL,                    -- e.g., 'claude-code', 'grok', 'cline'
  repository_path TEXT NOT NULL,            -- e.g., '/Users/veronelazio/Developer/Private/hive'
  launch_count INTEGER DEFAULT 1,           -- Increments on each launch
  first_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
  last_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
  status TEXT DEFAULT 'active',             -- 'active', 'closed', or 'crashed'
  session_metadata TEXT,                    -- JSON metadata about the session
  user_id TEXT DEFAULT 'default',          
  tool_version TEXT,                        -- Version of the tool at launch
  launch_context TEXT,                      -- JSON context data (e.g., launch flags)
  UNIQUE(tool_id, repository_path, user_id),
  FOREIGN KEY (user_id) REFERENCES users(id)
)
```

##### Indexes for Performance
```sql
-- Fast lookup by tool and repository
CREATE INDEX idx_ai_tool_launches_lookup 
ON ai_tool_launches(tool_id, repository_path);

-- Recent launches query optimization
CREATE INDEX idx_ai_tool_launches_recent 
ON ai_tool_launches(last_launched_at DESC);

-- Active sessions filtering
CREATE INDEX idx_ai_tool_launches_active 
ON ai_tool_launches(status) WHERE status = 'active';
```

### Launch Tracking Flow

#### 1. Tool Launch Detection
When a user clicks an AI CLI tool in the UI:
```typescript
// Main process (src/index.ts)
ipcMain.handle('launch-ai-tool', async (event, toolId) => {
  // 1. Show folder selection dialog
  const { filePaths } = await dialog.showOpenDialog({
    properties: ['openDirectory'],
    title: `Select folder for ${toolName}`
  });
  
  // 2. Get AIToolsDatabase instance (uses unified db connection)
  const aiToolsDb = AIToolsDatabase.getInstance(db);
  
  // 3. Check launch history
  const hasBeenLaunched = aiToolsDb.hasBeenLaunchedBefore(
    toolId, 
    selectedPath
  );
  
  // 4. Determine command (e.g., 'claude' vs 'claude --continue')
  const command = hasBeenLaunched ? 
    `${baseCommand} --continue` : 
    baseCommand;
  
  // 5. Record the launch
  aiToolsDb.recordLaunch(toolId, selectedPath, {
    version: detectedVersion,
    context: { resumed: hasBeenLaunched }
  });
  
  // 6. Send event to renderer to create terminal tab
  mainWindow.webContents.send('launch-ai-tool-terminal', {
    toolId,
    command,
    cwd: selectedPath
  });
});
```

#### 2. Terminal Tab Creation
The renderer process receives the launch event and creates a TTYD terminal tab:
```typescript
// Renderer process (src/renderer.ts)
window.electronAPI.onLaunchAIToolTerminal((data) => {
  // Create new terminal tab in TTYDTerminalPanel
  terminal.createAIToolTab({
    id: data.toolId,
    title: data.toolName,
    command: data.command,
    cwd: data.cwd,
    port: allocatedPort
  });
});
```

#### 3. Session Management
```typescript
// Track active sessions
aiToolsDb.recordLaunch(toolId, repoPath, {
  status: 'active',
  sessionData: { pid: process.pid, port: allocatedPort }
});

// Close session when terminal exits
process.on('exit', () => {
  aiToolsDb.closeSession(toolId, repoPath);
});
```

### Key Features

#### 1. Intelligent Resume Detection
- Tracks if a tool has been launched in a repository before
- Enables tools like Claude Code to use `--continue` flag automatically
- Preserves context across sessions

#### 2. Usage Analytics
```typescript
// Get comprehensive usage statistics
const stats = aiToolsDb.getUsageStats();
// Returns:
{
  totalLaunches: 142,
  uniqueRepositories: 23,
  mostUsedTool: 'claude-code',
  activeSessions: 3
}
```

#### 3. Repository-Specific History
```typescript
// Get all tools launched in a specific repository
const launches = aiToolsDb.getRepositoryLaunches('/path/to/repo');
// Returns array of launch records with timestamps and counts
```

#### 4. Recent Activity Tracking
```typescript
// Get recent launches across all repositories
const recent = aiToolsDb.getRecentLaunches(10);
// Returns last 10 launches with full details
```

### Integration Points

#### 1. Unified Database Connection
- Uses the same SQLite database as the main application
- Database initialized in main process: `~/.hive/hive-ai.db`
- Shared connection passed to AIToolsDatabase singleton
- No separate database connections or files

#### 2. Error Handling
- Graceful degradation if database operations fail
- Tools still launch even without tracking
- Errors logged but don't block user workflow

#### 3. Cleanup & Maintenance
```typescript
// Periodic cleanup of old records (90+ days)
aiToolsDb.cleanupOldRecords(90);
// Removes closed sessions older than 90 days
```

### Benefits

1. **Enhanced User Experience**
   - Tools remember they've been used in a project
   - Automatic context restoration where supported
   - No manual flag management needed

2. **Analytics & Insights**
   - Track tool usage patterns
   - Identify most-used tools per project
   - Understand developer workflows

3. **Future Capabilities**
   - Tool recommendations based on project type
   - Automatic tool configuration per repository
   - Team usage analytics for enterprise

---

## CLI Tools Management UI

### Overview
The CLI Tools Management UI provides a dedicated, independent panel for managing agentic coding CLI tools, with its own icon in the activity bar. This positions CLI Tools as a core feature alongside Memory Service, Analytics, and Settings - not buried within settings.

### UI Architecture

#### Independent Panel Design
**Location**: `src/components/cli-tools-panel/`
**Activity Bar Position**: Between Settings and Memory icons
**Icon**: AI assistant robot face (recognizable AI symbol)
**Access**: Direct click on AI CLI Tools icon in activity bar

#### Component Structure
```typescript
src/components/
├── cli-tools-panel/
│   ├── CliToolsPanel.tsx       # Main panel container
│   ├── CliToolCard.tsx         # Individual tool card component
│   ├── ToolsGrid.tsx           # 2x3 responsive grid layout
│   ├── InstallationProgress.tsx # Real-time installation feedback
│   ├── ConnectionStatus.tsx    # Memory Service connection indicator
│   └── ActivityLog.tsx         # Recent actions and updates
```

### Activity Bar Integration

#### Icon Design
```svg
<!-- Clean AI Assistant Icon - Positioned between Settings and Memory -->
<svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
  <path d="M12 2C10.9 2 10 2.9 10 4S10.9 6 12 6 14 5.1 14 4 13.1 2 12 2M12 7C9.24 7 7 9.24 7 12V19C7 20.66 8.34 22 10 22H14C15.66 22 17 20.66 17 19V12C17 9.24 14.76 7 12 7M9 12C9 10.34 10.34 9 12 9S15 10.34 15 12V13H9V12M9 15H15V19C15 19.55 14.55 20 14 20H10C9.45 20 9 19.55 9 19V15M10 16V18H11V16H10M13 16V18H14V16H13Z"/>
</svg>
```

**Tooltip**: "AI CLI Tools - Manage AI coding assistants"
**Keyboard Shortcut**: `Ctrl/Cmd + Shift + T` (for Tools)

### Batch Operations UI

#### Top Control Bar
```
┌─────────────────────────────────────────────────────────┐
│ AI CLI TOOLS MANAGEMENT                                  │
├─────────────────────────────────────────────────────────┤
│ [🚀 Install All Tools] [🔄 Update All Tools]             │
│ Memory Service: ● Connected (Port 3457)                  │
└─────────────────────────────────────────────────────────┘
```

**Install All Tools Button**:
- Icon: 🚀 (rocket for quick setup)
- Color: Blue (#3b82f6)
- Hover: "Install all 6 AI CLI tools"
- Progress: "Installing 3 of 6..."
- Complete: "✅ Installed 4, skipped 2"

**Update All Tools Button**:
- Icon: 🔄 (refresh for updates)
- Color: Gray (#6b7280)
- Hover: "Update all installed tools"
- Progress: "Updating 3 tools..."
- Complete: "✅ Updated 3 tools"

### Panel Layout

```
┌─────────────────────────────────────────────────────────┐
│ AI CLI TOOLS MANAGEMENT                           [─][□][×]│
├─────────────────────────────────────────────────────────┤
│                                                           │
│ Memory Service: ● Connected (Port 3457)                  │
│                                                           │
│ ┌─────────────────────┬─────────────────────┬──────────┐│
│ │ Claude Code         │ Gemini CLI          │ Qwen Code││
│ │ [AI Icon]           │ [AI Icon]           │ [AI Icon]││
│ │ ● Installed v2.1.0  │ ○ Not Installed     │ ● v1.0.5 ││
│ │ Memory: Connected   │                     │ Memory: ✓ ││
│ │ [Configure] [↻]     │ [Install]           │ [Update] ││
│ └─────────────────────┴─────────────────────┴──────────┘│
│                                                           │
│ ┌─────────────────────┬─────────────────────┬──────────┐│
│ │ OpenAI Codex        │ Aider               │ Cline    ││
│ │ [AI Icon]           │ [AI Icon]           │ [AI Icon]││
│ │ ○ Not Installed     │ ● Installed v0.21   │ ○ Not    ││
│ │                     │ Memory: Disabled    │          ││
│ │ [Install]           │ [Configure]         │ [Install]││
│ └─────────────────────┴─────────────────────┴──────────┘│
│                                                           │
│ Activity Log:                                            │
│ ├─ 08:45 Claude Code connected to Memory Service         │
│ ├─ 08:42 Gemini CLI update available (v2.0)             │
│ └─ 08:40 Panel opened                                    │
└─────────────────────────────────────────────────────────┘
```

### Visual Design System

#### HiveTechs Theme Integration
Based on the HiveTechs website's premium Paddle-inspired dark theme, the AI CLI Tools interface features:

**Color Palette**:
- Primary Yellow: `#FFC107` - Main brand color for CTAs and highlights
- Secondary Blue: `#007BFF` - Accent color for gradients
- Dark Background: `#0E1414` - Main application background
- Card Background: `#1A1F1F` - Tool cards and panels
- Border Color: `rgba(255, 193, 7, 0.2)` - Subtle yellow-tinted borders
- Text Primary: `#E0E0E0` - Main text
- Text Secondary: `#AAA` - Muted labels

**Button Styling**:
```css
/* Premium gradient buttons */
.tool-button {
  background: linear-gradient(135deg, #FFC107 0%, #007BFF 100%);
  box-shadow: 0 4px 6px rgba(255, 193, 7, 0.25);
  border: none;
  color: #0E1414;
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.tool-button:hover {
  background: linear-gradient(135deg, #FFD54F 0%, #1976D2 100%);
  transform: translateY(-2px);
  box-shadow: 0 6px 12px rgba(255, 193, 7, 0.35);
}

/* Secondary actions */
.tool-button-secondary {
  background: transparent;
  border: 1px solid rgba(255, 193, 7, 0.3);
  color: #FFC107;
}

.tool-button-secondary:hover {
  background: rgba(255, 193, 7, 0.1);
  border-color: #FFC107;
}
```

**Batch Operation Buttons** (Install All, Update All, Uninstall All):
- Positioned at top of tools panel for easy access
- Use consistent gradient theming
- Include appropriate icons (codicon-cloud-download, codicon-sync, codicon-trash)
- Confirmation dialogs for destructive operations

#### AI CLI Tool Icons and Branding

**Resource Location**: `/electron-poc/resources/ai-cli-icons/`

**Icon Collection Status**:

| Tool | Icon File | Source | Format | Status |
|------|-----------|--------|--------|--------|
| **Claude Code** | `claude.svg` | Wikipedia/Wikimedia Commons | SVG (85 bytes) | ✅ Official |
| **Gemini CLI** | `gemini.svg` | Wikipedia/Wikimedia Commons | SVG (6.7KB) | ✅ Official |
| **Grok** | `grok.svg` | Wikipedia/Wikimedia Commons | SVG (96 bytes) | ✅ Official |
| **Qwen Code** | `qwen.svg` | Custom placeholder | SVG (477 bytes) | 🔄 Placeholder |
| **OpenAI Codex** | `openai.svg` | Wikipedia/Wikimedia Commons | SVG (2.9KB) | ✅ Official |
| **Cline** | `cline.svg` | Custom placeholder | SVG (745 bytes) | 🔄 Placeholder |

**Sidebar Quick Launch Implementation** (✅ Fully Implemented):
1. **Location**: Left activity bar, between Memory icon and Settings
2. **Layout**: Vertical stack of 24x24px icons with 4px gap (scaled from 42x42 for consistency)
3. **Organization**:
   - Divider after Source Control icon
   - Analytics, Memory, CLI Tools icons
   - Divider
   - 6 AI tool icons stacked vertically
   - Divider
   - Settings fixed at bottom
4. **Smart Click Interaction**:
   - **Installed Tools**: Single click launches tool using `launchCliTool()` function
   - **Uninstalled Tools**: Redirects to AI CLI Tools panel with auto-highlight of tool card
   - **Folder Selection**: Prompts for folder if tool not previously launched
   - **Status Feedback**: Shows launch status in tool's status area
5. **Visual Indicators**:
   - **Download Badge**: Blue download arrow (↓) for uninstalled tools
   - **Position**: Bottom-right corner of icon (bottom: -2px, right: -2px)
   - **Style**: Blue gradient background (#2196F3 to #1976D2)
   - **Auto-Refresh**: Icons update automatically after install/update/uninstall
6. **CSS Implementation**:
   - **Icon Brightness**: All sidebar icons brightened to #cccccc (from #858585)
   - **Filter Strategy**: `filter: invert(1)` for black icons (OpenAI, Qwen, Grok, Cline)
   - **Preserved Colors**: Claude and Gemini icons keep original colors (no inversion)
   - **Hover Effects**: All icons use `filter: brightness(1.1)` on hover
   - **Activity Bar Width**: 48px (optimized from initial 56px expansion)
7. **Technical Architecture**:
   - **Webpack Imports**: SVG icons imported as ES modules in renderer.ts
   - **IPC Integration**: Uses `window.electronAPI.detectCliTool()` for status checks
   - **Refresh Function**: `refreshSidebarToolIcon()` updates individual icon states
   - **Event Delegation**: Click handlers check installation status before action
8. **Implementation Details**:
   - **Function Call**: Uses same `(window as any).launchCliTool(toolId)` as main panel
   - **Tool IDs**: claude-code, gemini-cli, grok, qwen-code, openai-codex, cline
   - **Icons Source**: `/resources/ai-cli-icons/` directory with official SVG logos
   - Custom placeholders for Qwen and Cline
   - Consistent 20x20px size for all icons
   - HiveTechs theme color scheme applied
   - Update Available: Small badge indicator

**Icon Design Guidelines**:
- Consistent 24x24px display size
- 2px padding within container
- HiveTechs yellow (#FFC107) hover highlight
- Smooth 0.3s transitions for all interactions
- Respect original brand colors in icons

#### Tool Card States
Each CLI tool is represented by a card with multiple visual states:

**1. Not Installed State**
```
┌─────────────────────────────────────┐
│ [Icon] Tool Name                    │
│ ○ Not Installed                     │
│                                     │
│ [Description text]                  │
│                                     │
│ [Install] [Learn More]              │
└─────────────────────────────────────┘
```

**2. Installed State**
```
┌─────────────────────────────────────┐
│ [Icon] Tool Name                    │
│ ● Installed ✓ | v1.2.3             │
│ Memory: Connected ✓                 │
│                                     │
│ Last updated: 2 hours ago           │
│ Auto-update: Enabled                │
│                                     │
│ [Details] [Update] [Uninstall]      │
└─────────────────────────────────────┘
```

**3. Installing State**
```
┌─────────────────────────────────────┐
│ [Icon] Tool Name                    │
│ ◐ Installing... 60%                 │
│ ▓▓▓▓▓▓░░░░                         │
│                                     │
│ Downloading package...              │
│                                     │
│ [Cancel]                            │
└─────────────────────────────────────┘
```

**4. Error State**
```
┌─────────────────────────────────────┐
│ [Icon] Tool Name                    │
│ ● Error                             │
│                                     │
│ ⚠️ Installation failed              │
│ Python 3.8+ required                │
│                                     │
│ [Retry] [View Logs] [Get Help]      │
└─────────────────────────────────────┘
```

#### Color Scheme & Icons
```css
/* Status Colors */
--status-installed: #10b981;    /* Green */
--status-not-installed: #6b7280; /* Gray */
--status-installing: #3b82f6;    /* Blue */
--status-error: #ef4444;         /* Red */
--status-update: #f59e0b;        /* Amber */

/* Tool Icons */
Claude Code: 🤖
Gemini CLI: ✨
Qwen Code: 🐉
OpenAI Codex: 🌟
Aider: 🔧
Cline: 💬
```

### Responsive Layout

#### Desktop (>1024px)
- 3-column grid layout
- 340px card width
- 16px gap between cards
- Sidebar for filters/search

#### Tablet (768-1024px)
- 2-column grid layout
- Flexible card width
- Collapsible sidebar

#### Mobile (<768px)
- Single column layout
- Full-width cards
- Bottom sheet for filters

### Interactive Features

#### Smart Recommendations
```typescript
interface ToolRecommendation {
  toolId: string;
  reason: string;
  badge: 'recommended' | 'popular' | 'free' | 'new';
}

// Display logic
if (tool.id === 'gemini') {
  showBadge('FREE', 'green');
  showTooltip('1000 requests/day at no cost!');
}
```

#### Batch Operations
- Select multiple tools with checkboxes
- Bulk install/update/remove actions
- Progress tracking for batch operations

#### Search & Filter
```typescript
interface FilterOptions {
  status: 'all' | 'installed' | 'available' | 'updates';
  integration: 'all' | 'memory-service' | 'standalone';
  type: 'all' | 'npm' | 'python' | 'binary';
}
```

### Launch Button Functionality

#### Overview
The Launch button provides seamless integration between installed CLI tools and the IDE's global project context. It appears for all installed tools and launches them in the appropriate terminal environment.

#### Implementation Architecture

**1. Global Project Context Integration**
```typescript
// Uses the same currentOpenedFolder variable shared across:
- File Explorer
- Source Control (Git)
- Bottom Status Bar
- Launch Button

// Project context flow:
currentOpenedFolder → Launch Button → Terminal Launch
```

**2. Launch Flow**
```typescript
async function launchCliTool(toolId: string): Promise<void> {
  // Step 1: Check global project context
  if (!currentOpenedFolder) {
    // Prompt user to select folder
    const result = await showOpenDialog({ properties: ['openDirectory'] });
    if (result.filePaths[0]) {
      currentOpenedFolder = result.filePaths[0];
      await handleOpenFolder(result.filePaths[0]);
    }
  }
  
  // Step 2: Launch in project context
  await electronAPI.launchCliTool(toolId, currentOpenedFolder);
}
```

**3. Platform-Specific Terminal Launch**
```typescript
// macOS: AppleScript to open Terminal.app
command = `osascript -e 'tell application "Terminal" to do script "cd \\"${projectPath}\\" && claude"'`;

// Windows: Command Prompt
command = `start cmd /k "cd /d ${projectPath} && claude"`;

// Linux: Try multiple terminal emulators
command = `gnome-terminal -- bash -c "cd '${projectPath}' && claude; exec bash"`;
```

#### Button States & UI

**1. Installed Tool Buttons**
```
┌─────────────────────────────────────┐
│ [Icon] Claude Code                  │
│ ● Installed ✓ | v1.0.86            │
│ Memory: Connected ✓                 │
│                                     │
│ [Launch] [Details] [Configure] [Update] │
└─────────────────────────────────────┘
```

**2. Launch Button Behavior**
- **Color**: Blue (#2196f3) - distinguishes from other actions
- **Position**: First button for installed tools
- **Status**: Shows "Launching..." during operation
- **Success**: Brief "Running in: [folder]" confirmation

**3. No Project Context Handling**
```
When no folder is open:
1. Launch button clicked
2. Status shows: "⚠️ Please open a project folder first"
3. File dialog opens automatically
4. User selects folder
5. Global context updates (Explorer, Git, Status Bar)
6. Tool launches in selected folder
```

#### IPC Communication

**1. Renderer → Main Process**
```typescript
// preload.ts
launchCliTool: (toolId: string, projectPath: string) => 
  ipcRenderer.invoke('cli-tool-launch', toolId, projectPath)

// index.ts handler
ipcMain.handle('cli-tool-launch', async (_, toolId, projectPath) => {
  await manager.launch(toolId, projectPath);
  return { success: true };
});
```

**2. CliToolsManager Implementation**
```typescript
public async launch(toolId: string, projectPath: string): Promise<void> {
  // Verify tool is installed
  const status = await this.getToolStatus(toolId);
  if (!status.installed) {
    throw new Error(`${tool.name} is not installed`);
  }
  
  // Launch with platform-specific command
  await this.launchClaudeCode(projectPath);
}
```

### Integrated Terminal System 🔄 TRANSITIONING TO TTYD

#### Vision
Transform the fixed bottom console into a powerful tabbed terminal system where users can run multiple AI tools simultaneously, each in its own named tab, alongside regular terminal sessions. This creates a unified workspace where all AI assistants are immediately accessible without window switching.

**Note**: The original basic HTML terminal section at the bottom center has been hidden (`display: none`) as of v1.7.2, since all logging functionality has been consolidated into the System Log tab within the TTYD terminal panel. The code remains in place but hidden for potential future use.

**Original Vision Alignment**: The ttyd approach actually fulfills our original vision better than xterm.js ever could. By providing real terminals that can handle any TUI application perfectly, we achieve the seamless AI tool integration we envisioned - where Claude Code, Aider, and other CLI tools work flawlessly within our IDE tabs.

#### Implementation Status: ✅ PRODUCTION-READY WITH TTYD
The terminal system has been successfully implemented using ttyd (terminal server) to provide real terminal emulation that perfectly handles sophisticated TUI applications like Claude Code. This approach delivers flawless compatibility with zero rendering issues.

#### Terminal Tab Architecture

**Tab Types & Naming Convention**:
```
┌──────────────────────────────────────────────────────────────────┐
│ ◀ │ System Log | Claude | Gemini | Qwen | Terminal 1 | Terminal 2 | 📊 | + │ ▶ │
└──────────────────────────────────────────────────────────────────┘
        ↑            ↑                    ↑              ↑    ↑
    Navigation   AI Tool Tabs      Generic Terminals  Toggle  New
      arrows      (Named)            (Numbered)       System Log
    (appear on overflow)
```

1. **System Log Tab** (Always First, Read-Only)
   - Current console output preserved exactly as-is
   - Read-only system messages and debugging info
   - Non-closeable, non-interactive
   - Shows backend activity, memory service status, process logs
   - Auto-scrolls to bottom for latest messages
   - **Toggle Visibility Feature** (v1.8.1):
     - Hidden by default on startup for cleaner UI
     - Toggle button (📊 icon) in terminal header bar
     - Button opacity: 70% when hidden, 100% when visible
     - Tooltip: "Toggle System Log (📊)"
     - When hidden, other tabs remain functional
     - Automatically switches to another tab if System Log was active
   - Search functionality to find specific logs

2. **AI Tool Tabs** (Named by Tool)
   - Claude, Gemini, Qwen, Codex, Aider, Cline
   - Launch with tool-specific commands (e.g., `claude` or `claude --resume`)
   - Per-repository launch tracking for intelligent resume detection
   - Icon indicator for running state
   - Color-coded for easy identification
   - Closeable with confirmation if tool is running

3. **Generic Terminal Tabs** (Numbered with Smart Reuse)
   - Terminal 1, Terminal 2, etc.
   - Intelligent number recycling: reuses lowest available number
   - User-created for general commands
   - Full shell access with working directory awareness
   - Closeable without confirmation

4. **Tab Navigation Features**
   - **Overflow Navigation Arrows**: Appear when tabs exceed container width
   - **Smooth Scrolling**: 80% viewport scroll on arrow click
   - **Keyboard Shortcuts**: Ctrl/Cmd + Shift + Left/Right for tab switching
   - **Auto-scroll**: Active tab automatically scrolls into view
   - **Responsive**: ResizeObserver monitors container for arrow visibility

#### Visual Design

```
Bottom Terminal Area (Resizable):
┌────────────────────────────────────────────────────────────────────┐
│ [📊 System Log] [🤖 Claude] [✨ Gemini] [🐉 Qwen] [Terminal 1] [+] │
├────────────────────────────────────────────────────────────────────┤
│ [System Log Tab - Read Only]                                        │
│ [INFO] [ProcessManager] websocket-backend started successfully      │
│ [INFO] [MemoryService] Server running on http://localhost:3457      │
│ [INFO] [Main] Memory Service ready on port: 3457                    │
│ [INFO] [MemoryService] Stats query result: 142 memories             │
│ [INFO] WebSocket reconnected successfully                           │
│ [INFO] [Main] Detecting CLI tool: claude-code                       │
│ [INFO] [CliToolsManager] Claude Code launched in /Users/dev/project │
│ [INFO] [ProcessManager] All systems operational                     │
│                                                                      │
│ [Search: ____________________] [Clear] [Export Logs]                │
└────────────────────────────────────────────────────────────────────┘

When Claude tab is active:
┌────────────────────────────────────────────────────────────────────┐
│ [📊 System Log] [🤖 Claude] [✨ Gemini] [🐉 Qwen] [Terminal 1] [+] │
├────────────────────────────────────────────────────────────────────┤
│ ~/Developer/Private/hive $ claude                                   │
│                                                                      │
│ Welcome to Claude Code v1.0.86                                      │
│ Connected to Memory Service ✓                                       │
│                                                                      │
│ You can ask me about your codebase or request changes.              │
│ Type /help for available commands.                                  │
│                                                                      │
│ > How can I improve the performance of this React component?        │
│                                                                      │
│ I'll analyze your React component for performance improvements...   │
│ [Claude's response continues...]                                    │
│                                                                      │
└────────────────────────────────────────────────────────────────────┘
```

#### Implementation Architecture

**Core Technologies (Production TTYD Implementation)**:
```typescript
interface TerminalSystemDependencies {
  'ttyd': 'latest',                      // Terminal server providing real terminals
  'node-pty': '^1.0.0',                  // Shell process spawning
  'tree-kill': '^1.2.2',                 // Process cleanup
  'portfinder': '^1.0.32',               // Dynamic port allocation
  'better-sqlite3': '^9.6.0',            // AI tools launch tracking database
}
```

**TTYD Terminal Server Manager**:
```typescript
// src/services/TTYDManager.ts
export class TTYDManager extends EventEmitter {
  private instances: Map<string, TTYDInstance> = new Map();
  
  async createTerminal(config: TTYDConfig): Promise<TTYDInstance> {
    // 1. Dynamic port allocation (7100-7999 range)
    const port = await PortManager.allocatePort({
      port: 7100,
      serviceName: `ttyd-${config.id}`,
      alternativePorts: Array.from({ length: 900 }, (_, i) => 7100 + i)
    });
    
    // 2. Prepare ttyd arguments
    const ttydArgs = [
      '--port', port.toString(),
      '--interface', '127.0.0.1',  // Security: localhost only
      '--writable'                  // Allow input
    ];
    
    // 3. Handle command execution with delay
    if (config.command) {
      const initCommand = `sleep 0.5 && ${config.command}`;
      ttydArgs.push('--', shell, '-c', `${initCommand}; exec ${shell} -i`);
    }
    
    // 4. Spawn ttyd process
    const ttydProcess = spawn('ttyd', ttydArgs, {
      cwd: config.cwd || process.env.HOME,
      env: {
        ...process.env,
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor'
      }
    });
    
    // 5. Return terminal instance with URL
    return {
      id: config.id,
      title: config.title,
      port,
      url: `http://localhost:${port}`,
      process: ttydProcess,
      status: 'running'
    };
  }
}
```

**Why TTYD Over xterm.js**:
1. **Real Terminal Emulation**: ttyd provides actual terminal processes, not JavaScript emulation
2. **Perfect TUI Support**: Handles Claude Code, vim, tmux, and other TUI apps flawlessly  
3. **Zero Rendering Issues**: No % characters, no duplicate UI, no cursor problems
4. **Low Maintenance**: Battle-tested solution used in production environments
5. **Native Performance**: Direct terminal output without JavaScript translation overhead

#### Terminal Tab Display Fix (DOM Safety Enhancement)

**Problem Solved**: Terminal tabs disappearing for AI CLI tools
**Root Cause**: DOM element references lost during Git panel updates
**Solution**: Enhanced TTYDTerminalPanel with safety checks and auto-recovery

```typescript
// Auto-recovery system for tabs container
private ensureTabsContainer(): HTMLElement {
  if (!this.tabsContainer) {
    this.tabsContainer = document.getElementById('isolated-terminal-tabs');
    if (!this.tabsContainer) {
      // Recreate if missing
      const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
      if (wrapper) {
        this.tabsContainer = document.createElement('div');
        this.tabsContainer.id = 'isolated-terminal-tabs';
        this.tabsContainer.className = 'isolated-terminal-tabs';
        this.tabsContainer.style.cssText = 'display: flex; align-items: center;';
        wrapper.appendChild(this.tabsContainer);
      }
    }
  }
  return this.tabsContainer;
}
```

**Key Improvements**:
- DOM element validation before every operation
- Automatic recreation of missing container elements  
- Null-safe operations throughout tab management
- Preserved functionality across panel refreshes
- No impact on terminal server connections

#### 🚨 TTYD Terminal Server Architecture (Working Implementation)

**How Our TTYD Integration Actually Works**:

1. **TTYDManager Service**: Centralized management of all ttyd instances
   ```typescript
   class TTYDManager extends EventEmitter {
     private terminals: Map<string, TTYDTerminal> = new Map();
     private processManager: ProcessManager;
     
     async createTerminal(options: {
       id: string;
       title: string;
       toolId?: string;      // For AI tool terminals
       cwd?: string;         // Working directory
       command?: string;     // Initial command (e.g., 'claude')
       env?: Record<string, string>;
     }): Promise<TTYDTerminal> {
       // Find available port in 7100-7999 range
       const port = await this.findAvailablePort();
       
       // Spawn ttyd with login shell for proper environment
       const ttydPath = '/opt/homebrew/bin/ttyd';
       const args = [
         '-p', port.toString(),
         '-t', 'disableLeaveAlert=true',
         '-t', 'fontSize=14',
         '-t', 'theme={"background":"#1e1e1e"}',
         '-W',  // Writable
         '/bin/zsh', '-l'  // Login shell for full environment
       ];
       
       const process = spawn(ttydPath, args, {
         cwd: options.cwd,
         env: { ...process.env, ...options.env }
       });
       
       // Store terminal instance
       const terminal = {
         id: options.id,
         title: options.title,
         url: `http://localhost:${port}`,
         port,
         process,
         status: 'starting',
         toolId: options.toolId,
         terminalNumber: options.toolId ? undefined : this.getNextTerminalNumber()
       };
       
       this.terminals.set(options.id, terminal);
       return terminal;
     }
   }
   ```

2. **Dynamic Port Allocation**: Managed range 7100-7999
   ```typescript
   private async findAvailablePort(): Promise<number> {
     const basePort = 7100;
     const maxPort = 7999;
     
     // Get all ports in use by our ttyd instances
     const usedPorts = Array.from(this.terminals.values())
       .map(t => t.port)
       .filter(p => p !== undefined);
     
     // Find first available port
     for (let port = basePort; port <= maxPort; port++) {
       if (!usedPorts.includes(port) && await this.isPortAvailable(port)) {
         return port;
       }
     }
     throw new Error('No available ports in range 7100-7999');
   }
   ```

3. **WebView Integration (NOT iframe)**: Each tab uses Electron webview
   ```typescript
   // TTYDTerminalPanel creates webview for each terminal
   private createTerminalContent(terminal: TerminalInfo): void {
     const webview = document.createElement('webview');
     webview.src = terminal.url;
     webview.setAttribute('nodeintegration', 'false');
     webview.setAttribute('webpreferences', 'contextIsolation=true');
     webview.style.width = '100%';
     webview.style.height = '100%';
     
     // Handle initial command execution
     if (terminal.command) {
       webview.addEventListener('dom-ready', () => {
         setTimeout(() => {
           webview.executeJavaScript(`
             if (window.term && window.term.paste) {
               window.term.paste('${terminal.command}\\n');
             }
           `);
         }, 500);
       });
     }
     
     tabContent.appendChild(webview);
   }
   ```

4. **Terminal Number Reuse System**: Smart recycling for generic terminals
   ```typescript
   // Track active terminal numbers
   const activeTerminalNumbers = new Set<number>();
   
   function getNextTerminalNumber(): number {
     let num = 1;
     while (activeTerminalNumbers.has(num)) {
       num++;
     }
     activeTerminalNumbers.add(num);
     return num;
   }
   
   // On terminal close, free the number
   function closeTerminal(terminalId: string): void {
     const terminal = terminals.get(terminalId);
     if (terminal?.terminalNumber) {
       activeTerminalNumbers.delete(terminal.terminalNumber);
     }
   }
   ```

5. **AI Tool Launch with Resume Detection**: Database-tracked launches
   ```typescript
   async launchAITool(toolId: string, projectPath: string) {
     // Check database for previous launch in this folder
     const previousLaunch = await db.getAIToolLaunch(toolId, projectPath);
     
     // Determine command based on history
     const command = previousLaunch ? 
       `${toolId} --resume` : 
       toolId;
     
     // Create terminal with appropriate title
     const terminal = await ttydManager.createTerminal({
       id: `tool-${toolId}-${Date.now()}`,
       title: getToolDisplayName(toolId),  // 'Claude', 'Gemini', etc.
       toolId: toolId,
       cwd: projectPath,
       command: command
     });
     
     // Record launch in database
     await db.recordAIToolLaunch(toolId, projectPath);
     
     return terminal;
   }

#### Terminal Panel UI Architecture (TTYDTerminalPanel)

**Complete Implementation with Tab Management**:
```typescript
// src/components/TTYDTerminalPanel.ts
export class TTYDTerminalPanel {
  private tabs: Map<string, TerminalTab> = new Map();
  private activeTabId: string | null = null;
  private terminalCounter = 0;
  private usedNumbers = new Set<number>();
  
  async createTerminalTab(
    toolId?: string,
    command?: string,
    cwd?: string
  ): Promise<TerminalTab> {
    // Generate unique ID and determine tab title
    const tabId = toolId || `terminal-${Date.now()}`;
    let title: string;
    
    if (toolId) {
      // AI tool tabs: Named by tool
      const toolNames: Record<string, string> = {
        'claude-code': 'Claude',
        'gemini-cli': 'Gemini',
        'qwen-code': 'Qwen',
        'aider': 'Aider',
        'cline': 'Cline'
      };
      title = toolNames[toolId] || 'Tool';
    } else {
      // Generic terminals: Smart number reuse
      const nextNumber = this.getNextAvailableNumber();
      title = `Terminal ${nextNumber}`;
    }
    
    // Create terminal via IPC
    const result = await window.terminalAPI.createTerminalProcess({
      terminalId: tabId,
      command,
      cwd,
      toolId
    });
    
    // Create tab UI
    const tab = this.createTabElement(tabId, title, toolId);
    
    // Create webview for ttyd
    const webview = this.createWebview(result.url);
    
    // Store tab info
    this.tabs.set(tabId, {
      id: tabId,
      title,
      toolId,
      element: tab,
      webview,
      url: result.url
    });
    
    // Activate the new tab
    this.activateTab(tabId);
    
    return this.tabs.get(tabId)!;
  }
  
  private getNextAvailableNumber(): number {
    // Smart number reuse: Find lowest available number
    let number = 1;
    while (this.usedNumbers.has(number)) {
      number++;
    }
    this.usedNumbers.add(number);
    return number;
  }
  
  private handleTabNavigation(): void {
    // Check if tabs overflow container
    const container = this.tabsContainer;
    const isOverflowing = container.scrollWidth > container.clientWidth;
    
    // Show/hide navigation arrows
    this.leftArrow.style.display = isOverflowing ? 'flex' : 'none';
    this.rightArrow.style.display = isOverflowing ? 'flex' : 'none';
    
    // Smooth scroll on arrow click (80% viewport)
    this.leftArrow.onclick = () => {
      container.scrollBy({
        left: -container.clientWidth * 0.8,
        behavior: 'smooth'
      });
    };
    
    this.rightArrow.onclick = () => {
      container.scrollBy({
        left: container.clientWidth * 0.8,
        behavior: 'smooth'
      });
    };
  }
}
```

#### Terminal Panel UI Architecture (TTYDTerminalPanel)

**Tab Management System**:
```typescript
class TTYDTerminalPanel {
  private tabs: Map<string, TabInfo> = new Map();
  private activeTabId: string | null = null;
  private tabScrollOffset: number = 0;  // For navigation arrows
  
  // Tab Creation & Management
  async createTerminal(type: 'generic' | 'ai-tool', options?: {
    toolId?: string;
    command?: string;
    cwd?: string;
  }): Promise<void> {
    // Request terminal from main process via IPC
    const result = await window.electronAPI.createTerminalProcess({
      terminalId: generateId(),
      toolId: options?.toolId,
      command: options?.command,
      cwd: options?.cwd || process.cwd()
    });
    
    if (result.success) {
      this.addTab(result.terminal);
      this.switchToTab(result.terminal.id);
    }
  }
  
  // Tab Navigation with Overflow Handling
  private setupTabNavigation(): void {
    const container = document.querySelector('.isolated-terminal-tabs-wrapper');
    const tabsElement = document.querySelector('.isolated-terminal-tabs');
    const leftArrow = document.getElementById('tab-nav-left');
    const rightArrow = document.getElementById('tab-nav-right');
    
    // ResizeObserver monitors for overflow
    const resizeObserver = new ResizeObserver(() => {
      const needsScroll = tabsElement.scrollWidth > container.clientWidth;
      leftArrow.style.display = needsScroll ? 'flex' : 'none';
      rightArrow.style.display = needsScroll ? 'flex' : 'none';
      this.updateArrowStates();
    });
    
    // Smooth scrolling (80% of viewport)
    leftArrow.addEventListener('click', () => this.scrollTabs('left'));
    rightArrow.addEventListener('click', () => this.scrollTabs('right'));
    
    // Keyboard shortcuts (Ctrl/Cmd + Shift + Arrow)
    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey) {
        if (e.key === 'ArrowLeft') this.switchToPreviousTab();
        if (e.key === 'ArrowRight') this.switchToNextTab();
      }
    });
  }
  
  private scrollTabs(direction: 'left' | 'right'): void {
    const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
    const tabs = document.querySelector('.isolated-terminal-tabs');
    const scrollAmount = Math.floor(wrapper.clientWidth * 0.8);
    
    if (direction === 'left') {
      this.tabScrollOffset = Math.max(0, this.tabScrollOffset - scrollAmount);
    } else {
      const maxScroll = tabs.scrollWidth - wrapper.clientWidth;
      this.tabScrollOffset = Math.min(maxScroll, this.tabScrollOffset + scrollAmount);
    }
    
    tabs.style.transform = `translateX(-${this.tabScrollOffset}px)`;
    this.updateArrowStates();
  }
  
  // Auto-scroll to show active tab
  private ensureTabVisible(tabId: string): void {
    const tabElement = document.querySelector(`[data-tab-id="${tabId}"]`);
    if (!tabElement) return;
    
    const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
    const tabLeft = tabElement.offsetLeft;
    const tabRight = tabLeft + tabElement.clientWidth;
    const viewLeft = this.tabScrollOffset;
    const viewRight = viewLeft + wrapper.clientWidth;
    
    if (tabLeft < viewLeft) {
      this.tabScrollOffset = tabLeft;
    } else if (tabRight > viewRight) {
      this.tabScrollOffset = tabRight - wrapper.clientWidth;
    }
    
    document.querySelector('.isolated-terminal-tabs').style.transform = 
      `translateX(-${this.tabScrollOffset}px)`;
  }
}
```

#### IPC Communication Layer

**Terminal IPC Handlers** (Main Process):
```typescript
// terminal-ipc-handlers.ts
export function registerTerminalHandlers(mainWindow: BrowserWindow): void {
  const ttydManager = new TTYDManager(processManager);
  const activeTerminalNumbers = new Set<number>();
  
  // Create terminal with intelligent numbering
  ipcMain.handle('create-terminal-process', async (event, options) => {
    const id = options.terminalId || generateId();
    let title: string;
    let terminalNumber: number | undefined;
    
    if (options.toolId) {
      // AI tool terminal - use tool name
      title = getToolDisplayName(options.toolId);
    } else {
      // Generic terminal - use recycled number
      terminalNumber = getNextTerminalNumber();
      activeTerminalNumbers.add(terminalNumber);
      title = `Terminal ${terminalNumber}`;
    }
    
    const terminal = await ttydManager.createTerminal({
      id,
      title,
      toolId: options.toolId,
      cwd: options.cwd || process.env.HOME,
      command: options.command,
      env: options.env
    });
    
    // Store terminal number for later recycling
    if (terminalNumber !== undefined) {
      terminal.terminalNumber = terminalNumber;
    }
    
    return { success: true, terminal };
  });
  
  // Close terminal and recycle number
  ipcMain.handle('kill-terminal-process', async (event, terminalId) => {
    const terminal = ttydManager.getTerminal(terminalId);
    if (terminal?.terminalNumber) {
      activeTerminalNumbers.delete(terminal.terminalNumber);
    }
    return await ttydManager.closeTerminal(terminalId);
  });
}
```

#### Tab UI Styling and Layout

**CSS Architecture**:
```css
/* Tab container with navigation arrows */
.isolated-terminal-header {
  padding-left: 30px;  /* Space for collapse button */
  position: relative;
}

.tab-nav-arrow {
  position: absolute;
  width: 24px;
  height: 100%;
  z-index: 1000;  /* Above collapse button */
  cursor: pointer;
  transition: background-color 0.2s;
}

#tab-nav-left { left: 30px; }
#tab-nav-right { right: 10px; }

.isolated-terminal-tabs-wrapper {
  overflow: hidden;
  margin: 0 34px;  /* Space for arrows */
}

.isolated-terminal-tabs {
  display: flex;
  transition: transform 0.3s ease;  /* Smooth scrolling */
}

.isolated-terminal-tab {
  flex-shrink: 0;  /* Prevent tab squishing */
  min-width: 120px;
  max-width: 200px;
}
```

#### Summary: Production Terminal System

**What We Built**:
- **Full Terminal Emulation**: Real bash/zsh terminals via ttyd, not JavaScript emulation
- **Perfect Claude Code Support**: Handles all TUI applications flawlessly
- **Smart Tab Management**: Intelligent terminal numbering with recycling
- **Overflow Navigation**: Arrow buttons and keyboard shortcuts for many tabs
- **AI Tool Integration**: Named tabs for Claude, Gemini, Qwen, etc.
- **Resume Detection**: Database tracking for intelligent `--resume` flag usage

**Key Implementation Files**:
1. **TTYDManager.ts**: Core ttyd process management and port allocation
2. **TTYDTerminalPanel.ts**: UI layer with tab management and navigation
3. **terminal-ipc-handlers.ts**: IPC bridge between renderer and main process
4. **CliToolsManager.ts**: AI tool launch integration (to be updated)
5. **renderer.ts**: HTML structure with navigation arrows

**Architecture Benefits**:
- **Zero Maintenance**: ttyd handles all terminal emulation complexity
- **Perfect Compatibility**: Works with any CLI tool without modification
- **Resource Efficient**: Each terminal is isolated with its own port
- **User-Friendly**: Familiar tab interface with VS Code-like shortcuts
- **Extensible**: Easy to add new AI tools or terminal features

**Next Steps for AI Tool Integration**:
1. Design database schema for AI tool launch tracking
2. Implement per-repository launch detection
3. Update CliToolsManager to use TTYDTerminalPanel
4. Add Claude Code launch with resume support

**Tab Component Structure**:
```typescript
interface TerminalTab {
  id: string;
  title: string;
  icon?: string;
  type: 'console' | 'ai-tool' | 'generic';
  isActive: boolean;
  isRunning: boolean;
  isCloseable: boolean;
  badge?: {
    text: string;
    color: string;
  };
}

// Tab rendering
<div className="terminal-tabs">
  <Tab 
    key="system-log" 
    title="System Log" 
    icon="📊" 
    isCloseable={false}
    isActive={activeTab === 'system-log'}
    isReadOnly={true}
  />
  {aiToolTabs.map(tab => (
    <Tab 
      key={tab.id}
      title={tab.title}
      icon={getToolIcon(tab.toolId)}
      isCloseable={true}
      isActive={activeTab === tab.id}
      badge={tab.isRunning ? { text: '●', color: 'green' } : null}
    />
  ))}
  {genericTabs.map((tab, index) => (
    <Tab 
      key={tab.id}
      title={`Terminal ${index + 1}`}
      isCloseable={true}
      isActive={activeTab === tab.id}
    />
  ))}
  <NewTabButton onClick={createNewTerminal} />
</div>
```

#### Launch Flow with Integrated Terminal

**1. Launch Button Click**:
```typescript
async function launchCliTool(toolId: string, mode: 'integrated' | 'external' = 'integrated') {
  if (mode === 'integrated') {
    // Check if tool already has a terminal
    const existingTerminal = terminalService.getToolTerminal(toolId);
    
    if (existingTerminal) {
      // Switch to existing tab
      terminalService.switchToTerminal(existingTerminal.id);
      
      // Optionally restart the tool
      if (!terminalService.isToolRunning(toolId)) {
        terminalService.sendCommand(existingTerminal.id, getToolCommand(toolId));
      }
    } else {
      // Create new terminal tab for tool
      const terminal = terminalService.createAIToolTerminal(toolId, getToolName(toolId));
      
      // Launch tool in terminal
      await terminalService.launchToolInTerminal(toolId, currentOpenedFolder);
    }
  } else {
    // External terminal launch (current implementation)
    await electronAPI.launchCliTool(toolId, currentOpenedFolder);
  }
}
```

**2. Terminal Creation Process (TTYD)**:
```typescript
async createAIToolTerminal(toolId: string, toolName: string): Promise<TTYDTerminalInstance> {
  // Get available port for ttyd
  const port = await this.ttydManager.getAvailablePort();
  
  // Spawn ttyd process
  const ttydProcess = spawn('ttyd', [
    '--port', port.toString(),
    '--once',                    // Single connection
    '--writable',               // Allow input
    '--check-origin', 'off',    // Allow Electron connection
    '--base-path', `/terminal/${toolId}`,
    '--title', toolName,
    '/bin/zsh'                  // Use zsh for AI tools
  ], {
    cwd: currentOpenedFolder || process.env.HOME,
    env: {
      ...process.env,
      TERM: 'xterm-256color',
      COLORTERM: 'truecolor'
    }
  });
  
  // Wait for ttyd to be ready
  await this.waitForPort(port);
  
  // Create webview for terminal
  const webview = document.createElement('webview');
  webview.src = `http://localhost:${port}`;
  webview.style.width = '100%';
  webview.style.height = '100%';
  webview.nodeintegration = false;
  webview.partition = 'persist:terminals';  // Isolated storage
  
  // Create terminal instance
  const instance: TTYDTerminalInstance = {
    id: `${toolId}-${Date.now()}`,
    type: 'ai-tool',
    title: toolName,
    icon: getToolIcon(toolId),
    toolId,
    ttydUrl: `http://localhost:${port}`,
    webview,
    ttydProcess,
    port,
    isActive: true,
    createdAt: new Date(),
    lastActivityAt: new Date()
  };
  
  // Add to terminals map
  this.terminals.set(instance.id, instance);
  
  // Create tab UI
  this.createTab(instance);
  
  return instance;
}
```

#### System Log Tab Implementation

**Special Handling for System Log**:
```typescript
class SystemLogManager {
  private logBuffer: string[] = [];
  private maxBufferSize: number = 10000; // Keep last 10k lines
  private logElement: HTMLElement;
  private searchTerm: string = '';
  
  appendLog(message: string, level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG') {
    const timestamp = new Date().toISOString();
    const formattedLog = `[${timestamp}] [${level}] ${message}`;
    
    // Add to buffer
    this.logBuffer.push(formattedLog);
    if (this.logBuffer.length > this.maxBufferSize) {
      this.logBuffer.shift();
    }
    
    // Update UI
    this.renderLog();
    
    // Auto-scroll to bottom if user is near bottom
    if (this.isNearBottom()) {
      this.scrollToBottom();
    }
  }
  
  search(term: string) {
    this.searchTerm = term;
    this.renderLog();
    this.highlightMatches();
  }
  
  exportLogs() {
    const blob = new Blob([this.logBuffer.join('\n')], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `system-logs-${Date.now()}.txt`;
    a.click();
  }
  
  clear() {
    this.logBuffer = ['[System Log Cleared]'];
    this.renderLog();
  }
}
```

**System Log Features (Current Implementation - v1.8.2)**:
- **Simple HTML Rendering**: Uses divs instead of xterm to avoid control characters
- **Console Capture**: Intercepts console.log, console.error, console.warn
- **Color Coding**: INFO (#cccccc), WARN (#dcdcaa), ERROR (#f44747)
- **Smart Auto-Scroll**: 
  - Automatically scrolls to bottom for new entries when viewing recent logs
  - Pauses auto-scroll when user scrolls up (>100px from bottom)
  - Resumes auto-scroll when user returns to bottom (<20px from bottom)
  - Multiple scroll attempts to ensure reliability
- **Manual Scrolling**: Full support for reviewing log history
- **Timestamps**: Shows time in toLocaleTimeString() format
- **Clean Output**: Simple args.join(' ') for readable messages
- **No xterm.js**: Avoids terminal control characters that cause display issues
- **Word Wrapping**: Long messages wrap properly with `word-wrap: break-word`
- **Performance**: Limited to 1000 entries to prevent memory issues

**System Log Filter Feature (Planned Enhancement)**:
```
┌────────────────────────────────────────────────────────────────────┐
│ [📊 System Log] [🤖 Claude] [✨ Gemini] [Terminal 1] [+]           │
├────────────────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ Filter: [All ▼] [❌ Errors] [⚠️ Warnings] [ℹ️ Info] [Clear]  │   │
│ └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│ [12:34:56] [INFO] System Log initialized                            │
│ [12:34:57] [INFO] Memory Service started on port 3457               │
│ [12:34:58] [WARN] WebSocket connection retry attempt 1              │
│ [12:34:59] [ERROR] Failed to connect to backend: ECONNREFUSED       │
│ [12:35:00] [INFO] WebSocket connected successfully                  │
│                                                                      │
│ Showing: 5 entries (2 INFO, 1 WARN, 1 ERROR) | Total: 142 entries  │
└────────────────────────────────────────────────────────────────────┘
```

**Filter Implementation Design**:
```typescript
interface SystemLogFilter {
  level: 'all' | 'error' | 'warn' | 'info';
  searchTerm?: string;
}

class SystemLogManager {
  private allEntries: LogEntry[] = [];
  private filteredEntries: LogEntry[] = [];
  private currentFilter: SystemLogFilter = { level: 'all' };
  
  applyFilter(filter: SystemLogFilter): void {
    this.currentFilter = filter;
    this.filteredEntries = this.allEntries.filter(entry => {
      // Level filter
      if (filter.level !== 'all' && entry.level.toLowerCase() !== filter.level) {
        return false;
      }
      // Search filter (optional future enhancement)
      if (filter.searchTerm && !entry.message.includes(filter.searchTerm)) {
        return false;
      }
      return true;
    });
    this.render();
  }
  
  addEntry(level: string, message: string): void {
    const entry = {
      timestamp: new Date(),
      level,
      message
    };
    this.allEntries.push(entry);
    
    // Check if entry passes current filter
    if (this.passesFilter(entry)) {
      this.filteredEntries.push(entry);
      this.renderEntry(entry);
    }
    
    this.updateStats();
  }
  
  clearLog(): void {
    this.allEntries = [];
    this.filteredEntries = [];
    this.render();
  }
}
```

**UI Components**:
1. **Filter Buttons**: Toggle buttons for each log level
   - Visual state indication (pressed/unpressed)
   - Single click to show only that level
   - Click again to show all

2. **Dropdown Alternative**: Single dropdown with options
   - All Messages
   - Errors Only
   - Warnings Only  
   - Info Only
   
3. **Clear Button**: Clears all log entries
   - Confirmation dialog for safety
   - Keyboard shortcut: Ctrl+L

4. **Statistics Bar**: Shows filtered vs total count
   - "Showing: X entries (Y errors, Z warnings) | Total: N entries"
   - Updates in real-time

**Visual Design**:
```css
.log-filter-bar {
  display: flex;
  align-items: center;
  padding: 8px;
  background: var(--vscode-editorWidget-background);
  border-bottom: 1px solid var(--vscode-editorWidget-border);
}

.filter-button {
  padding: 4px 8px;
  margin: 0 4px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.filter-button.active {
  background: var(--vscode-button-background);
  color: var(--vscode-button-foreground);
}

.filter-button:hover:not(.active) {
  background: var(--vscode-button-hoverBackground);
}

.log-stats {
  margin-left: auto;
  color: var(--vscode-descriptionForeground);
  font-size: 12px;
}
```

**Keyboard Shortcuts**:
- `Alt+E`: Show errors only
- `Alt+W`: Show warnings only
- `Alt+I`: Show info only
- `Alt+A`: Show all
- `Ctrl+L`: Clear log
- `Ctrl+F`: Focus search box (future)

#### Terminal Features

**1. Context Menu for Tabs**:
```
Right-click on tab:
- Restart Tool (AI tabs only)
- Clear Terminal
- Copy Terminal Output
- Split Right
- Close Tab
- Close Other Tabs
- Close All Tabs
```

**2. Keyboard Shortcuts**:
```
Ctrl/Cmd + T: New Terminal
Ctrl/Cmd + W: Close Current Tab
Ctrl/Cmd + Tab: Next Tab
Ctrl/Cmd + Shift + Tab: Previous Tab
Ctrl/Cmd + 1-9: Switch to Tab N
Ctrl/Cmd + Shift + C: Copy
Ctrl/Cmd + Shift + V: Paste
```

**3. Terminal Persistence**:
```typescript
interface TerminalSession {
  id: string;
  toolId?: string;
  buffer: string;
  cwd: string;
  env: Record<string, string>;
  timestamp: Date;
}

// Save session on close
saveTerminalSession(terminal: TerminalInstance): void {
  const session: TerminalSession = {
    id: terminal.id,
    toolId: terminal.toolId,
    buffer: terminal.xterm.serialize(),
    cwd: terminal.pty.process.cwd(),
    env: terminal.pty.process.env,
    timestamp: new Date()
  };
  
  localStorage.setItem(`terminal-session-${terminal.id}`, JSON.stringify(session));
}

// Restore on restart
restoreTerminalSessions(): void {
  const sessions = this.loadSavedSessions();
  sessions.forEach(session => {
    if (session.toolId) {
      // Restore AI tool terminal
      this.createAIToolTerminal(session.toolId, getToolName(session.toolId));
    }
  });
}
```

#### UI/UX Enhancements

**1. Tool Status Indicators**:
- 🟢 Green dot: Tool running
- 🟡 Yellow dot: Tool starting
- 🔴 Red dot: Tool error
- ⚫ Gray dot: Tool stopped

**2. Tab Overflow Handling**:
```
When tabs exceed width:
[<] Console | Claude | Gemini | ... | Terminal 2 [>] [+]
    ^ Scroll arrows appear
```

**3. Split Terminal View**:
```
┌─────────────────┬─────────────────┐
│ Claude          │ Gemini          │
│                 │                 │
│ > processing... │ > ready         │
└─────────────────┴─────────────────┘
```

#### Implementation Status & Next Steps

**🔄 TRANSITIONING TO TTYD ARCHITECTURE**

**Why We're Moving to TTYD**:
1. **Claude Code Compatibility**: Perfect rendering of TUI applications without % characters or duplicate UI
2. **Zero Maintenance**: Battle-tested terminal server used in production environments
3. **True Terminal Emulation**: Real PTY processes, not JavaScript emulation
4. **Native Performance**: Direct terminal I/O without translation overhead
5. **Simplified Codebase**: Remove complex xterm.js workarounds and rendering hacks

**TTYD Implementation Advantages**:
- **Perfect TUI Support**: vim, tmux, Claude Code, and all cursor-based apps work flawlessly
- **Isolation**: Each tab runs in its own ttyd instance with separate port
- **Security**: WebView sandboxing with no nodeintegration
- **Seamless Integration**: Appears as native tabs within the IDE
- **Auto-Command Execution**: Can send commands directly to terminal on launch

**Current Implementation Files (To Be Updated)**:
- `src/terminal-ipc-handlers.ts` → Will manage ttyd processes instead of node-pty
- `src/components/IsolatedTerminalPanel.ts` → Will embed webviews instead of xterm.js
- `src/preload.ts` → Simplified IPC for ttyd management
- `webpack.main.config.ts` → Remove xterm dependencies, add ttyd management

**TTYD Installation & Setup Requirements**:

```bash
# macOS Installation
brew install ttyd

# Linux Installation
sudo apt-get install ttyd  # Debian/Ubuntu
sudo yum install ttyd       # RHEL/CentOS

# Windows Installation
# Download binary from https://github.com/tsl0922/ttyd/releases
```

**TTYD Integration Steps**:

1. **Install ttyd binary** (can be bundled with app or installed separately)
2. **Port Management Service**: Create service to allocate ports for each terminal
3. **WebView Implementation**: Replace xterm.js divs with webview elements
4. **Process Management**: Spawn and manage ttyd processes per tab
5. **Command Injection**: Send initial commands to terminals for AI tools
6. **Cleanup**: Properly terminate ttyd processes when tabs close

**🚧 IN PROGRESS - Implementation Phases**

**Phase 1: TTYD Foundation** (Priority 1 - IMMEDIATE)
- Install and bundle ttyd with application
- Create TTYDManager service for process management
- Implement port allocation system (7000-8000 range)
- Replace terminal div containers with webview elements
- Test basic terminal creation and display

**Phase 2: AI Tool Integration** (Priority 2)
- Connect "Launch" buttons to create named terminal tabs
- Auto-execute tool commands (claude, gemini, etc.) via ttyd
- Tool-specific tab icons and names
- Working directory management for tool context

**Phase 3: Terminal Enhancements** (Priority 3)
- WebView resize handling (automatic with ttyd)
- Copy/paste support (native in ttyd terminals)
- Search within terminal output (browser find functionality)
- Terminal themes via ttyd CSS customization

**Phase 4: Professional Features** (Priority 4)
- Terminal session persistence (ttyd reconnection)
- Context menus for tabs (restart, clear, reload)
- Keyboard shortcuts (Ctrl+T new, Ctrl+W close)
- Split terminal view with multiple webviews
- Custom ttyd themes and fonts

**Phase 5: Stability & Polish** (Priority 5)
- Error recovery for crashed ttyd instances
- Health checks for ttyd processes
- Automatic port reclamation
- Debug logging for ttyd management

**Key Benefits of TTYD Over xterm.js**:

| Feature | xterm.js | TTYD |
|---------|----------|------|
| **Claude Code Rendering** | ❌ Broken (% chars, duplicate UI) | ✅ Perfect |
| **vim/tmux Support** | ⚠️ Limited | ✅ Full |
| **Maintenance Burden** | 🔴 High (constant fixes) | 🟢 Low |
| **True Terminal** | ❌ JavaScript emulation | ✅ Real PTY |
| **Performance** | ⚠️ JavaScript overhead | ✅ Native |
| **TUI Applications** | ❌ Many issues | ✅ All work |
| **Cursor Control** | ⚠️ Buggy | ✅ Perfect |
| **ANSI Support** | ⚠️ Partial | ✅ Complete |

#### Configuration Options

```typescript
interface TerminalSettings {
  defaultLocation: 'integrated' | 'external';
  fontSize: number;
  fontFamily: string;
  theme: 'dark' | 'light' | 'custom';
  scrollback: number;
  cursorStyle: 'block' | 'bar' | 'underline';
  cursorBlink: boolean;
  confirmCloseRunningTool: boolean;
  restoreTerminalsOnStartup: boolean;
  maxTabs: number;
}
```

#### Benefits of This Approach

1. **Unified Workspace**: All AI tools in one place
2. **Quick Switching**: Tab-based navigation between tools
3. **Parallel Usage**: Run multiple AI assistants simultaneously
4. **Visual Clarity**: Named tabs with icons for easy identification
5. **Flexibility**: Mix AI tools with regular terminals
6. **Persistence**: Restore terminal sessions across restarts
7. **Professional Feel**: IDE-grade terminal experience

### Installation Flow

#### Pre-Installation Checks
1. **Dependency Detection**
   ```typescript
   async checkDependencies(tool: CliTool): Promise<DependencyStatus> {
     // Check Node.js version for npm tools
     // Check Python version for pip tools
     // Check git for extension tools
     return { satisfied: boolean, missing: string[] };
   }
   ```

2. **Permission Verification**
   - Check write permissions to installation directory
   - Detect if sudo/admin required
   - Offer local installation alternative

---

## Global Folder Management System

### Overview
The Global Folder Management System provides a unified project context across all IDE components, ensuring that File Explorer, Source Control, Terminal, and AI Tools all work with the same project folder. This creates a cohesive VS Code-like experience where the entire application is aware of and synchronized with the current project.

### Architecture

#### Core State Management
```typescript
// Global state variable in src/renderer.ts
let currentOpenedFolder: string | null = null;

// Exposed to window for global access
(window as any).currentOpenedFolder = currentOpenedFolder;
```

#### Folder State Flow
```
User Action → Main Process → IPC Event → Renderer Handler → Update Global State → Notify All Components
```

### Folder Operations

#### 1. Opening a Folder
**Trigger Points**:
- Menu: File > Open Folder
- Welcome screen: "Open Folder" button  
- AI Tool Launch: Automatic via handleOpenFolder
- Command: `window.openFolder(path?)` - accepts optional path

**Process Flow**:
```typescript
// Main Process (src/index.ts)
Menu/Dialog → dialog.showOpenDialog() → mainWindow.webContents.send('menu-open-folder', path)

// Renderer Process (src/renderer.ts)
// Fixed: window.openFolder now accepts optional path parameter
window.openFolder = async (folderPath?: string) => {
  if (folderPath) {
    handleOpenFolder(folderPath);  // Direct open without dialog
  } else {
    // Show dialog only if no path provided
    const result = await showOpenDialog();
    if (result.filePaths[0]) handleOpenFolder(result.filePaths[0]);
  }
}

// Central handler for all folder opens
handleOpenFolder(folderPath) → {
  1. Update currentOpenedFolder = folderPath
  2. Update window.currentOpenedFolder
  3. Update window title
  4. Initialize Git: gitAPI.setFolder(folderPath)
  5. Create/refresh File Explorer
  6. Update Source Control view (no more welcome screen)
  7. Update status bar (git branch)
  8. Setup file selection handlers
}
```

#### 2. Closing a Folder
**Trigger Points**:
- Menu: File > Close Folder
- Command: `handleCloseFolder()`

**Process Flow**:
```typescript
// Main Process
Menu → mainWindow.webContents.send('menu-close-folder')

// Renderer Process
electronAPI.onMenuCloseFolder() → handleCloseFolder() → {
  1. Clear currentOpenedFolder = null
  2. Clear window.currentOpenedFolder
  3. Reset window title
  4. Clear File Explorer
  5. Reset Source Control
  6. Show welcome screen
}
```

### Component Integration

#### File Explorer
```typescript
// Uses global folder as root
window.fileExplorer.initialize(currentOpenedFolder);

// File selections are relative to this folder
fileExplorer.onFileSelect((filePath) => {
  // filePath is relative to currentOpenedFolder
  editorTabs.openFile(filePath);
});

// Explorer always respects currentOpenedFolder
// When reopened, checks if current path matches global folder
// Updates if necessary via getCurrentPath() method
```

#### Source Control (Git)
```typescript
// Git operations scoped to current folder
gitAPI.setFolder(currentOpenedFolder);

// Status bar shows branch for current folder
updateGitStatusBar(); // Uses currentOpenedFolder

// Source Control properly handles both window.gitUI and window.scmView
// Ensures refresh button works by maintaining both references
window.gitUI = new VSCodeSCMView(container);
window.scmView = window.gitUI; // Both references needed for proper operation
```

#### Terminal System
```typescript
// New terminals use current folder as working directory
createTerminalTab(toolId, command) {
  cwd: window.currentOpenedFolder || process.env.HOME
}
```

#### AI Tools Launch Integration
```typescript
// Launch button automatically sets global folder
ipcMain.handle('cli-tool-launch', async (_, toolId) => {
  // 1. Show folder selection dialog
  const selectedPath = await selectFolder();
  
  // 2. Check launch history for resume detection
  const aiToolsDb = AIToolsDatabase.getInstance();
  const hasBeenLaunched = aiToolsDb.hasBeenLaunchedBefore(toolId, selectedPath);
  const command = hasBeenLaunched ? 'claude --resume' : 'claude';
  
  // 3. FIRST: Update global folder context
  // This triggers handleOpenFolder() which updates:
  // - File Explorer, Source Control, Status Bar
  mainWindow.webContents.send('menu-open-folder', selectedPath);
  
  // 4. THEN: Launch tool in that folder
  setTimeout(() => {
    mainWindow.webContents.send('launch-ai-tool-terminal', {
      toolId, command, cwd: selectedPath
    });
  }, 100); // Ensures folder context is set first
  
  // 5. Record launch for future resume detection
  aiToolsDb.recordLaunch(toolId, selectedPath);
});
```

### IPC Communication

#### Events
```typescript
// Main → Renderer Events
'menu-open-folder'         // Opens a folder (from menu or AI tool launch)
'menu-close-folder'        // Closes current folder
'menu-open-file'           // Opens a specific file
'menu-new-file'            // Creates new file
'menu-save'                // Saves current file
'menu-save-as'             // Save with new name
'launch-ai-tool-terminal'  // Creates terminal tab with AI tool

// Preload Bridge (src/preload.ts)
electronAPI: {
  onMenuOpenFolder: (callback: (path: string) => void) => void;
  onMenuCloseFolder: (callback: () => void) => void;
  onMenuOpenFile: (callback: (path: string) => void) => void;
  onLaunchAIToolTerminal: (callback: (data: {
    toolId: string;
    command: string;
    cwd: string;
  }) => void) => void;
  // ... other handlers
}
```

### State Synchronization

#### Automatic Updates
When the global folder changes, these components automatically update:

1. **Window Title**: Shows project name
2. **File Explorer**: Displays folder tree
3. **Source Control**: Shows git status
4. **Status Bar**: Updates branch name
5. **Terminal**: New terminals use folder as cwd
6. **Editor**: File paths resolve relative to folder

#### Event Listeners
```typescript
// Components watch for folder changes
window.electronAPI.onMenuOpenFolder((folderPath) => {
  // Each component updates accordingly
  handleOpenFolder(folderPath);
});
```

### Benefits

1. **Single Source of Truth**: One variable manages all folder context
2. **Automatic Synchronization**: All components stay in sync
3. **Consistent UX**: VS Code-like project management
4. **Clean Architecture**: Clear separation of concerns
5. **Extensible**: Easy to add new folder-aware components

### Usage Examples

#### Opening Project via AI Tool Launch (Complete Flow)
```typescript
// User clicks Launch button → Selects folder → IDE updates
1. Launch button clicked in AI Tools Manager
2. Folder selection dialog appears
3. User selects project directory
4. Database checks for previous launches
5. Global folder context updated via 'menu-open-folder' event:
   - handleOpenFolder() called
   - currentOpenedFolder = selectedPath
   - Window title updates to show project name
   - File Explorer initializes with folder tree
   - Git integration sets folder context
   - Source Control view refreshes
   - Status bar shows git branch
6. Terminal tab created with appropriate command:
   - First time: 'claude'
   - Subsequent: 'claude --resume'
7. Launch recorded in database for future sessions
8. All IDE components now synchronized on project
```

#### Switching Projects
```typescript
// File > Open Folder → Select new project
1. Menu action triggered
2. New folder selected
3. All components refresh with new context
4. Previous project state cleared
5. New project fully loaded
```

---

## AI Tools Launch System with Resume Detection

### Overview
The AI Tools Launch System provides intelligent per-repository tracking for AI development tools, automatically detecting when to use resume flags based on launch history. This ensures seamless continuation of work across sessions.

### Architecture Components

#### 1. AIToolsDatabase Service
**Location**: `src/services/AIToolsDatabase.ts`
**Database**: `~/.hive/hive-ai.db`

```typescript
export class AIToolsDatabase {
  private db: Database.Database;
  private static instance: AIToolsDatabase | null = null;
  
  // Singleton pattern for global access
  public static getInstance(): AIToolsDatabase {
    if (!AIToolsDatabase.instance) {
      AIToolsDatabase.instance = new AIToolsDatabase();
    }
    return AIToolsDatabase.instance;
  }
  
  // Core methods for launch tracking
  hasBeenLaunchedBefore(toolId: string, repositoryPath: string): boolean;
  recordLaunch(toolId: string, repositoryPath: string, metadata?: any): boolean;
  getLaunchInfo(toolId: string, repositoryPath: string): AIToolLaunch | null;
  getRepositoryLaunches(repositoryPath: string): AIToolLaunch[];
  getUsageStats(): UsageStatistics;
}
```

#### 2. Database Schema
```sql
CREATE TABLE ai_tool_launches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  tool_id TEXT NOT NULL,                -- 'claude-code', 'gemini-cli', etc.
  repository_path TEXT NOT NULL,        -- Absolute path to project
  launch_count INTEGER DEFAULT 1,       -- Number of launches
  first_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
  last_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
  status TEXT DEFAULT 'active',         -- 'active', 'closed', 'crashed'
  session_metadata TEXT,                 -- JSON session data
  user_id TEXT DEFAULT 'default',
  tool_version TEXT,
  launch_context TEXT,
  UNIQUE(tool_id, repository_path, user_id)
);

-- Performance indexes
CREATE INDEX idx_ai_tool_launches_lookup 
  ON ai_tool_launches(tool_id, repository_path);
CREATE INDEX idx_ai_tool_launches_recent 
  ON ai_tool_launches(last_launched_at DESC);
```

#### 3. IPC Communication Flow
```typescript
// Main Process (src/index.ts)
ipcMain.handle('cli-tool-launch', async (_, toolId: string) => {
  // 1. Show folder selection dialog
  const { canceled, filePaths } = await dialog.showOpenDialog({
    title: `Select folder to launch ${getToolName(toolId)}`,
    properties: ['openDirectory'],
    defaultPath: process.env.HOME
  });
  
  if (canceled || !filePaths.length) return;
  
  const selectedPath = filePaths[0];
  
  // 2. Check launch history
  const hasBeenLaunched = aiToolsDb.hasBeenLaunchedBefore(toolId, selectedPath);
  
  // 3. Determine command
  const command = hasBeenLaunched ? 
    getResumeCommand(toolId) : 
    getLaunchCommand(toolId);
  
  // 4. Send to renderer for terminal creation
  mainWindow.webContents.send('launch-ai-tool-terminal', {
    toolId,
    toolName: getToolName(toolId),
    command,
    cwd: selectedPath
  });
  
  // 5. Record launch for future resume detection
  aiToolsDb.recordLaunch(toolId, selectedPath, {
    version: await getToolVersion(toolId),
    context: { source: 'ui-launch-button' }
  });
});

// Renderer Process (src/renderer.ts)
window.electronAPI.onLaunchAIToolTerminal((data) => {
  const terminal = window.isolatedTerminal;
  if (terminal) {
    // Create named terminal tab with command
    terminal.createTerminalTab(data.toolId, data.command, data.cwd);
  }
});
```

#### 4. Resume Detection Logic
```typescript
const RESUME_COMMANDS = {
  'claude-code': {
    initial: 'claude',
    resume: 'claude --resume'
  },
  'gemini-cli': {
    initial: 'gemini',
    resume: 'gemini --continue'
  },
  'qwen-code': {
    initial: 'qwen',
    resume: 'qwen --restore'
  },
  'aider': {
    initial: 'aider',
    resume: 'aider --yes-always'
  },
  'cline': {
    initial: 'cline',
    resume: 'cline --resume-session'
  }
};
```

### Launch Button UI Integration

#### Button States
```typescript
interface LaunchButtonState {
  idle: {
    text: '🚀 Launch',
    class: 'btn-primary',
    enabled: true
  },
  launching: {
    text: '⏳ Launching...',
    class: 'btn-primary disabled',
    enabled: false
  },
  error: {
    text: '❌ Launch Failed',
    class: 'btn-danger',
    enabled: true
  }
}
```

#### Visual Feedback Flow
1. **User clicks Launch** → Button shows "⏳ Launching..."
2. **Folder dialog opens** → User selects project directory
3. **Database check** → Determines if resume flag needed
4. **Terminal tab opens** → Named by tool (Claude, Gemini, etc.)
5. **Command executes** → With 0.5s delay for shell readiness
6. **Button returns to idle** → Ready for next launch

### Benefits of Resume Detection

1. **Context Preservation**: AI tools can resume previous conversations
2. **Efficiency**: No need to re-explain project context
3. **Automatic**: Users don't need to remember resume flags
4. **Per-Repository**: Each project maintains its own history
5. **Cross-Session**: Works across app restarts

### Integration Benefits

1. **Unified Experience**: Launch button serves dual purpose:
   - Opens project in IDE (Explorer, Git, etc.)
   - Launches AI tool with proper context

2. **Smart Detection**: Database tracking ensures:
   - First launch uses base command
   - Subsequent launches use resume flag
   - Per-repository granularity

3. **Complete IDE Sync**: Single action updates:
   - File Explorer with project tree
   - Source Control with git status
   - Status Bar with branch info
   - Terminal with AI tool
   - Window title with project name

### Debugging and Logging

```typescript
// Enhanced logging for troubleshooting
logger.info(`[Main] Database check - Tool: ${toolId}, Path: ${selectedPath}, Previously launched: ${hasBeenLaunched}`);
logger.info(`[Main] Sending menu-open-folder event for: ${selectedPath}`);
logger.info(`[Main] Sending launch-ai-tool-terminal event with command: ${command}`);

// Renderer side logging
console.log('[handleOpenFolder] Opening folder:', folderPath);
console.log('[handleOpenFolder] Previous folder:', currentOpenedFolder);
```

---

## Installation Detection & Dynamic UI Management

### Overview
The CLI Tools panel dynamically detects installed tools and updates the UI accordingly, providing real-time status updates and appropriate action buttons based on each tool's installation state.

### Implementation Details (Completed)

#### Claude Code Integration with Resume Detection
As of version 1.6.0, full Claude Code CLI integration has been implemented with intelligent per-repository launch tracking and resume detection:

1. **Real-time Detection**: 
   - Detects Claude Code installation via `claude --version` command
   - Parses version from output (e.g., "1.0.86 (Claude Code)")
   - Shows installation path (`which claude`)

2. **Launch Button with Resume Detection**:
   - **Launch** (Blue): Opens folder selector and launches Claude in new terminal tab
   - **First Launch**: Executes `claude` command in selected folder
   - **Subsequent Launches**: Automatically adds `--resume` flag for same folder
   - **Per-Repository Tracking**: Database tracks launches by repository path
   - **Smart Terminal Naming**: AI tool tabs named by tool (Claude, Gemini, etc.)

3. **AI Tools Database Architecture**:
   ```typescript
   // Database: ~/.hive/hive-ai.db
   interface AIToolLaunch {
     tool_id: string;           // 'claude-code', 'gemini-cli', etc.
     repository_path: string;    // Absolute path to project folder
     launch_count: number;       // Times launched in this repository
     first_launched_at: string;  // ISO timestamp of first launch
     last_launched_at: string;   // ISO timestamp of last launch
     status: 'active' | 'closed' | 'crashed';
     session_metadata?: string;  // JSON data about session
   }
   ```

4. **Complete Terminal Launch Flow**:
   ```typescript
   // 1. User clicks Launch button
   await electronAPI.launchCliTool('claude-code');
   
   // 2. Main process shows folder dialog
   const { canceled, filePaths } = await dialog.showOpenDialog({
     properties: ['openDirectory'],
     title: 'Select folder to launch Claude Code',
     buttonLabel: 'Launch Here'
   });
   
   // 3. Check launch history in database
   const hasBeenLaunched = aiToolsDb.hasBeenLaunchedBefore(
     'claude-code', selectedPath
   );
   logger.info(`Database check - Previously launched: ${hasBeenLaunched}`);
   
   // 4. Determine command based on history
   const command = hasBeenLaunched ? 'claude --resume' : 'claude';
   
   // 5. CRITICAL: Update global folder context FIRST
   // This ensures Explorer, Git, and Status Bar all update
   mainWindow.webContents.send('menu-open-folder', selectedPath);
   
   // 6. After delay, send terminal creation event
   setTimeout(() => {
     mainWindow.webContents.send('launch-ai-tool-terminal', {
       toolId: 'claude-code',
       toolName: 'Claude',
       command: command,
       cwd: selectedPath
     });
   }, 100); // Ensures folder context is set first
   
   // 7. Record launch in database for future resume detection
   aiToolsDb.recordLaunch('claude-code', selectedPath);
   ```

5. **Functional Buttons**:
   - **Launch**: Folder selection → Resume detection → Terminal creation
   - **Details** (Green): Refreshes and displays full tool status
   - **Configure**: Registers with Memory Service, updates MCP config
   - **Update**: Executes `npm update -g @anthropic-ai/claude-code`
   - **Docs**: Opens official Claude Code documentation

6. **Memory Service MCP Integration**:
   - Automatically creates MCP server configuration in `~/.claude/.mcp.json`
   - Generates MCP wrapper script at `~/.hive/memory-service-mcp-wrapper.js`
   - Unique authentication tokens per tool for security
   - Exposes `query_memory` and `contribute_learning` MCP tools

7. **Visual Feedback**:
   - "🚀 Launching..." → Terminal tab opens with tool name
   - "⚙️ Configuring..." → "✅ Configured"
   - "⬆️ Updating..." → "✅ Up to date"
   - "🔄 Loading details..." → Full status display

8. **Persistent Configuration**:
   - Tool status saved in `~/.hive/cli-tools-config.json`
   - Launch history saved in `~/.hive/hive-ai.db`
   - Memory Service tokens stored securely
   - MCP configuration persists across sessions

#### Technical Implementation
- **Manager**: `CliToolsManager` class (singleton pattern)
- **Detection**: `cli-tool-detector.ts` with exec-based version checking
- **IPC**: Main/renderer communication via Electron IPC
- **UI Updates**: Dynamic DOM manipulation with data attributes
- **Error Handling**: Try-catch blocks with user-friendly error messages

#### Memory Service Connection Detection
The `checkMemoryServiceConnection` method in `cli-tool-detector.ts`:
```typescript
private async checkMemoryServiceConnection(toolId: string): Promise<boolean> {
  // 1. Read cli-tools-config.json for existing token
  const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
  const token = config[toolId]?.memoryService?.token;
  
  // 2. Validate token with Memory Service
  const req = http.request({
    hostname: 'localhost',
    port: memoryServicePort,
    path: '/api/v1/memory/stats',
    headers: {
      'Authorization': `Bearer ${token}`,
      'X-Client-Name': toolId
    }
  });
  
  // 3. Return true if service responds (200 or 401)
  return res.statusCode === 200 || res.statusCode === 401;
}
```

#### Configure Button Implementation
The complete flow in `cli-tool-configure` IPC handler:

1. **Get Memory Service Port**:
   ```typescript
   const memoryServiceInfo = processManager.getProcessStatus('memory-service');
   const memoryServicePort = memoryServiceInfo?.port || 3457;
   ```

2. **Register with Memory Service**:
   ```typescript
   POST /api/v1/memory/register
   Body: { toolName: 'claude-code' }
   Response: { token: '86a752ef...' }
   ```

3. **Save Configuration**:
   ```json
   // ~/.hive/cli-tools-config.json
   {
     "claude-code": {
       "memoryService": {
         "endpoint": "http://localhost:3457",
         "token": "86a752ef...",
         "connectedAt": "2025-08-23T02:27:55.368Z"
       }
     }
   }
   ```

4. **Generate MCP Wrapper Script**:
   - Creates Node.js script at `~/.hive/memory-service-mcp-wrapper.js`
   - Implements MCP Server with `query_memory` and `contribute_learning` tools
   - Uses environment variables for endpoint and token

5. **Update MCP Configuration**:
   ```json
   // ~/.claude/.mcp.json
   {
     "servers": {
       "hive-memory-service": {
         "command": "node",
         "args": ["/Users/.../.hive/memory-service-mcp-wrapper.js"],
         "env": {
           "MEMORY_SERVICE_ENDPOINT": "http://localhost:3457",
           "MEMORY_SERVICE_TOKEN": "86a752ef..."
         }
       }
     }
   }
   ```

### Installation Detection System

#### Detection Methods
```typescript
interface ToolDetector {
  detectInstallation(tool: CliTool): Promise<InstallationInfo>;
  watchForChanges(tool: CliTool, callback: (status: InstallationStatus) => void): void;
  getVersion(tool: CliTool): Promise<string | null>;
  getExecutablePath(tool: CliTool): Promise<string | null>;
}

interface InstallationInfo {
  installed: boolean;
  version?: string;
  path?: string;
  lastUpdated?: Date;
  updateAvailable?: boolean;
  memoryServiceConnected?: boolean;
}
```

#### Detection Strategies by Tool Type

**1. NPM-based Tools (Claude Code, Gemini CLI, Qwen Code, Cline)**
```typescript
async detectNpmTool(packageName: string): Promise<InstallationInfo> {
  // Check global npm packages
  const globalCheck = await exec('npm list -g --depth=0 ' + packageName);
  
  // Check local project packages
  const localCheck = await exec('npm list --depth=0 ' + packageName);
  
  // Check PATH for executable
  const pathCheck = await exec('which ' + toolExecutable);
  
  return {
    installed: globalCheck.success || localCheck.success || pathCheck.success,
    version: extractVersion(globalCheck.output),
    path: pathCheck.output,
    updateAvailable: await checkNpmUpdates(packageName)
  };
}
```

**2. Python-based Tools (Aider)**
```typescript
async detectPythonTool(packageName: string): Promise<InstallationInfo> {
  // Check pip installations
  const pipCheck = await exec('pip show ' + packageName);
  
  // Check pipx installations
  const pipxCheck = await exec('pipx list | grep ' + packageName);
  
  // Check virtual environments
  const venvCheck = await checkVirtualEnvs(packageName);
  
  return {
    installed: pipCheck.success || pipxCheck.success || venvCheck.found,
    version: extractPipVersion(pipCheck.output),
    path: findPythonExecutable(packageName)
  };
}
```

### Dynamic UI State Management

#### State Management Architecture
```typescript
interface CliToolsState {
  tools: Map<string, ToolState>;
  refreshInterval: number;
  autoDetect: boolean;
}

interface ToolState {
  id: string;
  name: string;
  installed: boolean;
  version?: string;
  status: 'not-installed' | 'installed' | 'installing' | 'updating' | 'error';
  buttons: ButtonConfig[];
  lastChecked: Date;
}

interface ButtonConfig {
  label: string;
  action: 'install' | 'update' | 'configure' | 'uninstall' | 'docs';
  enabled: boolean;
  variant: 'primary' | 'secondary' | 'danger';
}
```

#### Button State Logic
```typescript
function getButtonsForTool(toolState: ToolState): ButtonConfig[] {
  const buttons: ButtonConfig[] = [];
  
  if (!toolState.installed) {
    // Not installed - show Install and Docs
    buttons.push({
      label: 'Install',
      action: 'install',
      enabled: true,
      variant: 'primary'
    });
    buttons.push({
      label: 'Docs',
      action: 'docs',
      enabled: true,
      variant: 'secondary'
    });
  } else {
    // Installed - show Configure, Update (if available), and Docs
    buttons.push({
      label: 'Configure',
      action: 'configure',
      enabled: true,
      variant: 'secondary'
    });
    
    if (toolState.updateAvailable) {
      buttons.push({
        label: 'Update',
        action: 'update',
        enabled: true,
        variant: 'primary'
      });
    }
    
    buttons.push({
      label: 'Docs',
      action: 'docs',
      enabled: true,
      variant: 'secondary'
    });
  }
  
  return buttons;
}
```

### Real-time Status Updates

#### Polling System
```typescript
class ToolStatusPoller {
  private intervals: Map<string, NodeJS.Timer> = new Map();
  
  startPolling(toolId: string, intervalMs: number = 30000) {
    const interval = setInterval(async () => {
      const status = await detectInstallation(toolId);
      updateUIState(toolId, status);
    }, intervalMs);
    
    this.intervals.set(toolId, interval);
  }
  
  stopPolling(toolId: string) {
    const interval = this.intervals.get(toolId);
    if (interval) {
      clearInterval(interval);
      this.intervals.delete(toolId);
    }
  }
}
```

#### File System Watchers
```typescript
class InstallationWatcher {
  private watchers: Map<string, FSWatcher> = new Map();
  
  watchInstallationPaths(tool: CliTool) {
    const pathsToWatch = [
      '/usr/local/lib/node_modules',  // Global npm
      '~/.npm-global',                 // User npm global
      '/usr/local/bin',                // Binary installations
      '~/.local/bin',                  // User binaries
      './node_modules'                 // Local project
    ];
    
    pathsToWatch.forEach(path => {
      const watcher = fs.watch(path, (event, filename) => {
        if (filename?.includes(tool.packageName)) {
          this.handleInstallationChange(tool);
        }
      });
      
      this.watchers.set(`${tool.id}-${path}`, watcher);
    });
  }
}
```

### UI Update Flow

#### Update Sequence
1. **Initial Load**
   - Scan all tools for installation status
   - Render cards with appropriate buttons
   - Start polling for changes

2. **Status Change Detection**
   - File system watcher triggers
   - Or polling interval completes
   - Detection system checks new status

3. **UI State Update**
   - Compare new status with current state
   - Update only changed elements
   - Animate transitions smoothly

4. **Button Configuration**
   - Recalculate available actions
   - Enable/disable buttons
   - Update tooltips and labels

#### React Component Example
```tsx
function CliToolCard({ tool }: { tool: CliTool }) {
  const [state, setState] = useState<ToolState>(null);
  
  useEffect(() => {
    // Initial detection
    detectInstallation(tool).then(setState);
    
    // Set up polling
    const interval = setInterval(() => {
      detectInstallation(tool).then(setState);
    }, 30000);
    
    return () => clearInterval(interval);
  }, [tool]);
  
  const buttons = getButtonsForTool(state);
  
  return (
    <Card>
      <CardHeader>
        <h3>{tool.name}</h3>
        <StatusIndicator status={state.status} />
      </CardHeader>
      <CardBody>
        {state.installed && <Version>{state.version}</Version>}
        <Description>{tool.description}</Description>
      </CardBody>
      <CardFooter>
        {buttons.map(button => (
          <Button
            key={button.action}
            variant={button.variant}
            onClick={() => handleAction(button.action, tool)}
            disabled={!button.enabled}
          >
            {button.label}
          </Button>
        ))}
      </CardFooter>
    </Card>
  );
}
```

### Error Handling

#### Detection Failures
```typescript
interface DetectionError {
  tool: string;
  error: Error;
  fallbackAction: 'assume-not-installed' | 'use-cached' | 'show-error';
}

async function safeDetection(tool: CliTool): Promise<InstallationInfo> {
  try {
    return await detectInstallation(tool);
  } catch (error) {
    console.error(`Detection failed for ${tool.name}:`, error);
    
    // Try cached status
    const cached = getCachedStatus(tool.id);
    if (cached && Date.now() - cached.timestamp < 3600000) {
      return cached.status;
    }
    
    // Default to not installed
    return { installed: false };
  }
}
```

### Performance Optimization

#### Caching Strategy
```typescript
class DetectionCache {
  private cache: Map<string, CachedStatus> = new Map();
  private readonly TTL = 5 * 60 * 1000; // 5 minutes
  
  get(toolId: string): InstallationInfo | null {
    const cached = this.cache.get(toolId);
    if (!cached) return null;
    
    if (Date.now() - cached.timestamp > this.TTL) {
      this.cache.delete(toolId);
      return null;
    }
    
    return cached.status;
  }
  
  set(toolId: string, status: InstallationInfo) {
    this.cache.set(toolId, {
      status,
      timestamp: Date.now()
    });
  }
}
```

#### Batch Detection
```typescript
async function detectAllTools(): Promise<Map<string, InstallationInfo>> {
  const tools = getAllTools();
  
  // Parallel detection with concurrency limit
  const results = await pLimit(3)(
    tools.map(tool => () => detectInstallation(tool))
  );
  
  return new Map(tools.map((tool, i) => [tool.id, results[i]]));
}

3. **Network Connectivity**
   - Test package registry access
   - Estimate download size
   - Show expected installation time

#### Installation Process
```typescript
interface InstallationStage {
  stage: 'preparing' | 'downloading' | 'installing' | 'configuring' | 'verifying';
  progress: number;
  message: string;
  eta?: number;
}

// Real-time updates via IPC
ipcRenderer.on('cli-tool-progress', (event, update: InstallationStage) => {
  updateProgressBar(update);
  showStatusMessage(update.message);
});
```

#### Post-Installation
1. **Verification**
   - Run version command
   - Test basic functionality
   - Verify PATH accessibility

2. **Configuration**
   - Auto-configure Memory Service integration
   - Set up authentication if needed
   - Create shell aliases if requested

3. **Documentation**
   - Show quick start guide
   - Display common commands
   - Link to full documentation

### Advanced Options Panel

#### Settings Categories
```typescript
interface AdvancedSettings {
  installation: {
    autoInstallRecommended: boolean;
    useLocalNpm: boolean;
    usePipx: boolean;
    customInstallPath?: string;
  };
  updates: {
    autoCheck: boolean;
    autoInstall: boolean;
    checkInterval: number; // hours
    includePrerelease: boolean;
  };
  integration: {
    memoryServiceAutoConnect: boolean;
    shareUsageAnalytics: boolean;
    enableExperimentalFeatures: boolean;
  };
}
```

#### Per-Tool Configuration
- Custom environment variables
- Model selection preferences
- Rate limiting settings
- API endpoint overrides

### Authentication Management

#### Auth Status Display
```typescript
type AuthStatus = 
  | { type: 'none' }
  | { type: 'required'; instructions: string }
  | { type: 'configured'; validUntil?: Date }
  | { type: 'invalid'; error: string };
```

#### Auth Configuration Flow
1. **Detect auth requirement**
2. **Show setup instructions**
3. **Provide copy-to-clipboard commands**
4. **Verify authentication**
5. **Store credentials securely**

### Error Handling & Recovery

#### Common Error Scenarios
```typescript
const ERROR_HANDLERS = {
  NETWORK_ERROR: {
    message: 'Unable to download package',
    actions: ['Retry', 'Use Proxy', 'Download Manually']
  },
  PERMISSION_DENIED: {
    message: 'Installation requires elevated permissions',
    actions: ['Use Local Install', 'Run as Admin', 'Change Directory']
  },
  DEPENDENCY_MISSING: {
    message: 'Required dependency not found',
    actions: ['Install Dependency', 'Use Alternative', 'Skip']
  }
};
```

#### Recovery Options
- Automatic retry with exponential backoff
- Alternative installation methods
- Manual installation guides
- Direct support links

### Performance Optimization

#### Lazy Loading
- Load tool cards on scroll
- Defer non-visible content rendering
- Cache tool metadata locally

#### Background Operations
- Non-blocking installations
- Parallel update checks
- Queue management for batch operations

#### Resource Management
```typescript
const RESOURCE_LIMITS = {
  maxConcurrentInstalls: 2,
  maxDownloadSpeed: undefined, // No limit by default
  diskSpaceBuffer: 500 * 1024 * 1024, // 500MB
  memoryLimit: 100 * 1024 * 1024, // 100MB per operation
};
```

### Accessibility Features

#### Keyboard Navigation
- Tab through tool cards
- Enter to install/configure
- Space to select for batch operations
- Escape to cancel operations

#### Screen Reader Support
- ARIA labels for all interactive elements
- Status announcements for operations
- Descriptive alt text for icons

#### Visual Accessibility
- High contrast mode support
- Colorblind-friendly status indicators
- Adjustable text size
- Reduced motion options

### Analytics & Telemetry

#### Usage Metrics (Privacy-Respecting)
```typescript
interface ToolUsageMetrics {
  toolId: string;
  installedAt: Date;
  lastUsed?: Date;
  updateCount: number;
  errorCount: number;
}
```

#### Aggregated Statistics
- Most popular tools
- Average installation success rate
- Common error patterns
- Update adoption rates

### Future UI Enhancements

1. **Tool Marketplace**
   - Community-contributed tool definitions
   - User ratings and reviews
   - Curated collections

2. **Workspace Profiles**
   - Save tool configurations per project
   - Quick switch between setups
   - Team sharing capabilities

3. **Interactive Tutorials**
   - Guided setup wizards
   - Interactive command previews
   - Video tutorials embedded

4. **AI-Powered Recommendations**
   - Suggest tools based on project type
   - Recommend configurations
   - Predict useful tool combinations

---

## Logging System

### SafeLogger Architecture
**Location**: `src/utils/SafeLogger.ts`

The SafeLogger system provides production-ready logging with automatic EPIPE error handling, essential for Electron apps with child processes using `stdio: 'inherit'`.

#### Key Features:
- **EPIPE Error Handling**: Gracefully handles pipe errors without crashing
- **Log Rotation**: Automatic rotation at 10MB with 5 file retention
- **Log Levels**: DEBUG, INFO, WARN, ERROR, FATAL
- **Dual Output**: Console (development) and file (always)
- **Async Queue**: Non-blocking file writes with queue management
- **Auto-cleanup**: Handles process exit gracefully
- **Cross-Process Support**: Works in both Electron main and child processes

#### Implementation Details:

**Dynamic Electron Detection** (Critical for child processes):
```typescript
// SafeLogger dynamically detects context
if (options.logDir) {
  this.logDir = options.logDir;
} else {
  try {
    // Try Electron's app if available (main process)
    const { app } = require('electron');
    this.logDir = path.join(app.getPath('userData'), 'logs');
  } catch {
    // Fallback for child processes (Memory Service, etc.)
    this.logDir = path.join(os.homedir(), '.hive-consensus', 'logs');
  }
}
```

**EPIPE Error Prevention**:
```typescript
private safeConsoleLog(level: string, message: string): void {
  try {
    if (process.stdout && process.stdout.writable) {
      process.stdout.write(`[${level}] ${message}\n`);
    }
  } catch (error: any) {
    // Silently ignore EPIPE errors - prevents crashes
    if (error.code !== 'EPIPE' && error.code !== 'EBADF') {
      // Non-EPIPE errors try stderr as fallback
      if (process.stderr?.writable) {
        process.stderr.write(`[LOGGER ERROR] ${error.message}\n`);
      }
    }
  }
}
```

#### Usage Best Practices:
```typescript
import { logger } from './utils/SafeLogger';

// ALWAYS use template literals for multi-value logging
logger.info(`Server started on port ${port}`);
logger.error(`Connection failed: ${error.message}`);

// AVOID multi-parameter logging (causes TypeScript errors)
// ❌ BAD: logger.info('Port', port, 'ready');
// ✅ GOOD: logger.info(`Port ${port} ready`);

// Structured logging with metadata
logger.info('Request received', { 
  method: 'POST', 
  path: '/api/query',
  userId: 123 
});
```

#### Log File Location:
- **macOS**: `~/Library/Application Support/Hive Consensus/logs/`
- **Windows**: `%APPDATA%/Hive Consensus/logs/`
- **Linux**: `~/.config/Hive Consensus/logs/`

#### Log File Format:
```
hive-2025-08-20T19-30-45-123Z.log
```

## Troubleshooting

### Known Issues

#### ~~EPIPE Errors During Consensus Operations~~ (FIXED in v1.4.0)
**Previous Issue**: Uncaught Exception dialogs with "Error: write EPIPE" during consensus streaming
**Root Cause**: Console.log statements in database callbacks when child process uses `stdio: 'inherit'`
**Solution Implemented**: SafeLogger system that gracefully handles EPIPE errors without crashing

#### Port Conflicts on Startup
**Symptoms**: "EADDRINUSE" errors when starting the app
**Cause**: Previous instance still running or port in use by another app
**Solution**: 
- App uses PortManager for automatic port allocation
- Finds next available port (up to 100 ports ahead)
- Never manually kill ports - let PortManager handle it

#### Memory Service Not Responding
**Symptoms**: Memory Service panel shows "Loading..." indefinitely
**Cause**: Express app not attached to HTTP server
**Solution**: Ensure `this.server = http.createServer(this.app)` in server.ts

## Recent Improvements (2025-08-25)

### System Log Enhancements (v1.8.2)
- **Smart Auto-Scroll**: Intelligently pauses when reviewing history, resumes at bottom
- **Manual Scrolling**: Full support for reviewing past logs without interruption
- **Improved Reliability**: Multiple scroll attempts ensure logs stay visible
- **Cleaner UI**: Removed unnecessary refresh button from terminal panel
- **Performance**: Optimized with 1000 entry limit and efficient DOM updates

### AI Tools Launch Tracking System
- **Launch History Database**: Track AI tool launches per repository
- **Intelligent Resume**: Automatic `--continue` flag for previously used tools
- **Unified Database**: Uses main `hive-ai.db` for all tracking
- **Usage Analytics**: Comprehensive statistics on tool usage patterns
- **Error Recovery**: Graceful degradation if database operations fail

### Smart Git Push System
- **Intelligent Strategy Analysis**: Analyzes repository and calculates actual push size (not repo size)
- **7 Push Strategies**: Standard, Chunked, Force, Fresh Branch, Squash, Bundle, Cleanup
- **Custom Commands**: Full support for cross-branch pushing and complex git operations
- **Enterprise Features**: Dry run validation, force with lease, atomic pushes
- **Visual Enhancements**: 
  - Clickable ahead/behind badges in Source Control panel
  - Smart Push dialog with strategy recommendations
  - Real-time push size calculation
  
### Repository Management
- **BFG Integration**: Clean large files from git history
- **Size Optimization**: Reduced 11GB repo to 5.9GB
- **Push Size vs Repo Size**: Smart recommendations based on actual data to push

### UI/UX Improvements
- **Panel Refresh System**: `recreatePanel()` method for reliable updates
- **DOM Safety**: Auto-recovery for missing elements
- **Event-Driven Updates**: No polling, fully reactive
- **System Log Auto-Scroll**: Fixed with requestAnimationFrame for reliable scrolling
- **Badge Interactions**: Click ahead/behind badges for smart actions

### Terminal Tab Fixes
- **DOM Element Recovery**: Auto-recreate missing tab containers
- **Safety Checks**: Validate elements before operations
- **AI CLI Tool Integration**: Proper event handling for tool launches

### IPC Handler Enhancements
- **Custom Command Support**: Properly handle custom git commands in dry run
- **Push Size Calculation**: Added `git rev-list` for accurate measurements
- **Shell Configuration**: Fixed shell parameter types for TypeScript

## System Integration Summary: Zero-Fallback Architecture

### How It All Works Together

#### Application Startup Sequence
```
1. Main Process Initializes
   → PortManager.initialize()       // Pre-scan ALL ports
   → ProcessManager.initialize()     // Setup control tower
   → StartupOrchestrator.start()     // Visual loading

2. Port Pre-Scanning (Parallel)
   → Load configuration/environment
   → Discover available port ranges
   → Scan all ranges in parallel
   → Build service port pools
   → Mark scan complete

3. Service Initialization (Sequential)
   → Database initialization
   → IPC handlers registration
   → Memory Service startup
      → PortManager allocates from pool
      → ProcessManager spawns with port
      → Service uses PROVIDED port only
   → Backend Server startup
      → Same pattern, different pool
   → All services running

4. Runtime Port Discovery
   → Renderer needs service port
   → IPC call to main process
   → ProcessManager provides port
   → Renderer connects to service
   → NO FALLBACK if service down
```

### Key Design Decisions

#### Why Pre-Scan?
- **Instant Allocation**: Services get ports immediately
- **Early Failure**: Know at startup if ports unavailable
- **No Race Conditions**: Scan completes before any allocation
- **Performance**: One scan vs. checking on each allocation

#### Why No Fallbacks?
- **Predictability**: System behavior is consistent
- **Debuggability**: Failures are clear and immediate
- **Security**: No predictable fallback ports
- **Correctness**: Either works properly or fails clearly

#### Why IPC Discovery?
- **Isolation**: Components don't know about each other's ports
- **Flexibility**: Port changes don't affect components
- **Centralization**: Single source of truth (ProcessManager)
- **Dynamic**: Adapts to runtime changes

### Production Deployment Considerations

```yaml
# production.env - Environment-specific configuration
MEMORY_PORT_START=3450
MEMORY_PORT_END=3550
MEMORY_PORT_POOL_SIZE=20

BACKEND_PORT_START=8700
BACKEND_PORT_END=8800
BACKEND_PORT_POOL_SIZE=10

TERMINAL_PORT_START=7100
TERMINAL_PORT_END=7200
TERMINAL_PORT_POOL_SIZE=50

# Container deployment
DOCKER_PORT_MAPPING=dynamic
KUBERNETES_SERVICE_DISCOVERY=enabled
```

### Monitoring & Diagnostics

```typescript
// Built-in diagnostics for port management
PortManager.getDiagnostics():
{
  scanComplete: true,
  scanDuration: 1847,  // ms
  totalPortsScanned: 520,
  availablePorts: {
    'memory-service': 18,
    'backend-server': 9,
    'ttyd-terminals': 45
  },
  allocatedPorts: [
    { service: 'memory-service', port: 3459, duration: 3600000 },
    { service: 'websocket-backend', port: 8767, duration: 3600000 }
  ],
  failedAllocations: [],
  portConflicts: []
}
```

### Error Handling Examples

```typescript
// When port allocation fails:
try {
  const port = await PortManager.allocatePortForService('memory-service');
  // ...
} catch (error) {
  if (error.code === 'NO_PORTS_AVAILABLE') {
    // Show user dialog: "Memory Service cannot start - no ports available"
    // Log diagnostic info
    // Offer retry or exit
  }
}

// When service discovery fails:
try {
  const port = await window.api.invoke('memory-service-port');
  // ...
} catch (error) {
  // Show in UI: "Memory Service is not running"
  // Disable features that depend on it
  // NO ATTEMPT TO CONNECT TO GUESSED PORT
}
```

## Future Enhancements

### Planned Features
1. **Repository Intelligence**: Full codebase analysis
2. **Planning Mode**: AI-powered task decomposition
3. **Team Collaboration**: Multi-user support
4. **Plugin System**: Extensible architecture
5. **Cloud Sync**: Full cloud backup
6. **Mobile Companion**: iOS/Android apps
7. **Voice Interface**: Speech input/output
8. **AI Agents**: Autonomous task execution
9. **CLI Tools UI**: Settings panel for tool management
10. **Extended CLI Support**: Additional AI CLI tools
11. **Launch Animation**: Cool loading graphic during app startup to provide visual feedback while services initialize

### Architecture Evolution
- Microservices migration
- Kubernetes deployment
- GraphQL API layer
- Event sourcing
- CQRS pattern implementation

---

## Appendix

### Environment Variables
```bash
OPENROUTER_API_KEY      # AI model access
CLOUDFLARE_ACCOUNT_ID   # D1 sync
CLOUDFLARE_D1_TOKEN     # D1 authentication
NODE_ENV               # development/production
MEMORY_SERVICE_PORT    # Default: 3457
```

### File Structure
```
electron-poc/
├── src/
│   ├── index.ts              # Main process
│   ├── renderer.ts           # UI entry
│   ├── preload.ts           # IPC bridge
│   ├── components/          # UI components
│   ├── memory-service/      # Memory service
│   ├── git/                # Git integration
│   ├── utils/              # Utilities
│   └── types/              # TypeScript types
├── assets/                  # Static resources
├── styles/                 # CSS files
└── dist/                   # Build output
```

### Key Design Decisions
1. **Electron over Web**: Desktop-first for file system access
2. **TypeScript**: Type safety and better tooling
3. **SQLite**: Local-first data storage
4. **Child Processes**: Isolation and fault tolerance
5. **IPC over HTTP**: Secure main-renderer communication
6. **Vanilla over Framework**: Minimal dependencies
7. **ProcessManager**: Production-ready process handling
8. **Memory Service**: Extensible AI tool integration

---

*This document is the single source of truth for the Hive Consensus architecture. It should be updated whenever significant architectural changes are made.*

**Last Updated**: 2025-08-23
**Version**: 1.7.4
**Maintainer**: Hive Development Team

### Change Log
- **v1.6.0 (2025-08-21)**: Complete AI CLI Tools Integration with Memory Service
  - **Claude Code Integration**: Full lifecycle management (install, update, configure)
  - **MCP Protocol Support**: Automatic MCP server configuration for Memory Service
  - **Token Authentication**: Secure per-tool authentication tokens
  - **Visual UI Feedback**: Progress indicators and status updates
  - **Details Button**: Refresh tool status and restore full details view
  - **Persistent Configuration**: Tool configs saved in ~/.hive/cli-tools-config.json
  - **Memory Service Bridge**: Claude Code can now query and contribute to AI memory

- **v1.6.2 (2025-08-22)**: Complete AI CLI Tools Integration Pattern Documentation
  - **Integration Pattern**: Comprehensive architectural pattern for all AI CLI tools
  - **Core Components**: Tool registry, detection system, IPC layer, UI architecture
  - **Five-Phase Flow**: Detection, Installation, Configuration, Launch, Update
  - **Terminal Management**: TTYD integration with port allocation and PID tracking
  - **Button Matrix**: Complete functionality mapping for all UI buttons
  - **Database Schema**: Tool tracking, launch history, update checks
  - **Error Handling**: Comprehensive patterns for all error scenarios
  - **Replication Guide**: Step-by-step instructions for adding new tools
  - **Testing Strategy**: Unit and integration test patterns
  - **Security Model**: Token isolation, command validation, audit logging
- **v1.6.1 (2025-08-22)**: CLI Tools Update Button Implementation
  - **Update Functionality**: Complete implementation of update button for all CLI tools
  - **NPM Package Updates**: Support for `npm update -g` with package name mapping
  - **Python/pip Support**: Handle pip-based tools like Aider with `pip install --upgrade`
  - **Version Detection**: Tool-specific version extraction (claude --version, npm list, etc.)
  - **Error Handling**: Comprehensive error handling for permissions, missing npm, network issues
  - **PATH Resolution**: Enhanced PATH with common binary locations for package managers
  - **UI State Management**: Real-time status updates without full panel refresh
  - **Security Hardening**: Command injection prevention via whitelist validation
  - **Documentation**: Complete architectural documentation in MASTER_ARCHITECTURE.md
- **v1.7.0 (2025-08-23)**: Complete AI CLI Tools Suite Implementation
  - **Gemini CLI Integration**: Full implementation with FREE tier support
  - **Qwen Code Integration**: Package/binary name mismatch discovered and documented
  - **OpenAI Codex Integration**: GPT-5 and multimodal support implemented
- **v1.7.2 (2025-08-23)**: UI Consolidation & Icon Resources
  - **Terminal Consolidation**: Hidden basic HTML terminal section at bottom center
  - **System Log Primary**: All logging now goes through System Log tab in TTYD panel
  - **Cleaner UI**: Removed redundant terminal display for better workspace
  - **Code Preserved**: Terminal section code kept but hidden for potential future use
  - **AI CLI Icons**: Downloaded and stored official logos for all AI CLI tools
  - **Icon Resources**: Created `/resources/ai-cli-icons/` with SVG logos
  - **Sidebar Planning**: Documented quick launch implementation for left activity bar
- **v1.7.1 (2025-08-23)**: Grok MCP Integration & Dynamic Port Handling
  - **Grok MCP Support**: Full Model Context Protocol integration for Grok CLI
  - **Dynamic Port Configuration**: Automatic MCP config updates on app startup
  - **Port Discovery**: Memory Service port (3457-3560) tracked and propagated
  - **Multi-Tool MCP**: Support for both Claude Code and Grok MCP configurations
  - **Setup Wizard Enhancement**: Interactive Grok API key configuration
  - **Token Persistence**: Unique authentication tokens maintained across restarts
  - **updateAllMCPConfigurations**: Function to sync all tool configs with actual port
  - **Grok Auto-Creation Fix**: MCP config now created if missing on startup
  - **Grok Detection Fix**: Added 'grok' to memory service connection check list
- **v1.7.2 (2025-08-23)**: AI CLI Tool Quick Launch Icons & UI Improvements
  - **Quick Launch Icons**: Implemented 6 AI tool icons in left sidebar for instant access
  - **Sidebar Reorganization**: Added dividers and fixed Settings at bottom
  - **Icon Resources**: Official logos for all 6 tools - Claude, Gemini, Grok, Qwen, OpenAI, Cline
  - **Dark Theme Icons**: Downloaded dark theme versions from LobeHub CDN for perfect visibility
  - **Icon Styling**: Smart CSS with inversion for black icons, preserved colors for Claude/Gemini
  - **Launch Integration**: Icons use identical `launchCliTool()` function as Launch buttons
  - **Terminal Cleanup**: Hidden basic HTML terminal in favor of TTYD System Log
  - **TypeScript Fix**: Resolved @types/node version mismatch for clean compilation
  - **Dual-Location Support**: Detector checks both ~/.grok and ~/.hive for configs
  - **UI Theming**: Applied HiveTechs gradient theme to all AI CLI tool buttons
  - **Batch Operations**: Added Install All, Update All, Uninstall All buttons
  - **Tool Reordering**: Cline moved to bottom as least-used tool
  - **Comprehensive Lessons Learned**: Documentation of all quirks and workarounds
- **v1.7.3 (2025-08-23)**: Icon Sizing, Brightness Consistency & Smart Click Handling
  - **Icon Size Optimization**: Scaled AI CLI icons from 42x42 to 24x24 for visual consistency
  - **Brightness Unification**: Brightened all sidebar icons from #858585 to #cccccc for equal visibility
  - **Smart Click Handling**: Uninstalled tools redirect to AI CLI Tools panel with auto-highlight
  - **Download Indicators**: Blue download arrow badge for uninstalled tools instead of warning symbol
  - **Auto-Refresh Icons**: Sidebar icons automatically refresh after install/update/uninstall operations
  - **CSS Filter Strategy**: Intelligent use of invert() for black icons while preserving colored logos
  - **Activity Bar Width**: Optimized at 48px (initially expanded to 56px, then reverted)
  - **Hover Effects**: Consistent brightness(1.1) filter across all icons for unified interaction
- **v1.7.4 (2025-08-23)**: Enhanced Panel Resize System & Complete DOM Documentation
  - **TTYD Panel Expansion**: Increased max width from 600px to 1200px for better terminal usage
  - **Consensus Panel Width**: Increased max width from 600px to 800px for improved readability
  - **Center Area Protection**: Added 400px minimum width constraint to prevent collapse
  - **Smart Resize Logic**: Panels only resize if center area maintains minimum width
  - **Complete DOM Documentation**: Added comprehensive section on DOM structure and panel architecture
  - **Panel Type Specifications**: Documented all four panel types (Fixed, Collapsible, Resizable, Flexible)
  - **Resize Mechanism Details**: Full documentation of drag handle system and formulas
  - **Layout Strategy**: Detailed flexbox implementation and CSS architecture
  - **Performance Guidelines**: Added DOM optimization and memory management best practices
  - **Accessibility Features**: Documented keyboard navigation and ARIA attributes
- **v1.7.5 (2025-08-23)**: Center Area Collapse, Unified Toggle Styling & ResizeObserver Fixes
  - **Center Area Collapse**: Implemented collapsible center/editor area matching consensus panel behavior
  - **Toggle Button Unification**: All panels now use identical discrete toggle styling (removed blue TTYD styling)
  - **Auto-Expand Behavior**: Panels automatically expand to fill space when others collapse
  - **Expand-to-Fill Class**: Added CSS class for dynamic panel space distribution
  - **ResizeObserver Fix**: Resolved loop errors using requestAnimationFrame for deferred DOM updates
  - **TypeScript Fixes**: Resolved variable naming conflicts and redeclaration issues
  - **Panel Variable Refactor**: Used unique names for different panel element references
  - **Collapse State Persistence**: All three main panels (TTYD, Center, Consensus) maintain collapse state
  - **Toggle Icons**: Consistent + (collapsed) and − (expanded) symbols across all panels
  - **Performance Improvement**: Eliminated ResizeObserver notification delivery errors
- **v1.8.2 (2025-08-25)**: System Log Auto-Scroll & UI Refinements
  - **Smart Auto-Scroll**: System Log now intelligently handles scrolling behavior
  - **Manual Scroll Detection**: Auto-scroll pauses when user scrolls up >100px from bottom
  - **Auto-Resume**: Auto-scroll resumes when user returns to within 20px of bottom
  - **Refresh Button Removal**: Removed unnecessary refresh button from terminal panel header
  - **Cleaner Controls**: Terminal panel now only shows essential controls (toggle, new tab)
  - **Improved Reliability**: Multiple scroll attempts ensure new logs are always visible
  - **Performance Optimization**: Efficient DOM updates with 1000 entry limit
  - **AI Tools Database Integration**: Launch tracking now uses unified hive-ai.db connection
  - **Smart Git Push Documentation**: Added comprehensive push strategy documentation
  - **Source Control Panel Layout Fix**: Redesigned with proper flexbox for optimal scrolling
    - Branch info stays fixed at top with sync indicators
    - Each resource group (Staged, Changes, Untracked) scrolls independently (200px max)
    - Commits section has its own scrollbar (200px max)
    - Eliminated gaps with `#git-content { padding: 0 }` CSS fix
    - Height calculation `calc(100vh - 22px)` prevents bottom bar overflow
- **v1.8.1 (2025-08-24)**: System Log Toggle & Terminal Tab Stability Fixes
  - **System Log Toggle Feature**: Added toggle button (📊) to show/hide System Log tab
  - **Hidden by Default**: System Log tab now starts hidden for cleaner initial UI
  - **Toggle Button Design**: Log list icon with 70% opacity when hidden, 100% when visible
  - **Tooltip Enhancement**: "Toggle System Log (📊)" tooltip for clear functionality
  - **Smart Tab Switching**: Automatically switches to another tab when hiding active System Log
  - **Centralized ProcessManager**: Fixed terminal tab issues with single ProcessManager instance
  - **Port Conflict Resolution**: Resolved EADDRINUSE errors through centralized port allocation
  - **Control Tower Architecture**: ProcessManager serves as single source of truth for all processes
- **v1.6.0 (2025-08-22)**: Enhanced Process Cleanup & Claude Code Integration
  - **PidTracker System**: Tracks all process PIDs to disk for cleanup across restarts
  - **Unified Cleanup Function**: Single performCleanup() prevents duplicate handlers
  - **Orphan Process Recovery**: Automatically kills lingering processes on startup
  - **Development Cleanup Script**: scripts/cleanup-dev.sh for manual cleanup
  - **Fixed Duplicate Handlers**: Removed duplicate before-quit handlers (lines 1639, 2865)
  - **Claude Code Launch Integration**: AI tool launch with resume detection per repository
  - **Global Folder Management**: Launch button updates File Explorer, Source Control, Status Bar
  - **Fixed WebSocket Port Warning**: Early registration of websocket-backend-port handler

- **v1.5.0 (2025-08-20)**: Memory Service Recovery & SafeLogger Cross-Process Support
  - **Fixed Memory Service Startup**: Resolved IPC ready message race condition in ProcessManager
  - **SafeLogger Cross-Process Support**: Dynamic Electron detection for main and child processes
  - **Template Literal Logging**: Fixed all multi-parameter log statements to use template literals
  - **IPC Ready Promise Fix**: Ready promise now created before message handlers to prevent interception
  - **Child Process Logging**: SafeLogger now falls back to ~/.hive-consensus/logs for child processes
  - **Memory Service Operational**: Successfully starts, connects via IPC, and queries database

- **v1.4.0 (2025-08-20)**: Production Logging System & Code Cleanup
  - **SafeLogger Implementation**: Production-ready logging system that handles EPIPE errors gracefully
  - **Replaced All Console Statements**: 206 console.log/error statements replaced with SafeLogger
  - **Log Rotation & Management**: Automatic log file rotation with 10MB max size and 5 file retention
  - **Code Cleanup**: Removed duplicate Memory Service functions and orphaned code
  - **Fixed EPIPE Errors**: No more uncaught exception dialogs during consensus operations
  
- **v1.3.0 (2025-08-20)**: Critical Service Fixes & Dynamic Port Management
  - **Fixed Memory Service**: Express app now properly attached to HTTP server
  - **Dynamic Port Discovery**: Frontend discovers backend ports via IPC instead of hardcoded values
  - **Webpack Port Configuration**: Moved Electron Forge webpack from port 9000 to 9100 to avoid conflicts
  - **Enhanced Port Management**: All IPC handlers now use ProcessManager's dynamic port allocation

- **v1.2.0 (2025-08-20)**: Python Runtime & AI Helpers Architecture
  - Implemented bundled Python runtime architecture for production
  - Added environment variable configuration for Python paths
  - Created self-contained app structure with all ML dependencies
  - Documented Python subprocess management and communication
  - Added production deployment strategies (portable Python, binary compilation)
  - Included troubleshooting guide for common Python/AI Helper issues

- **v1.1.0 (2025-08-20)**: Major process architecture overhaul
  - Implemented parallel startup for all services
  - Fixed AI Helper subprocess communication via stdio inheritance
  - Added dynamic port allocation with 100+ port ranges
  - Introduced deferred consensus initialization
  - Enhanced ProcessManager with comprehensive status reporting
  - Documented 2025 best practices for process management