# Migration Plan: Dioxus to Tauri 2.0

## Executive Summary
Migrating Hive from Dioxus to Tauri 2.0 will provide:
- **Binary size reduction**: From 15MB to 0.6-3MB (5-25x smaller)
- **Memory usage**: From 55MB to 30-50MB (10-45% reduction)
- **Startup time**: From 417ms to <500ms (comparable)
- **Production maturity**: 81,000+ GitHub stars vs 24,500 (3.3x more mature)
- **Mobile support**: iOS and Android via Tauri 2.0
- **Security**: External audit by Radically Open Security

## Architecture Comparison

### Current (Dioxus)
```
┌─────────────────────────────────────┐
│         Dioxus Desktop App          │
│  ┌─────────────────────────────┐    │
│  │   Virtual DOM (React-like)   │    │
│  └─────────────────────────────┘    │
│  ┌─────────────────────────────┐    │
│  │    WebView (WebKit/WebView2) │    │
│  └─────────────────────────────┘    │
│  ┌─────────────────────────────┐    │
│  │     Rust Business Logic      │    │
│  │   (Consensus, AI Helpers)    │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

### Target (Tauri 2.0)
```
┌─────────────────────────────────────┐
│         Tauri 2.0 App               │
│  ┌─────────────────────────────┐    │
│  │   Frontend (React/Vue/Svelte)│    │
│  │      Running in WebView       │    │
│  └──────────┬──────────────────┘    │
│             │ IPC Commands           │
│  ┌──────────▼──────────────────┐    │
│  │    Rust Backend (Tauri)      │    │
│  │   - Consensus Engine         │    │
│  │   - AI Helpers               │    │
│  │   - File Operations          │    │
│  │   - Terminal Management      │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

## Phase 1: Project Setup (Week 1)

### 1.1 Initialize Tauri Project
```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0"

# Create new Tauri project structure
cargo tauri init
```

### 1.2 Choose Frontend Framework
**Recommendation: React with TypeScript**
- Largest ecosystem and community
- Best Tauri documentation and examples
- Familiar to most developers
- Excellent tooling (Vite, hot-reload)

Alternative options:
- Vue 3: Lighter weight, easier learning curve
- Svelte: Best performance, smallest bundle
- SolidJS: React-like with better performance

### 1.3 Project Structure
```
hive/
├── src-tauri/           # Rust backend
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs      # Tauri app entry
│   │   ├── commands/    # IPC command handlers
│   │   ├── consensus/   # Existing consensus engine
│   │   └── state/       # Application state
│   └── tauri.conf.json  # Tauri configuration
├── src/                 # Frontend (React/Vue/Svelte)
│   ├── App.tsx
│   ├── components/
│   ├── hooks/
│   └── stores/
├── package.json
└── vite.config.ts
```

## Phase 2: Backend Migration (Week 2-3)

### 2.1 Core Components to Migrate

#### Consensus Engine (Priority 1)
Current: `src/desktop/consensus_integration.rs`
Target: `src-tauri/src/commands/consensus.rs`

```rust
// Tauri command example
#[tauri::command]
async fn run_consensus(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<ConsensusResult, String> {
    // Existing consensus logic
    let manager = state.consensus_manager.lock().await;
    manager.process_query(&query).await
        .map_err(|e| e.to_string())
}

// Streaming with events
#[tauri::command]
async fn run_consensus_streaming(
    query: String,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.consensus_manager.lock().await;
    let (result, mut rx) = manager.process_query_streaming(&query).await?;
    
    // Stream events to frontend
    while let Some(event) = rx.recv().await {
        window.emit("consensus-update", event)?;
    }
    
    Ok(())
}
```

#### File Explorer (Priority 2)
Current: `src/desktop/file_explorer.rs`
Target: `src-tauri/src/commands/filesystem.rs`

```rust
#[tauri::command]
async fn read_directory(path: String) -> Result<Vec<FileEntry>, String> {
    // Existing file system logic
}

#[tauri::command]
async fn open_file(path: String) -> Result<String, String> {
    // File reading logic
}
```

#### Terminal Management (Priority 3)
Current: `src/desktop/terminal_tabs.rs`
Target: `src-tauri/src/commands/terminal.rs`

**Challenge**: WebView cannot directly access PTY
**Solution**: Use Tauri's shell plugin or implement WebSocket bridge

```rust
use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn create_terminal(
    app: tauri::AppHandle,
) -> Result<TerminalId, String> {
    // Create PTY process
    // Return terminal ID for frontend
}
```

### 2.2 State Management
```rust
// src-tauri/src/state.rs
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub consensus_manager: Arc<Mutex<ConsensusManager>>,
    pub file_watcher: Arc<Mutex<FileWatcher>>,
    pub terminal_manager: Arc<Mutex<TerminalManager>>,
    pub analytics: Arc<Mutex<AnalyticsData>>,
}

// In main.rs
fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            run_consensus,
            run_consensus_streaming,
            read_directory,
            open_file,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Phase 3: Frontend Migration (Week 3-4)

### 3.1 Component Mapping

| Dioxus Component | Tauri/React Equivalent | Notes |
|-----------------|------------------------|-------|
| `ConsensusPanel` | `ConsensusPanel.tsx` | Use React hooks for state |
| `FileExplorer` | `FileExplorer.tsx` | Tree component (Arco Design/Ant Design) |
| `TerminalTabs` | `Terminal.tsx` | Xterm.js or Hyper terminal |
| `ChatInterface` | `Chat.tsx` | Markdown rendering with react-markdown |
| `StatusBar` | `StatusBar.tsx` | CSS Grid/Flexbox layout |

### 3.2 Frontend Architecture (React Example)
```typescript
// src/hooks/useConsensus.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function useConsensus() {
    const [loading, setLoading] = useState(false);
    const [result, setResult] = useState<string>('');
    const [progress, setProgress] = useState<ConsensusProgress>({});
    
    useEffect(() => {
        const unlisten = listen('consensus-update', (event) => {
            setProgress(event.payload);
        });
        
        return () => {
            unlisten.then(fn => fn());
        };
    }, []);
    
    const runQuery = async (query: string) => {
        setLoading(true);
        try {
            await invoke('run_consensus_streaming', { query });
        } catch (error) {
            console.error(error);
        } finally {
            setLoading(false);
        }
    };
    
    return { runQuery, loading, result, progress };
}
```

### 3.3 UI Component Library
**Recommendation: Arco Design or Ant Design**
- Complete component set
- VS Code-like aesthetics available
- Tree view for file explorer
- Terminal component support
- Excellent TypeScript support

## Phase 4: Feature Parity (Week 4-5)

### 4.1 Critical Features to Preserve

#### Progress Tracking
```typescript
// Frontend
interface ConsensusProgress {
    stage: 'Generator' | 'Refiner' | 'Validator' | 'Curator';
    progress: number;
    tokens: number;
    cost: number;
    streaming: string;
}

// Backend event
window.emit("consensus-progress", {
    stage: "Generator",
    progress: 45,
    tokens: 1234,
    cost: 0.0023,
    streaming: "Analyzing query..."
});
```

#### File Operations
- Use Tauri's fs API for secure file access
- Implement permission system with allowlist
- Preserve auto-save functionality

#### Terminal Integration
Options:
1. **Xterm.js + WebSocket**: Full PTY support
2. **Tauri Shell Plugin**: Simpler but limited
3. **Hybrid**: Commands via Tauri, output via WebSocket

### 4.2 Tauri Plugins to Use
```toml
# tauri.conf.json
{
  "plugins": {
    "fs": {
      "all": false,
      "readFile": true,
      "writeFile": true,
      "readDir": true,
      "scope": ["$APPDATA/**", "$DOCUMENT/**"]
    },
    "shell": {
      "all": false,
      "execute": true,
      "scope": [
        { "name": "git", "cmd": "git", "args": true },
        { "name": "npm", "cmd": "npm", "args": true }
      ]
    },
    "notification": true,
    "clipboard": true,
    "dialog": true,
    "updater": true
  }
}
```

## Phase 5: Performance Optimization (Week 5-6)

### 5.1 Bundle Size Optimization
```javascript
// vite.config.ts
export default {
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'ui-vendor': ['@arco-design/web-react'],
          'editor': ['@monaco-editor/react'],
        }
      }
    },
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true
      }
    }
  }
}
```

### 5.2 Rust Backend Optimization
```toml
# Cargo.toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols
panic = "abort"     # Smaller panic handler
```

### 5.3 Expected Metrics
- **Binary size**: 2-3MB (vs 15MB Dioxus)
- **Memory usage**: 35-45MB (vs 55MB)
- **Startup time**: 300-400ms (vs 417ms)
- **CPU usage**: 5-10% idle (vs 15-20%)

## Phase 6: Mobile Support (Week 6-7)

### 6.1 iOS Configuration
```json
// tauri.conf.json
{
  "tauri": {
    "bundle": {
      "iOS": {
        "developmentTeam": "TEAM_ID",
        "minimumSystemVersion": "13.0"
      }
    }
  }
}
```

### 6.2 Android Configuration
```json
{
  "tauri": {
    "bundle": {
      "android": {
        "minSdkVersion": 24,
        "targetSdkVersion": 34
      }
    }
  }
}
```

## Migration Checklist

### Week 1: Setup
- [ ] Initialize Tauri 2.0 project
- [ ] Setup React/Vue/Svelte frontend
- [ ] Configure build pipeline
- [ ] Setup development environment

### Week 2-3: Backend
- [ ] Migrate consensus engine to Tauri commands
- [ ] Implement IPC communication layer
- [ ] Port file system operations
- [ ] Setup state management

### Week 3-4: Frontend
- [ ] Build main layout components
- [ ] Implement consensus UI
- [ ] Create file explorer
- [ ] Add terminal support

### Week 4-5: Feature Parity
- [ ] Progress tracking and streaming
- [ ] Analytics dashboard
- [ ] Settings management
- [ ] Keyboard shortcuts

### Week 5-6: Optimization
- [ ] Bundle size optimization
- [ ] Performance profiling
- [ ] Memory leak detection
- [ ] Cross-platform testing

### Week 6-7: Mobile
- [ ] iOS build configuration
- [ ] Android build configuration
- [ ] Responsive UI adjustments
- [ ] Mobile-specific features

## Risk Mitigation

### Technical Risks
1. **Terminal Integration Complexity**
   - Mitigation: Use proven solutions (xterm.js + node-pty)
   - Fallback: Shell command execution without full PTY

2. **State Synchronization**
   - Mitigation: Use Zustand/Redux for frontend state
   - Implement robust error handling

3. **Performance Regression**
   - Mitigation: Continuous benchmarking
   - Profile both frontend and backend

### Migration Risks
1. **Feature Loss**
   - Mitigation: Comprehensive testing matrix
   - User acceptance testing

2. **Timeline Slippage**
   - Mitigation: Prioritize core features
   - Incremental migration possible

## Success Metrics

### Performance
- Binary size < 3MB
- Memory usage < 45MB
- Startup time < 400ms
- 60 FPS UI interactions

### Functionality
- 100% feature parity with Dioxus version
- All tests passing
- No regression in consensus accuracy

### User Experience
- Improved responsiveness
- Native feel on all platforms
- Consistent cross-platform behavior

## Conclusion

Migrating from Dioxus to Tauri 2.0 offers significant benefits:
- **5-25x smaller binaries**
- **3.3x larger ecosystem**
- **Production validation** from major applications
- **Mobile platform support**
- **Better performance characteristics**

The migration can be completed in 6-7 weeks with proper planning and execution. The architecture change from single-process Dioxus to Tauri's frontend/backend separation provides better scalability and maintainability for future development.