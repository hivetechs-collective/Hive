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
14. [CLI Tools Management UI](#cli-tools-management-ui)
15. [Future Enhancements](#future-enhancements)

---

## System Overview

### Purpose
Hive Consensus is an advanced AI-powered development environment that combines:
- **4-Stage Consensus AI Processing** (Generator → Refiner → Validator → Curator)
- **Universal Memory Infrastructure** (Memory-as-a-Service for AI tools)
- **VS Code-like Development Environment** in Electron
- **Deep Git Integration** with visual source control
- **Real-time Collaboration** between human and AI

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
**Port**: 3457 (configurable)
**Responsibilities**:
- Memory-as-a-Service API
- External tool integration
- Query processing
- Learning contribution
- Statistics tracking

---

## Process Architecture

### Process Hierarchy
```
Electron Main Process (Orchestrator)
├── Memory Service (Node.js Child Process)
│   ├── Express Server (Dynamic Port: 3457-3560)
│   ├── WebSocket Server
│   └── IPC Channel to Main
├── WebSocket Backend Server (Rust Binary)
│   ├── Primary Port: 8765 (Dynamic: 8766-8865)
│   ├── Consensus Engine with AI Helpers
│   └── Deferred Initialization Architecture
└── File Watchers
    └── Git Status Monitor
```

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

#### 2. Stdio Configuration (Critical for AI Helpers)
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

#### 6. Enhanced Status Reporting
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

### Component Hierarchy
```
App Root (renderer.ts)
├── Header Bar
│   ├── App Title
│   └── Window Controls
├── Sidebar (Collapsible)
│   ├── File Explorer
│   └── Git Status View
├── Main Content Area
│   ├── Editor Tabs
│   │   ├── Code Editor (Monaco)
│   │   ├── Git Diff View
│   │   └── Memory Dashboard
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

### UI Components

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
    HTTP/REST API
         ↓
Memory Service (Express Server)
         ↓
    IPC Channel
         ↓
   Main Process
         ↓
   SQLite Database
```

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
- getStatus()      // Working tree status
- stage(files)     // Stage changes
- unstage(files)   // Unstage changes
- commit(message)  // Create commit
- push()          // Push to remote
- pull()          // Pull from remote
- sync()          // Pull + Push
- getBranches()   // List branches
- switchBranch()  // Change branch
```

### Authentication System
**Location**: `src/git/authentication/`

#### Askpass Helper
- Intercepts Git credential requests
- Shows authentication dialog
- Securely passes credentials
- Supports username/password and tokens

### Visual Interface
- File status indicators (M, A, D, U, ?)
- Inline diff visualization
- Commit message input
- Push/pull notifications
- Branch selector

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

### Build System
```bash
# Development
npm start           # Electron Forge dev server

# Production
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

## CLI Tools Management

### Overview
The CLI Tools Management system provides automated installation, updates, and integration for AI-powered CLI tools, with a primary focus on Claude Code CLI integration with our Memory Service.

### Architecture
**Location**: `src/utils/CliToolsManager.ts`
**Purpose**: Manage lifecycle of external AI CLI tools
**Integration**: Direct connection to Memory Service for memory-as-a-service

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
   - Primary integration with Memory Service
   - Auto-configuration of memory endpoints
   - Token-based authentication

2. **GitHub Copilot CLI** (`gh copilot`)
   - Extension-based installation
   - Requires GitHub CLI prerequisite

3. **OpenAI CLI** (`openai-cli`)
   - Python-based installation
   - API key configuration

### Installation Flow
```
1. Check tool prerequisites
2. Verify system dependencies
3. Execute installation command (npm/pip/gh)
4. Verify installation success
5. Configure Memory Service integration (if applicable)
6. Save status to database and config
```

### Memory Service Integration
For Claude CLI specifically:
1. Register tool with Memory Service API
2. Receive authentication token
3. Configure Claude CLI with:
   - Memory Service endpoint (http://localhost:3457)
   - Authentication token
   - Auto-sync enabled

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
// Main process handlers
'cli-tools:install': Install a specific tool
'cli-tools:check-updates': Check for tool updates
'cli-tools:get-status': Get all tool statuses
'cli-tools:update': Update a specific tool
```

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
│ [Check Update] [Configure] [Remove] │
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

## Installation Detection & Dynamic UI Management

### Overview
The CLI Tools panel dynamically detects installed tools and updates the UI accordingly, providing real-time status updates and appropriate action buttons based on each tool's installation state.

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

**Last Updated**: 2025-08-20
**Version**: 1.2.0
**Maintainer**: Hive Development Team

### Change Log
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