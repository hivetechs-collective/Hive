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
14. [Future Enhancements](#future-enhancements)

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
Electron Main Process
├── Memory Service (Node.js Child Process)
│   ├── Express Server (Port 3457)
│   ├── WebSocket Server
│   └── IPC Channel to Main
├── Backend Server (Rust - Port 3456)
│   └── Consensus Engine
└── File Watchers
    └── Git Status Monitor
```

### ProcessManager System
**Location**: `src/utils/ProcessManager.ts`

**Features**:
- Automatic process spawning
- Health monitoring
- Auto-restart on crash (configurable retries)
- Graceful shutdown
- IPC message routing
- Port management integration

**Process Lifecycle**:
```typescript
1. Register Process → 
2. Allocate Port (via PortManager) → 
3. Spawn Child Process → 
4. Monitor Health → 
5. Handle Messages/Crashes → 
6. Cleanup on Exit
```

### PortManager System
**Location**: `src/utils/PortManager.ts`

**Features**:
- Port availability checking
- Automatic conflict resolution
- Process killing on port conflicts
- Alternative port selection
- Port allocation tracking

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
  queriesToday: 6,         // Actual consensus queries
  contributionsToday: 5,   // Messages added today
  connectedTools: 0,       // Currently connected tools
  hitRate: 92,            // Query success rate
  avgResponseTime: 45     // Milliseconds
}
```

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

**Last Updated**: 2025-08-19
**Version**: 1.0.0
**Maintainer**: Hive Development Team