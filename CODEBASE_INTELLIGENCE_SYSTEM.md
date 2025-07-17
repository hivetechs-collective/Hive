# Codebase Intelligence System Design

## Overview

The `@codebase` command triggers a comprehensive analysis of the entire repository, creating a searchable, semantic index that gives consensus AI stages complete awareness of the codebase structure, relationships, and purpose.

## Core Concept

When a user types `@codebase` in the consensus chat, the system:
1. Performs a deep scan of every file in the repository
2. Parses and understands the code structure
3. Extracts all objects (functions, classes, types, etc.)
4. Analyzes relationships between files and components
5. Derives the architectural patterns and design
6. Stores everything in a searchable index
7. Makes this knowledge available to all 4 consensus stages

## System Architecture

### 1. Command Detection
```rust
// In consensus message processing
if message.starts_with("@codebase") {
    trigger_codebase_analysis(&repository_path).await?;
}
```

### 2. Codebase Scanner
```rust
pub struct CodebaseScanner {
    repository_path: PathBuf,
    file_reader: Arc<FileReader>,
    progress_callback: Arc<dyn ProgressCallback>,
}

impl CodebaseScanner {
    pub async fn scan_repository(&self) -> Result<RepositoryScan> {
        // 1. Build complete file tree
        // 2. Read every file
        // 3. Detect languages and frameworks
        // 4. Extract raw content for parsing
    }
}
```

### 3. AST-Based Object Extraction
```rust
pub struct ObjectExtractor {
    parsers: HashMap<Language, Box<dyn LanguageParser>>,
}

pub struct ExtractedObject {
    pub id: String,                    // Unique identifier
    pub name: String,                  // Object name
    pub kind: ObjectKind,              // Function, Class, Type, etc.
    pub file_path: PathBuf,           
    pub line_start: usize,
    pub line_end: usize,
    pub signature: String,             // Full signature
    pub documentation: Option<String>, // Doc comments
    pub visibility: Visibility,        // Public, Private, etc.
    pub dependencies: Vec<String>,     // What it uses
    pub dependents: Vec<String>,       // What uses it
    pub context: String,               // Why it exists (derived)
}

pub enum ObjectKind {
    Function,
    Class,
    Interface,
    Type,
    Enum,
    Module,
    Variable,
    Constant,
    // ... more
}
```

### 4. Relationship Analyzer
```rust
pub struct RelationshipAnalyzer {
    objects: HashMap<String, ExtractedObject>,
    graph: DiGraph<String, RelationType>,
}

pub enum RelationType {
    Imports,
    Extends,
    Implements,
    Uses,
    UsedBy,
    Contains,
    ContainedBy,
    Tests,
    TestedBy,
}

impl RelationshipAnalyzer {
    pub fn analyze_relationships(&mut self) -> Result<()> {
        // Build dependency graph
        // Identify architectural patterns
        // Detect circular dependencies
        // Find entry points
        // Map data flow
    }
    
    pub fn derive_context(&self, object_id: &str) -> String {
        // Use graph analysis to understand WHY an object exists
        // Consider its relationships, usage patterns, etc.
    }
}
```

### 5. Architecture Deriver
```rust
pub struct ArchitectureDeriver {
    pub patterns: Vec<ArchitecturalPattern>,
    pub layers: Vec<Layer>,
    pub components: Vec<Component>,
    pub entry_points: Vec<EntryPoint>,
}

pub enum ArchitecturalPattern {
    MVC,
    Microservices,
    Monolithic,
    EventDriven,
    Layered,
    Pipeline,
    // ... more
}

impl ArchitectureDeriver {
    pub fn derive_architecture(&self, objects: &[ExtractedObject], relationships: &RelationshipGraph) -> Architecture {
        // Identify design patterns
        // Detect architectural style
        // Map component boundaries
        // Understand data flow
    }
}
```

### 6. Keyword and Concept Extraction
```rust
pub struct ConceptExtractor {
    pub domain_keywords: HashSet<String>,
    pub technical_terms: HashSet<String>,
    pub business_concepts: HashSet<String>,
}

impl ConceptExtractor {
    pub fn extract_keywords(&self, content: &str, context: &ExtractedObject) -> Keywords {
        // Extract domain-specific terms
        // Identify technical concepts
        // Find business logic keywords
        // Weight by importance/frequency
    }
}
```

### 7. Storage Schema
```sql
-- Core tables for codebase intelligence
CREATE TABLE codebase_scans (
    id TEXT PRIMARY KEY,
    repository_path TEXT NOT NULL,
    scan_timestamp TIMESTAMP NOT NULL,
    total_files INTEGER,
    total_objects INTEGER,
    architecture_summary TEXT,
    is_current BOOLEAN DEFAULT TRUE
);

CREATE TABLE indexed_objects (
    id TEXT PRIMARY KEY,
    scan_id TEXT NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_start INTEGER,
    line_end INTEGER,
    signature TEXT,
    documentation TEXT,
    visibility TEXT,
    context TEXT, -- Why this exists
    embedding BLOB, -- Vector embedding for semantic search
    FOREIGN KEY (scan_id) REFERENCES codebase_scans(id)
);

CREATE TABLE object_relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    from_object_id TEXT NOT NULL,
    to_object_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    strength REAL DEFAULT 1.0,
    FOREIGN KEY (scan_id) REFERENCES codebase_scans(id)
);

CREATE TABLE indexed_keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_id TEXT NOT NULL,
    keyword TEXT NOT NULL,
    category TEXT, -- domain, technical, business
    frequency INTEGER,
    importance REAL,
    related_objects TEXT, -- JSON array of object IDs
    FOREIGN KEY (scan_id) REFERENCES codebase_scans(id)
);

-- Full-text search indices
CREATE VIRTUAL TABLE objects_fts USING fts5(
    name, signature, documentation, context,
    content=indexed_objects
);

CREATE INDEX idx_objects_kind ON indexed_objects(kind);
CREATE INDEX idx_keywords_keyword ON indexed_keywords(keyword);
CREATE INDEX idx_relationships_from_to ON object_relationships(from_object_id, to_object_id);
```

### 8. Search Interface
```rust
pub struct CodebaseSearch {
    database: Arc<DatabaseManager>,
}

impl CodebaseSearch {
    pub async fn search(&self, query: &str) -> Result<SearchResults> {
        // Full-text search across objects
        // Semantic search using embeddings
        // Graph traversal for relationships
        // Keyword matching
    }
    
    pub async fn find_by_concept(&self, concept: &str) -> Result<Vec<ExtractedObject>> {
        // Find all objects related to a business concept
    }
    
    pub async fn get_architecture_context(&self) -> Result<Architecture> {
        // Return the full architectural understanding
    }
    
    pub async fn explain_object(&self, object_id: &str) -> Result<ObjectExplanation> {
        // Explain what an object does and why it exists
        // Include its relationships and usage
    }
}
```

### 9. Consensus Integration
```rust
// Enhanced repository context with full codebase intelligence
pub struct IntelligentRepositoryContext {
    basic_context: RepositoryContext,
    codebase_index: Arc<CodebaseSearch>,
    current_scan: CodebaseScan,
}

impl IntelligentRepositoryContext {
    pub async fn get_context_for_question(&self, question: &str) -> Result<String> {
        // 1. Extract key terms from question
        let terms = extract_search_terms(question);
        
        // 2. Search indexed objects
        let relevant_objects = self.codebase_index.search(&terms).await?;
        
        // 3. Get architectural context
        let architecture = self.codebase_index.get_architecture_context().await?;
        
        // 4. Build comprehensive context
        let mut context = String::new();
        context.push_str("## 🧠 INTELLIGENT CODEBASE CONTEXT\n\n");
        context.push_str(&format!("Architecture: {:?}\n", architecture.pattern));
        context.push_str(&format!("Total Objects: {}\n", self.current_scan.total_objects));
        context.push_str("\n### Relevant Components:\n");
        
        for obj in relevant_objects.iter().take(10) {
            context.push_str(&format!(
                "- {} `{}` at {}:{}\n  Purpose: {}\n",
                obj.kind, obj.name, obj.file_path.display(), obj.line_start, obj.context
            ));
        }
        
        Ok(context)
    }
}
```

## User Experience

### Running @codebase Analysis
```
User: @codebase

Consensus: 🔍 Starting deep codebase analysis...
📂 Scanning 2,847 files...
📊 Extracting 15,423 objects...
🔗 Analyzing 45,892 relationships...
🏗️ Deriving architecture...
💾 Indexing for search...

✅ Codebase analysis complete!

📈 Summary:
- Architecture: Event-driven microservices
- Languages: Rust (78%), TypeScript (15%), SQL (7%)
- Total Objects: 15,423
  - Functions: 8,234
  - Types: 3,456
  - Modules: 892
  - Classes: 2,841
- Key Concepts: consensus, pipeline, repository, analysis
- Entry Points: main.rs, cli.rs, web_server.rs

🧠 I now have complete understanding of your codebase!
You can ask me anything about your code, architecture, or specific components.
```

### Using the Intelligence
```
User: How does the consensus pipeline work?

Consensus: Based on my analysis of your codebase:

The consensus pipeline (src/consensus/pipeline.rs:35) is a 4-stage AI orchestration system:

1. **Generator Stage** (src/consensus/stages/generator.rs:24)
   - Purpose: Initial response creation
   - Uses: FileReader, RepositoryContext
   - Called by: ConsensusPipeline::run_single_stage

2. **Refiner Stage** (src/consensus/stages/refiner.rs:18)
   - Purpose: Enhances Generator output
   - Relationships: Depends on Generator output
   - Key methods: build_messages, analyze_response_quality

[... detailed explanation with actual code references ...]
```

## Implementation Phases

### Phase 1: Basic Scanning
- File tree traversal
- Language detection
- Basic object extraction (functions, types)

### Phase 2: Deep Analysis
- Full AST parsing
- Relationship mapping
- Architecture detection

### Phase 3: Intelligence Layer
- Context derivation (why things exist)
- Semantic embeddings
- Advanced search

### Phase 4: Real-time Updates
- File watcher integration
- Incremental updates
- Git-aware changes

## Benefits

1. **Complete Awareness**: AI understands every object and its purpose
2. **Accurate Responses**: No hallucination - everything is indexed
3. **Relationship Understanding**: Knows how components interact
4. **Architectural Insight**: Understands the big picture
5. **Searchable Knowledge**: Fast retrieval of relevant information
6. **Dynamic Updates**: Stays current with code changes

## Future Enhancements

1. **Git Integration**: Track how objects evolved over time
2. **Team Knowledge**: Index who wrote what and when
3. **Documentation Sync**: Connect code to external docs
4. **Performance Metrics**: Understand hot paths and bottlenecks
5. **Security Analysis**: Identify potential vulnerabilities
6. **Test Coverage**: Map tests to code they cover