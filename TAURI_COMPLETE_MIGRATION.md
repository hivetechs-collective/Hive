# Complete Tauri Migration Plan: Full Feature Parity

## 🎯 Goal: Recreate 100% of hive-consensus GUI Features in Tauri

## 📋 Complete Feature Inventory

Based on analysis of the current Dioxus implementation, here are ALL features that need migration:

### 1. **Core Consensus Features** ✅ (Partially Done)
- [x] Consensus engine integration
- [x] 4-stage pipeline display
- [x] Progress tracking
- [x] Token counting
- [x] Cost accounting
- [ ] Profile selection dropdown
- [ ] Model browser
- [ ] Auto-accept toggle
- [ ] History view

### 2. **File Explorer** 🔄
- [ ] Tree view with expand/collapse
- [ ] File icons by type
- [ ] Git status indicators
- [ ] Context menu (rename, delete, new file/folder)
- [ ] File preview on click
- [ ] Search/filter
- [ ] Drag and drop support

### 3. **Terminal System** 🔧
- [ ] Multiple terminal tabs
- [ ] PTY support for real shell
- [ ] Terminal registry for command routing
- [ ] Xterm.js integration
- [ ] Resize support
- [ ] Copy/paste
- [ ] ANSI color support
- [ ] Clear terminal command

### 4. **LazyGit Integration** 🔧
- [ ] LazyGit detection/installation
- [ ] Embedded LazyGit terminal
- [ ] Git status in status bar
- [ ] Quick commit shortcuts
- [ ] Branch switching UI
- [ ] Diff viewer

### 5. **Analytics Dashboard** 📊
- [ ] Cost breakdown charts
- [ ] Query history
- [ ] Model usage statistics
- [ ] Performance metrics
- [ ] Export functionality
- [ ] Time-based filtering

### 6. **Settings & Configuration** ⚙️
- [ ] API key management
- [ ] Theme selection (dark/light)
- [ ] Auto-save toggle
- [ ] Temperature/token settings
- [ ] Profile management
- [ ] Keyboard shortcuts config

### 7. **Status Bar** 📍
- [ ] Git branch/status
- [ ] Current file path
- [ ] Line/column position
- [ ] Problems count
- [ ] Consensus status
- [ ] Cost counter

### 8. **Keyboard & Commands** ⌨️
- [ ] Command palette (Cmd+Shift+P)
- [ ] Global shortcuts
- [ ] Vim mode support
- [ ] Quick actions
- [ ] Search everywhere (Cmd+P)

### 9. **Dialogs & Modals** 💬
- [ ] Welcome/onboarding dialog
- [ ] Settings dialog
- [ ] About dialog
- [ ] Update available dialog
- [ ] Operation confirmation dialog
- [ ] Error/warning dialogs

### 10. **Additional Features** ➕
- [ ] Resizable panels
- [ ] Activity bar (left sidebar)
- [ ] Tab management
- [ ] Notification system
- [ ] Auto-updater
- [ ] Window state persistence

## 🏗️ Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1)
**Goal**: Get the basic app working with essential features

#### 1.1 Complete Frontend Setup
```bash
cd frontend
npm install
npm run dev  # Verify React app runs
```

#### 1.2 Create Core Layout
```typescript
// src/App.tsx - Main layout structure
<Layout>
  <Sider>
    <ActivityBar />
    <FileExplorer />
  </Sider>
  <Layout>
    <Header>
      <Tabs />
    </Header>
    <Content>
      <SplitPane>
        <ConsensusPanel />
        <TerminalPanel />
      </SplitPane>
    </Content>
    <Footer>
      <StatusBar />
    </Footer>
  </Layout>
</Layout>
```

#### 1.3 Port State Management
```typescript
// src/stores/appStore.ts
interface AppState {
  consensus: ConsensusState;
  fileExplorer: FileExplorerState;
  terminal: TerminalState;
  git: GitState;
  settings: SettingsState;
  analytics: AnalyticsState;
}
```

### Phase 2: File Explorer & Editor (Week 2)

#### 2.1 Implement File Tree
```typescript
// src/components/FileExplorer.tsx
import { Tree } from 'antd';
import { invoke } from '@tauri-apps/api/core';

const FileExplorer = () => {
  const [treeData, setTreeData] = useState([]);
  
  const loadDirectory = async (path: string) => {
    const files = await invoke('read_directory', { path });
    // Convert to tree structure
  };
  
  return (
    <Tree
      treeData={treeData}
      onSelect={handleFileSelect}
      onExpand={handleExpand}
      showIcon
      showLine
    />
  );
};
```

#### 2.2 Add Monaco Editor
```typescript
// src/components/CodeEditor.tsx
import MonacoEditor from '@monaco-editor/react';

const CodeEditor = ({ file }) => {
  const [content, setContent] = useState('');
  
  useEffect(() => {
    invoke('read_file', { path: file.path })
      .then(setContent);
  }, [file]);
  
  return (
    <MonacoEditor
      value={content}
      language={detectLanguage(file)}
      theme="vs-dark"
      options={{
        minimap: { enabled: true },
        fontSize: 14,
      }}
    />
  );
};
```

### Phase 3: Terminal Integration (Week 3)

#### 3.1 Backend PTY Support
```rust
// src-tauri/src/terminal/pty.rs
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

pub struct TerminalPty {
    pty: Box<dyn MasterPty>,
    child: Box<dyn Child>,
}

impl TerminalPty {
    pub fn spawn(shell: &str, size: PtySize) -> Result<Self> {
        let pty_system = native_pty_system();
        let pty = pty_system.openpty(size)?;
        let cmd = CommandBuilder::new(shell);
        let child = pty.spawn(cmd)?;
        Ok(Self { pty, child })
    }
}
```

#### 3.2 Frontend Terminal Component
```typescript
// src/components/Terminal.tsx
import { Terminal } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { WebLinksAddon } from 'xterm-addon-web-links';

const TerminalComponent = ({ id }) => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<Terminal>();
  
  useEffect(() => {
    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
      },
    });
    
    term.open(terminalRef.current!);
    
    // Connect to backend PTY via WebSocket
    const ws = new WebSocket('ws://localhost:3030/terminal');
    ws.onmessage = (e) => term.write(e.data);
    term.onData((data) => ws.send(data));
    
    return () => {
      term.dispose();
      ws.close();
    };
  }, []);
  
  return <div ref={terminalRef} style={{ height: '100%' }} />;
};
```

### Phase 4: LazyGit Integration (Week 3)

#### 4.1 LazyGit Wrapper
```rust
// src-tauri/src/git/lazygit.rs
#[tauri::command]
pub async fn ensure_lazygit_installed() -> Result<bool> {
    match Command::new("lazygit").arg("--version").output() {
        Ok(_) => Ok(true),
        Err(_) => {
            // Prompt user to install
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn launch_lazygit(window: Window) -> Result<()> {
    // Create a PTY for LazyGit
    let pty = TerminalPty::spawn("lazygit", PtySize::default())?;
    
    // Stream output to frontend
    tokio::spawn(async move {
        while let Some(data) = pty.read().await {
            window.emit("lazygit-output", data)?;
        }
    });
    
    Ok(())
}
```

#### 4.2 Git Status Integration
```typescript
// src/components/GitPanel.tsx
const GitPanel = () => {
  const [gitStatus, setGitStatus] = useState<GitStatus>();
  
  useEffect(() => {
    const interval = setInterval(async () => {
      const status = await invoke('get_git_status');
      setGitStatus(status);
    }, 1000);
    
    return () => clearInterval(interval);
  }, []);
  
  return (
    <div>
      <div>Branch: {gitStatus?.branch}</div>
      <div>Changes: {gitStatus?.changes.length}</div>
      <Button onClick={() => invoke('launch_lazygit')}>
        Open LazyGit
      </Button>
    </div>
  );
};
```

### Phase 5: Analytics & Settings (Week 4)

#### 5.1 Analytics Dashboard
```typescript
// src/components/AnalyticsDashboard.tsx
import { LineChart, Line, AreaChart, Area } from 'recharts';

const AnalyticsDashboard = () => {
  const [analytics, setAnalytics] = useState<AnalyticsData>();
  
  useEffect(() => {
    invoke('get_analytics_data').then(setAnalytics);
  }, []);
  
  return (
    <div>
      <Card title="Cost Over Time">
        <LineChart data={analytics?.costHistory}>
          <Line type="monotone" dataKey="cost" stroke="#8884d8" />
        </LineChart>
      </Card>
      
      <Card title="Query Volume">
        <AreaChart data={analytics?.queryHistory}>
          <Area dataKey="count" fill="#82ca9d" />
        </AreaChart>
      </Card>
    </div>
  );
};
```

#### 5.2 Settings Dialog
```typescript
// src/components/SettingsDialog.tsx
const SettingsDialog = ({ visible, onClose }) => {
  const [settings, setSettings] = useState<Settings>();
  
  return (
    <Modal title="Settings" open={visible} onCancel={onClose}>
      <Tabs>
        <TabPane tab="General" key="general">
          <Form>
            <Form.Item label="Theme">
              <Select value={settings?.theme}>
                <Option value="dark">Dark</Option>
                <Option value="light">Light</Option>
              </Select>
            </Form.Item>
          </Form>
        </TabPane>
        
        <TabPane tab="API Keys" key="api">
          <Input.Password
            placeholder="OpenRouter API Key"
            value={settings?.apiKey}
            onChange={(e) => updateApiKey(e.target.value)}
          />
        </TabPane>
      </Tabs>
    </Modal>
  );
};
```

### Phase 6: Polish & Testing (Week 5)

#### 6.1 Keyboard Shortcuts
```typescript
// src/hooks/useKeyboardShortcuts.ts
const useKeyboardShortcuts = () => {
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      // Command palette
      if (e.metaKey && e.shiftKey && e.key === 'p') {
        openCommandPalette();
      }
      // Quick open
      if (e.metaKey && e.key === 'p') {
        openQuickOpen();
      }
      // Save
      if (e.metaKey && e.key === 's') {
        saveCurrentFile();
      }
    };
    
    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, []);
};
```

#### 6.2 Window State Persistence
```typescript
// Already configured in tauri.conf.json with window-state plugin
// Automatically saves/restores window position and size
```

## 🚀 Migration Execution Plan

### Week 1: Foundation
```bash
# Day 1-2: Setup and basic layout
cd frontend
npm install
npm run dev

# Day 3-4: Port core components
# - Layout structure
# - State management
# - Basic routing

# Day 5: Integration test
npm run tauri:dev
```

### Week 2: File System
```bash
# Day 1-2: File explorer
# Day 3-4: Monaco editor integration
# Day 5: File operations (save, rename, delete)
```

### Week 3: Terminal & Git
```bash
# Day 1-2: Terminal PTY backend
# Day 3: Xterm.js frontend
# Day 4-5: LazyGit integration
```

### Week 4: Features
```bash
# Day 1-2: Analytics dashboard
# Day 3: Settings dialog
# Day 4: Status bar
# Day 5: Keyboard shortcuts
```

### Week 5: Polish
```bash
# Day 1: Testing all features
# Day 2: Performance optimization
# Day 3: Cross-platform testing
# Day 4: Build and package
# Day 5: Documentation
```

## 📊 Success Criteria

### Functional Requirements
- [ ] All Dioxus features working in Tauri
- [ ] No regression in consensus accuracy
- [ ] Terminal fully functional
- [ ] LazyGit integrated
- [ ] File operations working
- [ ] Settings persist

### Performance Requirements
- [ ] Binary size < 3MB
- [ ] Memory usage < 45MB
- [ ] Startup time < 400ms
- [ ] No UI freezing during consensus
- [ ] 60 FPS during normal operation

### User Experience
- [ ] Keyboard shortcuts working
- [ ] Window state persists
- [ ] Dark/light theme switching
- [ ] Smooth animations
- [ ] Native feel on all platforms

## 🔨 Build & Test Commands

```bash
# Development
npm run tauri:dev

# Build for production
npm run tauri:build

# Run tests
npm test
cargo test

# Check bundle size
du -sh src-tauri/target/release/bundle/

# Profile performance
npm run tauri:build -- --release
/usr/bin/time -l ./src-tauri/target/release/hive-consensus
```

## 🎯 The Result

After completing this migration, you'll have:

1. **Exact same features** as hive-consensus GUI
2. **5-7x smaller** binary (2-3MB vs 15MB)
3. **Better performance** (30-45MB RAM vs 55MB)
4. **Cross-platform** including mobile
5. **Modern tech stack** (React + Rust)
6. **Production ready** with auto-updates

The key is methodical execution - each week builds on the previous, ensuring we maintain feature parity while gaining all the benefits of Tauri 2.0.