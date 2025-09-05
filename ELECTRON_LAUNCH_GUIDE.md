# Hive Electron App Launch Guide

## Architecture Overview

The Hive Electron application consists of two main components that must be launched together:

1. **Backend Server** (Rust) - `hive-backend-server-enhanced`
2. **Frontend App** (Electron) - `electron-poc`

## Prerequisites

- Rust toolchain installed
- Node.js 18+ and npm installed
- SQLite database at `~/.hive/hive-ai.db`
- OpenRouter API key configured

## Launch Instructions

### Step 1: Start the Backend Server

The backend server provides WebSocket streaming, consensus processing, and maintenance services.

```bash
# Build the backend (first time or after changes)
cd /Users/veronelazio/Developer/Private/hive
cargo build --bin hive-backend-server-enhanced

# Run the backend server
./target/debug/hive-backend-server-enhanced
```

The backend will start on **http://localhost:8765** with:
- WebSocket endpoint: `ws://localhost:8765/ws`
- REST API: `http://localhost:8765/api/*`
- Health check: `http://localhost:8765/health`
- Maintenance status: `http://localhost:8765/api/maintenance/status`

### Step 2: Start the Electron Frontend

In a **separate terminal**, launch the Electron app:

```bash
# Navigate to the Electron app directory
cd /Users/veronelazio/Developer/Private/hive/electron-poc

# Install dependencies (first time only)
npm install

# Start the Electron app
npm start
```

The Electron app will:
1. Open a desktop window with the chat interface
2. Connect to the backend via WebSocket
3. Load settings from the SQLite database
4. Display the developer console for debugging

## Backend Features

### Consensus Processing
- 4-stage pipeline with streaming support
- Real-time token counting and cost calculation
- Model selection based on profiles
- WebSocket streaming to frontend

### Maintenance System
- **Automatic OpenRouter sync**: Every 24 hours
- **Database cleanup**: Every 7 days
- **CPU throttling**: Max 50% CPU usage
- **Status monitoring**: Via REST API

### API Endpoints

```bash
# Check health and features
curl http://localhost:8765/health

# Get maintenance status
curl http://localhost:8765/api/maintenance/status

# Force OpenRouter sync
curl -X POST http://localhost:8765/api/maintenance/sync

# Quick consensus test
curl -X POST http://localhost:8765/api/consensus/quick \
  -H "Content-Type: application/json" \
  -d '{"query": "Hello"}'
```

## Frontend Features

### Main Components
- **Chat Interface**: Real-time streaming responses
- **Settings Panel**: API key configuration and profiles
- **Progress Indicators**: Visual feedback for each consensus stage
- **Token Counter**: Live token usage and cost display

### WebSocket Communication
The frontend uses IPC bridge for secure WebSocket communication:
- Main process handles WebSocket connection
- Renderer process receives messages via IPC
- Automatic reconnection on disconnect

## Troubleshooting

### Backend Issues

**Port already in use:**
```bash
# Find and kill existing process
lsof -i :8765
kill -9 <PID>
```

**Database connection errors:**
```bash
# Check database exists
ls -la ~/.hive/hive-ai.db

# Check permissions
chmod 644 ~/.hive/hive-ai.db
```

**High CPU usage:**
- Maintenance system has built-in throttling
- Check with: `curl http://localhost:8765/api/maintenance/status`

### Frontend Issues

**WebSocket connection failed:**
1. Ensure backend is running first
2. Check backend logs for errors
3. Verify port 8765 is accessible

**Blank screen or errors:**
1. Open developer console (auto-opens)
2. Check for JavaScript errors
3. Verify backend health: `curl http://localhost:8765/health`

**Settings not saving:**
- Check database write permissions
- Verify configurations table exists
- Check Electron main process logs

## Development Workflow

### Making Backend Changes
1. Stop backend with `Ctrl+C`
2. Make code changes
3. Rebuild: `cargo build --bin hive-backend-server-enhanced`
4. Restart backend

### Making Frontend Changes
1. Frontend hot-reloads automatically
2. For main process changes, type `rs` in terminal
3. For major changes, restart with `Ctrl+C` then `npm start`

## Important Notes

- **Always start backend first**, then frontend
- Backend runs maintenance tasks automatically (no manual intervention needed)
- WebSocket reconnects automatically if connection drops
- Database is shared between backend and frontend
- Maintenance runs with CPU throttling to prevent overheating

## Monitoring

### Backend Logs
Watch for:
- `✅ Enhanced Backend Server running`
- `✅ Background maintenance system started`
- `🔄 Running scheduled OpenRouter sync`
- WebSocket connection messages

### Frontend Logs
Check developer console for:
- WebSocket connection status
- IPC message flow
- Settings load/save operations
- Consensus streaming events

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Electron Frontend                     │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Chat UI     │  │ Settings UI   │  │ Progress UI  │  │
│  └─────────────┘  └──────────────┘  └──────────────┘  │
│                           │                              │
│                    ┌──────▼────────┐                    │
│                    │  IPC Bridge   │                    │
│                    └──────┬────────┘                    │
└───────────────────────────┼─────────────────────────────┘
                            │
                     WebSocket (ws://)
                            │
┌───────────────────────────▼─────────────────────────────┐
│               Rust Backend Server (8765)                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  Consensus   │  │  Maintenance │  │  REST API    │ │
│  │   Engine     │  │    System    │  │  Endpoints   │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                           │                             │
│                    ┌──────▼────────┐                   │
│                    │  SQLite DB    │                   │
│                    │ ~/.hive/*.db  │                   │
│                    └───────────────┘                   │
└──────────────────────────────────────────────────────────┘
```

## Quick Start Summary

```bash
# Terminal 1: Start Backend
cd /Users/veronelazio/Developer/Private/hive
./target/debug/hive-backend-server-enhanced

# Terminal 2: Start Frontend  
cd /Users/veronelazio/Developer/Private/hive/electron-poc
npm start

# Terminal 3: Monitor (optional)
curl http://localhost:8765/health
curl http://localhost:8765/api/maintenance/status
```

That's it! The system should now be running with full consensus processing and automatic maintenance.