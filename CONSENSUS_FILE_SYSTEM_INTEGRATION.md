# Consensus File System Integration Plan
*Creating a Claude Code-Style Repository-Aware AI Assistant*

## 🎯 Vision Statement

Transform Hive Consensus into a **Claude Code-style AI assistant** that specializes in the currently open repository, maintains persistent learning about codebases, and can autonomously execute developer-guided tasks. Like Claude Code, it won't automatically do things but will have exceptional capability when asked.

**Key Capabilities:**
- **Repository Specialization**: Deep understanding of the current repo and active file context
- **Persistent Learning**: "Learn this entire codebase" and remember it for future questions
- **Self-Planning**: Create implementation plans and save them as local task lists
- **Autonomous Execution**: When directed, automatically implement planned changes
- **Developer-Guided**: Always takes lead from developer, never acts without permission

## 🧠 Claude Code-Style Behavior Model

### Repository Context Awareness System
```rust
// src/consensus/repository_intelligence.rs
pub struct RepositoryIntelligence {
    // Current repository context
    current_repo: Option<RepoContext>,
    active_file: Option<PathBuf>,
    open_files: HashMap<PathBuf, FileState>,
    
    // Persistent learning storage
    repo_memory: Arc<Mutex<RepoMemoryStore>>,
    codebase_summary: Option<CodebaseSummary>,
    
    // Planning and execution
    task_planner: TaskPlanner,
    execution_engine: ExecutionEngine,
}

#[derive(Debug, Clone)]
pub struct RepoContext {
    pub root_path: PathBuf,
    pub name: String,
    pub language_breakdown: HashMap<String, f64>, // e.g., "rust": 0.85, "toml": 0.15
    pub architecture_pattern: ArchitecturePattern, // MVC, Clean, Monolithic, etc.
    pub key_files: Vec<KeyFile>,
    pub dependencies: Dependencies,
    pub build_system: BuildSystem,
    pub last_learned: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct CodebaseSummary {
    pub learning_timestamp: SystemTime,
    pub total_files: usize,
    pub total_lines: usize,
    pub architecture_overview: String,
    pub main_components: Vec<Component>,
    pub data_flow: Vec<DataFlow>,
    pub patterns_used: Vec<Pattern>,
    pub entry_points: Vec<EntryPoint>,
    pub test_coverage: TestCoverage,
    pub complexity_analysis: ComplexityAnalysis,
}
```

### Conversation-Style Interactions
Unlike traditional tools, consensus should respond to natural requests:

```rust
// Example interactions
"Learn this entire codebase" -> Deep analysis, save persistent summary
"What does the auth module do?" -> Uses learned knowledge + current context  
"Add logging to all database operations" -> Creates plan, asks for permission
"Implement the user dashboard we discussed" -> References previous conversations
"Fix the performance issues in the API" -> Analyzes, plans, executes systematically
```

### Persistent Learning and Memory
```rust
// src/consensus/memory.rs
pub struct RepoMemoryStore {
    // Codebase knowledge
    learned_codebases: HashMap<PathBuf, CodebaseSummary>,
    conversation_context: Vec<ConversationMemory>,
    
    // Implementation patterns learned
    developer_preferences: DeveloperPreferences,
    coding_patterns: Vec<CodingPattern>,
    architectural_decisions: Vec<ArchitecturalDecision>,
}

#[derive(Debug, Clone)]
pub struct ConversationMemory {
    pub timestamp: SystemTime,
    pub repo_path: PathBuf,
    pub topic: String,
    pub key_decisions: Vec<String>,
    pub implementation_notes: Vec<String>,
    pub follow_up_items: Vec<String>,
}

impl RepoMemoryStore {
    // Learning functions
    pub async fn learn_codebase(&mut self, repo_path: &Path) -> Result<CodebaseSummary>;
    pub async fn update_understanding(&mut self, repo_path: &Path, new_info: LearningUpdate);
    
    // Memory retrieval
    pub fn get_codebase_knowledge(&self, repo_path: &Path) -> Option<&CodebaseSummary>;
    pub fn get_relevant_conversations(&self, topic: &str) -> Vec<&ConversationMemory>;
    pub fn get_similar_implementations(&self, pattern: &str) -> Vec<ImplementationExample>;
}
```

### Self-Planning and Task Management
```rust
// src/consensus/planning.rs
pub struct TaskPlanner {
    current_plans: HashMap<String, ImplementationPlan>,
    execution_queue: VecDeque<PlanStep>,
    developer_approval_required: bool,
}

#[derive(Debug, Clone)]
pub struct ImplementationPlan {
    pub id: String,
    pub title: String,
    pub description: String,
    pub phases: Vec<Phase>,
    pub estimated_effort: EstimatedEffort,
    pub dependencies: Vec<String>,
    pub risk_assessment: RiskAssessment,
    pub created_at: SystemTime,
    pub status: PlanStatus,
}

#[derive(Debug, Clone)]
pub struct Phase {
    pub name: String,
    pub steps: Vec<PlanStep>,
    pub deliverables: Vec<Deliverable>,
    pub verification_criteria: Vec<VerificationCriteria>,
}

#[derive(Debug, Clone)]
pub struct PlanStep {
    pub action: ActionType,
    pub target_files: Vec<PathBuf>,
    pub description: String,
    pub depends_on: Vec<String>,
    pub estimated_time: Duration,
}

impl TaskPlanner {
    // Planning functions
    pub async fn create_implementation_plan(&mut self, request: &str, context: &RepoContext) -> Result<ImplementationPlan>;
    pub async fn save_plan_to_repo(&self, plan: &ImplementationPlan, repo_path: &Path) -> Result<PathBuf>;
    pub async fn break_down_complex_task(&self, task: &str) -> Result<Vec<PlanStep>>;
    
    // Execution coordination
    pub async fn request_execution_approval(&self, plan: &ImplementationPlan) -> Result<ExecutionApproval>;
    pub async fn execute_plan_step(&mut self, step: &PlanStep) -> Result<StepResult>;
}
```

## 🏗️ Core Architecture

### File System Operations Module
```rust
// src/consensus/file_system.rs
pub struct ConsensusFileSystem {
    // Context awareness
    current_repository: Option<RepositoryContext>,
    open_files: HashMap<String, OpenFileContext>,
    ide_state: Arc<Mutex<IdeState>>,
    
    // Security and permissions
    security_policy: SecurityPolicy,
    operation_log: Vec<FileOperation>,
    
    // User interaction
    approval_handler: Box<dyn ApprovalHandler>,
}

#[derive(Debug, Clone)]
pub struct RepositoryContext {
    pub root_path: PathBuf,
    pub git_info: Option<GitInfo>,
    pub project_type: ProjectType, // Rust, TypeScript, Python, etc.
    pub important_files: Vec<PathBuf>, // README, Cargo.toml, package.json, etc.
    pub ignore_patterns: Vec<String>, // .gitignore, .hiveignore
}

#[derive(Debug, Clone)]
pub struct OpenFileContext {
    pub path: PathBuf,
    pub content: String,
    pub language: Option<String>,
    pub is_dirty: bool,
    pub last_modified: SystemTime,
}
```

### Generator Stage Capabilities (Read-Only)
```rust
impl GeneratorFileCapabilities {
    // File reading operations (like Claude Code's Read tool)
    pub async fn read_file(&self, path: &Path) -> Result<FileContent>;
    pub async fn read_file_range(&self, path: &Path, start: usize, end: usize) -> Result<String>;
    
    // Directory operations (like Claude Code's LS tool)
    pub async fn list_directory(&self, path: &Path) -> Result<Vec<DirEntry>>;
    pub async fn explore_directory_tree(&self, path: &Path, max_depth: usize) -> Result<DirectoryTree>;
    
    // Search operations (like Claude Code's Grep/Glob tools)
    pub async fn search_files(&self, pattern: &str, file_pattern: Option<&str>) -> Result<Vec<SearchMatch>>;
    pub async fn find_files(&self, glob_pattern: &str) -> Result<Vec<PathBuf>>;
    pub async fn grep_content(&self, pattern: &str, paths: &[PathBuf]) -> Result<Vec<GrepMatch>>;
    
    // Repository analysis
    pub async fn analyze_project_structure(&self) -> Result<ProjectAnalysis>;
    pub async fn get_git_status(&self) -> Result<GitStatus>;
    pub async fn get_dependencies(&self) -> Result<Dependencies>;
    
    // Context awareness
    pub async fn get_open_files(&self) -> Result<Vec<OpenFileContext>>;
    pub async fn get_related_files(&self, file_path: &Path) -> Result<Vec<PathBuf>>;
    pub async fn get_file_imports(&self, file_path: &Path) -> Result<Vec<ImportInfo>>;
}
```

### Curator Stage Capabilities (Read + Write)
```rust
impl CuratorFileCapabilities {
    // Include all Generator capabilities PLUS:
    
    // File modification (like Claude Code's Edit/MultiEdit tools)
    pub async fn edit_file(&self, path: &Path, edits: Vec<Edit>) -> Result<EditResult>;
    pub async fn multi_edit_files(&self, edits: HashMap<PathBuf, Vec<Edit>>) -> Result<MultiEditResult>;
    pub async fn replace_file_content(&self, path: &Path, new_content: &str) -> Result<()>;
    
    // File creation (like Claude Code's Write tool)
    pub async fn create_file(&self, path: &Path, content: &str) -> Result<()>;
    pub async fn create_directory(&self, path: &Path) -> Result<()>;
    pub async fn create_file_tree(&self, base_path: &Path, structure: FileTree) -> Result<()>;
    
    // File operations
    pub async fn delete_file(&self, path: &Path) -> Result<()>;
    pub async fn move_file(&self, from: &Path, to: &Path) -> Result<()>;
    pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<()>;
    
    // Advanced operations
    pub async fn refactor_rename(&self, old_name: &str, new_name: &str, scope: RefactorScope) -> Result<RefactorResult>;
    pub async fn apply_code_template(&self, template: &CodeTemplate, target_path: &Path) -> Result<()>;
    pub async fn extract_function(&self, file_path: &Path, selection: TextRange, new_name: &str) -> Result<ExtractionResult>;
}
```

## 🔐 Security & Safety Model

### Three-Tier Security System

#### Tier 1: Path Whitelisting
```rust
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    // Allowed base paths
    pub allowed_roots: Vec<PathBuf>,
    
    // Explicitly blocked paths
    pub blocked_paths: Vec<PathBuf>,
    
    // File type restrictions
    pub allowed_extensions: Option<HashSet<String>>,
    pub blocked_extensions: HashSet<String>,
    
    // Operation restrictions
    pub generator_permissions: GeneratorPermissions,
    pub curator_permissions: CuratorPermissions,
}

#[derive(Debug, Clone)]
pub struct GeneratorPermissions {
    pub can_read_files: bool,
    pub can_list_directories: bool,
    pub can_search_content: bool,
    pub max_file_size: usize, // Bytes
    pub max_files_per_operation: usize,
}

#[derive(Debug, Clone)]
pub struct CuratorPermissions {
    pub can_modify_files: bool,
    pub can_create_files: bool,
    pub can_delete_files: bool,
    pub requires_approval_for: Vec<OperationType>,
    pub backup_before_modify: bool,
}
```

#### Tier 2: User Approval System
```rust
#[derive(Debug, Clone)]
pub enum ApprovalRequired {
    Never,
    Always,
    ForDestructiveOps,
    ForFilesOutsideProject,
    ForSystemFiles,
}

pub trait ApprovalHandler: Send + Sync {
    async fn request_approval(&self, operation: &FileOperation) -> Result<ApprovalDecision>;
    async fn show_preview(&self, operation: &FileOperation) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum ApprovalDecision {
    Approve,
    Reject,
    ApproveWithModifications(FileOperation),
    RequestMoreInfo,
}
```

#### Tier 3: Operation Logging & Rollback
```rust
#[derive(Debug, Clone)]
pub struct FileOperation {
    pub id: Uuid,
    pub operation_type: OperationType,
    pub timestamp: SystemTime,
    pub stage: ConsensusStage, // Generator or Curator
    pub target_paths: Vec<PathBuf>,
    pub description: String,
    pub backup_info: Option<BackupInfo>,
    pub status: OperationStatus,
}

pub struct RollbackManager {
    operations: Vec<FileOperation>,
    backups: HashMap<Uuid, BackupData>,
}

impl RollbackManager {
    pub async fn create_backup(&self, paths: &[PathBuf]) -> Result<BackupInfo>;
    pub async fn rollback_operation(&self, operation_id: Uuid) -> Result<()>;
    pub async fn rollback_to_timestamp(&self, timestamp: SystemTime) -> Result<()>;
}
```

## 🎮 User Interface Integration

### 1. File Operations Panel
```rust
#[component]
pub fn FileOperationsPanel() -> Element {
    let operations = use_signal(|| Vec::<FileOperation>::new());
    let show_approval_dialog = use_signal(|| false);
    let pending_operation = use_signal(|| Option::<FileOperation>::None);
    
    rsx! {
        div { class: "file-operations-panel",
            // Header with controls
            div { class: "operations-header",
                h3 { "Consensus File Operations" }
                button { 
                    onclick: move |_| toggle_auto_approve(),
                    "Auto-approve safe operations"
                }
            }
            
            // Live operations feed
            div { class: "operations-feed",
                for operation in operations.read().iter() {
                    FileOperationItem { operation: operation.clone() }
                }
            }
            
            // Approval dialog
            if *show_approval_dialog.read() {
                ApprovalDialog { 
                    operation: pending_operation.read().clone(),
                    on_decision: move |decision| handle_approval_decision(decision),
                }
            }
        }
    }
}
```

### 2. Repository Context Display
```rust
#[component]
pub fn RepositoryContextWidget() -> Element {
    let repo_context = use_signal(|| Option::<RepositoryContext>::None);
    let open_files = use_signal(|| Vec::<OpenFileContext>::new());
    
    rsx! {
        div { class: "repo-context-widget",
            // Current repository info
            if let Some(repo) = repo_context.read().as_ref() {
                div { class: "current-repo",
                    span { class: "repo-icon", "📁" }
                    span { class: "repo-name", "{repo.root_path.file_name().unwrap_or_default().to_str().unwrap_or("Unknown")}" }
                    span { class: "repo-path", "{repo.root_path.display()}" }
                }
            }
            
            // Open files context
            div { class: "open-files-context",
                h4 { "Files Available to Consensus" }
                for file in open_files.read().iter() {
                    div { class: "context-file",
                        span { class: "file-icon", get_file_icon(&file.path) }
                        span { class: "file-name", "{file.path.file_name().unwrap_or_default().to_str().unwrap_or("Unknown")}" }
                        if file.is_dirty {
                            span { class: "dirty-indicator", "●" }
                        }
                    }
                }
            }
        }
    }
}
```

### 3. File Change Preview
```rust
#[component]
pub fn FileChangePreview(operation: FileOperation) -> Element {
    let show_diff = use_signal(|| false);
    
    rsx! {
        div { class: "file-change-preview",
            // Operation summary
            div { class: "operation-summary",
                h3 { "{operation.operation_type:?}: {operation.description}" }
                div { class: "affected-files",
                    "Affects {operation.target_paths.len()} file(s)"
                }
            }
            
            // File-by-file changes
            for path in operation.target_paths.iter() {
                div { class: "file-change",
                    div { class: "file-header",
                        span { class: "file-path", "{path.display()}" }
                        button { 
                            onclick: move |_| show_diff.set(!*show_diff.read()),
                            if *show_diff.read() { "Hide Diff" } else { "Show Diff" }
                        }
                    }
                    
                    if *show_diff.read() {
                        DiffViewer { 
                            file_path: path.clone(),
                            operation: operation.clone(),
                        }
                    }
                }
            }
            
            // Action buttons
            div { class: "preview-actions",
                button { 
                    class: "approve-btn",
                    onclick: move |_| approve_operation(&operation),
                    "Approve Changes"
                }
                button { 
                    class: "reject-btn", 
                    onclick: move |_| reject_operation(&operation),
                    "Reject"
                }
                button { 
                    class: "modify-btn",
                    onclick: move |_| show_modification_dialog(&operation),
                    "Modify"
                }
            }
        }
    }
}
```

## ⚙️ Consensus Pipeline Integration

### Generator Stage Enhancement
```rust
// src/consensus/stages/generator.rs
impl GeneratorStage {
    pub async fn generate_with_file_context(
        &self,
        prompt: &str,
        context: &GenerationContext,
    ) -> Result<GeneratorOutput> {
        let mut enhanced_context = context.clone();
        
        // 1. Analyze current repository
        if let Some(repo_context) = self.file_system.get_repository_context().await? {
            enhanced_context.repository = Some(repo_context);
        }
        
        // 2. Gather relevant files based on prompt
        let relevant_files = self.find_relevant_files(prompt).await?;
        for file_path in relevant_files {
            let content = self.file_system.read_file(&file_path).await?;
            enhanced_context.add_file_context(file_path, content);
        }
        
        // 3. Analyze project structure if needed
        if self.should_analyze_project_structure(prompt) {
            let project_analysis = self.file_system.analyze_project_structure().await?;
            enhanced_context.project_analysis = Some(project_analysis);
        }
        
        // 4. Get open files context
        let open_files = self.file_system.get_open_files().await?;
        enhanced_context.open_files = open_files;
        
        // 5. Generate with enhanced context
        self.generate_internal(prompt, &enhanced_context).await
    }
    
    async fn find_relevant_files(&self, prompt: &str) -> Result<Vec<PathBuf>> {
        let mut relevant_files = Vec::new();
        
        // Extract file mentions from prompt
        let mentioned_files = extract_file_mentions(prompt);
        for file_mention in mentioned_files {
            if let Some(path) = self.resolve_file_path(&file_mention).await? {
                relevant_files.push(path);
            }
        }
        
        // Search for relevant content
        let keywords = extract_keywords(prompt);
        for keyword in keywords {
            let matches = self.file_system.search_files(&keyword, None).await?;
            relevant_files.extend(matches.into_iter().map(|m| m.path));
        }
        
        // Limit to prevent context overflow
        relevant_files.truncate(20);
        Ok(relevant_files)
    }
}
```

### Curator Stage Enhancement
```rust
// src/consensus/stages/curator.rs
impl CuratorStage {
    pub async fn curate_with_file_operations(
        &self,
        input: &CuratorInput,
    ) -> Result<CuratorOutput> {
        let mut output = CuratorOutput::new();
        
        // 1. Parse file operations from curator input
        let file_operations = self.parse_file_operations(input).await?;
        
        // 2. Validate operations against security policy
        for operation in &file_operations {
            self.validate_operation(operation).await?;
        }
        
        // 3. Request user approval for dangerous operations
        let approved_operations = self.request_approvals(file_operations).await?;
        
        // 4. Execute operations with rollback support
        let execution_results = self.execute_operations(approved_operations).await?;
        
        // 5. Update IDE state
        self.update_ide_state(&execution_results).await?;
        
        // 6. Return results
        output.file_operations = execution_results;
        Ok(output)
    }
    
    async fn parse_file_operations(&self, input: &CuratorInput) -> Result<Vec<FileOperation>> {
        let mut operations = Vec::new();
        
        // Parse different operation types from curator output
        if let Some(file_edits) = &input.file_edits {
            for (path, edits) in file_edits {
                operations.push(FileOperation {
                    id: Uuid::new_v4(),
                    operation_type: OperationType::EditFile,
                    target_paths: vec![path.clone()],
                    description: format!("Edit {} with {} changes", path.display(), edits.len()),
                    ..Default::default()
                });
            }
        }
        
        if let Some(new_files) = &input.new_files {
            for (path, content) in new_files {
                operations.push(FileOperation {
                    id: Uuid::new_v4(),
                    operation_type: OperationType::CreateFile,
                    target_paths: vec![path.clone()],
                    description: format!("Create new file {}", path.display()),
                    ..Default::default()
                });
            }
        }
        
        Ok(operations)
    }
}
```

## 📊 Context Sharing & State Management

### Shared Context Store
```rust
// src/consensus/context.rs
#[derive(Debug, Clone)]
pub struct ConsensusContext {
    // Repository context
    pub repository: Option<RepositoryContext>,
    pub open_files: Vec<OpenFileContext>,
    pub project_analysis: Option<ProjectAnalysis>,
    
    // File operation history
    pub recent_operations: Vec<FileOperation>,
    pub operation_results: HashMap<Uuid, OperationResult>,
    
    // User preferences
    pub auto_approval_settings: AutoApprovalSettings,
    pub security_policy: SecurityPolicy,
    
    // IDE integration
    pub active_tab: Option<String>,
    pub selected_files: Vec<PathBuf>,
    pub editor_state: EditorState,
}

impl ConsensusContext {
    pub fn update_from_ide(&mut self, ide_state: &IdeState) {
        self.open_files = ide_state.get_open_files();
        self.active_tab = ide_state.get_active_tab();
        self.selected_files = ide_state.get_selected_files();
    }
    
    pub fn get_file_content(&self, path: &Path) -> Option<&str> {
        self.open_files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.content.as_str())
    }
}
```

### IDE State Synchronization
```rust
// src/desktop/ide_state.rs
pub struct IdeState {
    pub file_explorer: FileExplorerState,
    pub editor_tabs: EditorTabsState,
    pub consensus_state: ConsensusState,
    pub file_operations: Arc<Mutex<Vec<FileOperation>>>,
}

impl IdeState {
    pub async fn sync_with_consensus(&mut self, context: &ConsensusContext) {
        // Update file explorer if files were created/deleted
        for operation in &context.recent_operations {
            match operation.operation_type {
                OperationType::CreateFile | OperationType::CreateDirectory => {
                    self.file_explorer.refresh_path(&operation.target_paths[0]).await;
                }
                OperationType::DeleteFile => {
                    self.file_explorer.remove_path(&operation.target_paths[0]);
                }
                OperationType::EditFile => {
                    self.refresh_open_file(&operation.target_paths[0]).await;
                }
                _ => {}
            }
        }
        
        // Update editor tabs if files were modified
        for operation in &context.recent_operations {
            if matches!(operation.operation_type, OperationType::EditFile) {
                for path in &operation.target_paths {
                    if let Some(tab) = self.editor_tabs.find_tab_by_path(path) {
                        tab.reload_from_disk().await?;
                    }
                }
            }
        }
    }
}
```

## 🚀 Implementation Phases

### Phase 1: Repository Intelligence Foundation (Week 1-2)
1. **Repository Context System** (`src/consensus/repository_intelligence.rs`)
   - Automatic repo detection and analysis
   - Language pattern recognition  
   - Architecture pattern detection
   - Integration with IDE state (current active file, open tabs)

2. **Basic Learning Infrastructure** (`src/consensus/memory.rs`)
   - Persistent storage for codebase summaries
   - File reading and analysis capabilities
   - Basic conversation memory

3. **IDE Integration**
   - Repository context widget showing current repo awareness
   - Active file tracking
   - Open files context display

**Milestone**: "What files are in this project?" and "What's the current active file?" work perfectly

### Phase 2: Codebase Learning System (Week 3-4)
1. **Deep Codebase Analysis**
   - Implement "Learn this entire codebase" command
   - Comprehensive project structure analysis
   - Dependency mapping and data flow analysis
   - Code pattern recognition

2. **Persistent Memory System**
   - Save/load codebase summaries to local storage
   - Conversation context preservation
   - Developer preference learning

3. **Knowledge Retrieval**
   - Context-aware responses using learned knowledge
   - "What does the auth module do?" type queries
   - Related file suggestions

**Milestone**: "Learn this codebase" creates comprehensive summary that persists across sessions

### Phase 3: Task Planning and Execution (Week 5-6)
1. **Self-Planning System** (`src/consensus/planning.rs`)
   - Natural language to implementation plan conversion
   - Automatic task breakdown
   - Save plans as local markdown files in repo

2. **Execution Engine**
   - Step-by-step plan execution with developer approval
   - File modification capabilities integrated with planning
   - Progress tracking and rollback support

3. **Developer Workflow Integration**
   - Plan saving in repo (like `.hive/plans/feature-auth.md`)
   - Approval dialogs for each plan step
   - Integration with existing consensus pipeline

**Milestone**: "Add authentication to this app" creates detailed plan and executes with approval

### Phase 4: Advanced Capabilities (Week 7-8)
1. **Multi-File Operations**
   - Complex refactoring across multiple files
   - Project-wide changes and migrations
   - Intelligent code generation based on existing patterns

2. **Conversation Continuity**
   - Reference previous conversations and decisions
   - Build on previous work and context
   - Learn from user feedback and preferences

3. **Claude Code-Style Polish**
   - Natural conversation flow
   - Helpful error messages and guidance
   - Performance optimization for large codebases

**Milestone**: Full Claude Code-style experience with repository specialization

## 🎯 Success Metrics

### Functional Goals
- [ ] Generator can read and understand entire repositories
- [ ] Curator can make sophisticated multi-file changes
- [ ] All operations are properly secured and approved
- [ ] IDE state stays synchronized with file changes
- [ ] Users can easily control and monitor file operations

### Performance Goals
- [ ] File operations complete within 2 seconds for normal files
- [ ] Repository analysis completes within 10 seconds for typical projects
- [ ] UI remains responsive during file operations
- [ ] Memory usage stays reasonable for large repositories

### Security Goals
- [ ] No unauthorized file access possible
- [ ] All destructive operations require explicit approval
- [ ] Complete audit trail of all operations
- [ ] Ability to rollback any changes
- [ ] Security policy is easily configurable

## 🔄 Integration with Existing Systems

### Consensus Pipeline
- Enhance existing generator and curator stages
- Maintain backward compatibility
- Add new file operation output types
- Preserve existing prompt processing

### IDE Components
- Integrate with existing file explorer
- Sync with tab system
- Work with existing editor components
- Preserve HiveTechs styling and UX

### Database Integration
- Log operations to activity_logs table
- Store approval preferences
- Track file operation metrics
- Integrate with existing analytics

## 📝 Configuration Example

```toml
# ~/.hive/file_operations.toml
[security]
# Allowed base directories
allowed_roots = [
    "~/Developer",
    "~/Projects", 
    "/tmp/hive-workspace"
]

# Always block these paths
blocked_paths = [
    "/System",
    "/usr/bin",
    "~/.ssh",
    "~/.hive/api_keys.toml"
]

# File type restrictions
blocked_extensions = ["exe", "dmg", "pkg"]

[generator_permissions]
can_read_files = true
can_list_directories = true
can_search_content = true
max_file_size = "10MB"
max_files_per_operation = 50

[curator_permissions]
can_modify_files = true
can_create_files = true
can_delete_files = false  # Require explicit approval
requires_approval_for = ["delete", "move", "large_changes"]
backup_before_modify = true

[auto_approval]
# Auto-approve safe operations
safe_extensions = ["md", "txt", "json", "toml"]
safe_operations = ["read", "small_edits"]
max_auto_approve_size = "1MB"
```

## 💬 Claude Code-Style User Experience

### Natural Conversation Flow
```
Developer: "Learn this entire codebase"
Consensus: "I'll analyze the entire repository structure, dependencies, and code patterns. This may take a few minutes for a large codebase."

[Analysis progress shown in UI]

Consensus: "✅ Codebase analysis complete! I've learned about:
- 47 Rust files with a desktop GUI architecture using Dioxus
- Main components: Editor, File Explorer, Consensus Pipeline, TUI Interface  
- Key patterns: Component-based UI, async/await for file operations
- Entry points: main.rs (CLI) and desktop module
- Current focus: VS Code-style code editor with syntax highlighting

I've saved this summary for future conversations. What would you like to know or work on?"
```

### Context-Aware Responses
```
Developer: "What does the auth module do?"
Consensus: "Based on my analysis of the codebase, I don't see a dedicated auth module yet. However, I notice you have:
- A security system planned in SECURITY_SYSTEM.md
- Trust dialogs mentioned in the desktop module
- User preferences in the config system

Would you like me to create an authentication system? I can design it to fit your existing architecture patterns."
```

### Self-Planning Capabilities  
```
Developer: "Add comprehensive logging to the entire application"
Consensus: "I'll create a detailed implementation plan for adding logging throughout the application. Let me analyze the current codebase structure and create a plan..."

[Creates and saves /Users/veronelazio/Developer/Private/hive/.hive/plans/comprehensive-logging.md]

Consensus: "✅ Created implementation plan: comprehensive-logging.md

Plan Summary:
1. Add logging infrastructure (tracing crate)
2. Add logging to consensus pipeline (4 stages)  
3. Add logging to desktop UI components
4. Add logging to file operations
5. Add log level configuration
6. Add log file rotation

Estimated effort: 6-8 hours across 23 files
Would you like me to start implementing this plan? I'll request approval for each phase."
```

### Autonomous Execution with Approval
```
Developer: "Yes, implement the logging plan"
Consensus: "Starting Phase 1: Logging Infrastructure

I need to:
- Add `tracing` and `tracing-subscriber` to Cargo.toml
- Create src/logging.rs module
- Initialize logging in main.rs

Proceed with these changes? [Approve] [Modify] [Cancel]"

Developer: [Approve]
Consensus: "✅ Phase 1 complete. Logging infrastructure added.
✅ Tests passing, build successful.

Phase 2: Consensus Pipeline Logging
I'll add structured logging to all 4 consensus stages with correlation IDs for tracking requests through the pipeline.

Proceed? [Approve] [Modify] [Cancel]"
```

## 🎉 Revolutionary Capabilities Unlocked

### Claude Code-Style Commands
- **"Learn this entire codebase"** → Deep analysis with persistent memory
- **"What does the [module] do?"** → Context-aware explanations using learned knowledge
- **"Add [feature] to this application"** → Creates plan, saves to repo, executes with approval
- **"Refactor this to use [pattern]"** → Multi-file refactoring with existing code style
- **"Fix the performance issues"** → Analyzes, plans systematically, implements solutions
- **"Add tests for everything"** → Comprehensive test generation matching existing patterns
- **"Update all documentation"** → Project-wide documentation updates
- **"Migrate from [X] to [Y]"** → Complex technology migrations with safety

### Repository Specialization Benefits
- **Understands your specific codebase** rather than giving generic advice
- **Learns your coding patterns** and follows them consistently  
- **Remembers previous conversations** and builds on past decisions
- **Knows your project structure** and suggests appropriate file locations
- **Aware of your dependencies** and uses existing libraries
- **Maintains conversation context** across multiple sessions

### Developer-Guided Workflow
- **Never acts without permission** - always requests approval for changes
- **Creates implementation plans** before making any modifications
- **Saves plans in your repository** for review and future reference
- **Executes step-by-step** with verification at each phase
- **Learns from your feedback** and improves suggestions over time
- **Preserves your development style** while adding AI capabilities

This transforms Hive from a sophisticated AI assistant into a **true AI development partner** that specializes in your specific repository, learns from your codebase, and can autonomously execute complex tasks under your guidance - just like having Claude Code integrated directly into your IDE.