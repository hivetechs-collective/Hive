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

## 🚨 Current Status & Critical Issues

### ✅ Completed Features (Phase 1.1-1.2, 1.5)
- Repository context system (`repository_context.rs`)
- IDE state tracking with current project awareness
- Context flows through all 4 consensus stages
- App starts with no folder open
- Basic repository type detection (Rust, TypeScript, etc.)

### ❌ Critical Issues Discovered from Testing
1. **No File Reading**: Consensus cannot read actual file contents
2. **Generic Responses**: Without file access, responses are generic and inaccurate
3. **Hallucination**: Curator stage invents code examples instead of reading real code
4. **Poor Stage Coordination**: Validator's critiques aren't acted upon by subsequent stages
5. **No Verification**: Stages make assumptions without verifying against actual data

### 📊 Test Results Analysis
When asked to analyze the Hive repository:
- **Generator**: Only identified basic Rust structure (Cargo.toml, src/)
- **Refiner**: Added generic Rust best practices, no specific insights
- **Validator**: Correctly criticized the generic analysis (best stage!)
- **Curator**: Made up fake code examples that don't exist

**Root Cause**: Consensus has no file reading capabilities!

## 🔧 Enhanced Implementation Plan

### Phase 1.3: Repository Context Widget (1 day)
Add visual feedback showing what consensus knows:
```rust
// Visual indicator in UI showing:
- Current repository path and type
- Number of files discovered
- Last analysis timestamp
- Memory status (learned/not learned)
- Active file being edited
```

### Phase 1.4: File Reading Capabilities (CRITICAL - 2-3 days)
**This is the most critical missing piece!**

```rust
// src/consensus/file_operations.rs
pub struct FileReader {
    security_policy: SecurityPolicy,
    cache: FileCache,
}

impl FileReader {
    // Essential reading operations
    pub async fn read_file(&self, path: &Path) -> Result<String>;
    pub async fn read_file_lines(&self, path: &Path, start: usize, end: usize) -> Result<Vec<String>>;
    pub async fn list_directory(&self, path: &Path) -> Result<Vec<DirEntry>>;
    pub async fn glob_files(&self, pattern: &str) -> Result<Vec<PathBuf>>;
    pub async fn search_content(&self, pattern: &str, paths: &[PathBuf]) -> Result<Vec<SearchMatch>>;
}

// Integration with consensus stages
impl ConsensusContext {
    pub async fn read_file_for_analysis(&self, path: &Path) -> Result<FileContent> {
        // Check security policy
        // Read from cache if available
        // Read from disk with size limits
        // Return content with metadata
    }
}
```

### Phase 1.5: Stage Coordination Enhancement (2 days)
Fix the pipeline to properly act on feedback:

```rust
// src/consensus/pipeline.rs
impl ConsensusPipeline {
    // Enhanced stage communication
    pub async fn run_with_verification(&mut self, prompt: &str) -> Result<ConsensusOutput> {
        let mut stage_outputs = Vec::new();
        
        // Generator stage with file reading
        let generator_output = self.generator.generate_with_files(prompt).await?;
        stage_outputs.push(generator_output.clone());
        
        // Refiner stage with verification requirement
        let refiner_output = self.refiner.refine_with_verification(
            &generator_output,
            VerificationRequirement::MustReadActualFiles
        ).await?;
        stage_outputs.push(refiner_output.clone());
        
        // Validator stage with action items
        let validator_output = self.validator.validate_with_actions(
            &refiner_output,
            ValidationMode::RequireSpecificExamples
        ).await?;
        
        // If validator finds issues, loop back
        if validator_output.has_critical_issues() {
            // Force re-reading of actual files
            let verified_output = self.verify_with_file_reading(&validator_output.issues).await?;
            stage_outputs.push(verified_output);
        }
        
        // Curator must use real examples
        let curator_output = self.curator.curate_with_real_code(
            &stage_outputs,
            CurationMode::NoHallucination
        ).await?;
        
        Ok(ConsensusOutput { stages: stage_outputs, final: curator_output })
    }
}
```

### Phase 1.6: Anti-Hallucination System (1 day)
Prevent stages from making up information:

```rust
// src/consensus/verification.rs
pub struct VerificationSystem {
    file_reader: FileReader,
    fact_checker: FactChecker,
}

impl VerificationSystem {
    // Verify all code examples exist
    pub async fn verify_code_examples(&self, response: &str) -> Result<VerificationReport> {
        let code_blocks = extract_code_blocks(response);
        let mut report = VerificationReport::new();
        
        for block in code_blocks {
            if block.claimed_source.is_some() {
                // Verify the code actually exists in the claimed file
                let actual_content = self.file_reader.read_file(&block.claimed_source).await?;
                if !actual_content.contains(&block.content) {
                    report.add_issue(VerificationIssue::FakeCode {
                        claimed_source: block.claimed_source,
                        content: block.content,
                    });
                }
            }
        }
        
        report
    }
    
    // Verify file paths exist
    pub async fn verify_file_references(&self, response: &str) -> Result<Vec<InvalidPath>> {
        let mentioned_paths = extract_file_paths(response);
        let mut invalid = Vec::new();
        
        for path in mentioned_paths {
            if !self.file_reader.path_exists(&path).await? {
                invalid.push(path);
            }
        }
        
        invalid
    }
}

// Integration with stages
impl CuratorStage {
    pub async fn curate_with_verification(&self, input: &StageOutputs) -> Result<CuratorOutput> {
        let initial_output = self.curate_internal(input).await?;
        
        // Verify no hallucination
        let verification = self.verifier.verify_response(&initial_output.content).await?;
        
        if verification.has_issues() {
            // Replace fake content with real content
            let corrected = self.replace_with_real_content(
                &initial_output.content,
                &verification.issues
            ).await?;
            
            Ok(CuratorOutput {
                content: corrected,
                verification_status: VerificationStatus::Corrected,
                ..initial_output
            })
        } else {
            Ok(initial_output)
        }
    }
}
```

### Phase 2: Deep Learning System (Week 2)
Building on file reading to enable "Learn this entire codebase":

```rust
// src/consensus/learning.rs
pub struct CodebaseLearner {
    file_reader: FileReader,
    analyzer: CodeAnalyzer,
    memory: PersistentMemory,
}

impl CodebaseLearner {
    pub async fn learn_codebase(&mut self, root: &Path) -> Result<CodebaseSummary> {
        // 1. Discover all files
        let all_files = self.discover_all_files(root).await?;
        
        // 2. Analyze project structure
        let structure = self.analyze_structure(&all_files).await?;
        
        // 3. Read and analyze key files
        let key_files = self.identify_key_files(&all_files);
        let mut file_summaries = HashMap::new();
        
        for file in key_files {
            let content = self.file_reader.read_file(&file).await?;
            let summary = self.analyzer.analyze_file(&file, &content).await?;
            file_summaries.insert(file, summary);
        }
        
        // 4. Analyze patterns and architecture
        let patterns = self.detect_patterns(&file_summaries).await?;
        let architecture = self.analyze_architecture(&structure, &patterns).await?;
        
        // 5. Create comprehensive summary
        let summary = CodebaseSummary {
            root_path: root.to_path_buf(),
            total_files: all_files.len(),
            structure,
            architecture,
            patterns,
            key_components: self.identify_components(&file_summaries).await?,
            entry_points: self.find_entry_points(&file_summaries).await?,
            dependencies: self.analyze_dependencies(root).await?,
            learned_at: SystemTime::now(),
        };
        
        // 6. Save to persistent memory
        self.memory.save_codebase_summary(&summary).await?;
        
        Ok(summary)
    }
}
```

### Phase 3: File Writing with Safety (Week 3)
Once reading works perfectly, add writing:

```rust
// src/consensus/file_writer.rs
pub struct FileWriter {
    reader: FileReader,
    backup_manager: BackupManager,
    approval_system: ApprovalSystem,
}

impl FileWriter {
    // Safe file operations with approval
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<WriteResult> {
        // 1. Check security policy
        self.verify_write_allowed(path)?;
        
        // 2. Create backup
        let backup = self.backup_manager.backup_file(path).await?;
        
        // 3. Request approval
        let approval = self.approval_system.request_approval(
            FileOperation::Write { path, content }
        ).await?;
        
        match approval {
            Approval::Approved => {
                // 4. Write file
                fs::write(path, content).await?;
                
                Ok(WriteResult::Success { backup_id: backup.id })
            }
            Approval::Rejected => {
                Ok(WriteResult::Rejected)
            }
            Approval::Modified(new_content) => {
                // User modified the content
                fs::write(path, new_content).await?;
                Ok(WriteResult::ModifiedAndWritten { backup_id: backup.id })
            }
        }
    }
}
```

## 🎯 Success Criteria

### Immediate Goals (Phase 1)
- [ ] Consensus can read actual file contents
- [ ] No more generic responses - all answers based on real code
- [ ] No hallucinated code examples
- [ ] Validator feedback leads to file re-reading
- [ ] Visual indicator shows repository awareness

### Medium-term Goals (Phase 2-3)
- [ ] "Learn this entire codebase" works and persists
- [ ] Can answer specific questions about actual code
- [ ] Creates accurate implementation plans
- [ ] File writing with proper approval flow

### Long-term Goals
- [ ] Full Claude Code-style experience
- [ ] Repository specialization that improves over time
- [ ] Natural conversation flow
- [ ] Safe and reliable file operations

## 📊 Metrics for Success

### Accuracy Metrics
- Repository analysis accuracy: >95% (vs current ~10%)
- Code example accuracy: 100% (no hallucination)
- File path accuracy: 100% (all paths verified)

### Performance Metrics  
- File read time: <100ms per file
- Repository analysis: <30s for 1000 files
- Response time with file reading: <5s

### User Experience Metrics
- Approval required for writes: 100%
- Rollback available: 100%
- Context preserved across stages: 100%

## 🚀 Next Steps

1. **Commit current progress** ✓
2. **Implement Phase 1.4 (File Reading)** - CRITICAL
3. **Test with real repository analysis**
4. **Implement Phase 1.5 (Stage Coordination)**
5. **Add Phase 1.6 (Anti-Hallucination)**
6. **Implement Phase 1.3 (UI Widget)** 

This plan addresses all the critical issues discovered in testing and provides a clear path to achieving Claude Code-style functionality in Hive.

## 🚨 CRITICAL ISSUES DISCOVERED (2025-07-17)

### 1. @codebase Command Not Triggering
- The @codebase command is designed but NOT wired up in consensus engine
- No actual deep scanning or indexing is happening when user types @codebase
- Need to detect @codebase in ConsensusEngine::process_streaming() and trigger CodebaseIntelligence::analyze_codebase()

### 2. Refiner Stage Not Actually Refining
- Refiner is providing commentary about what COULD be improved
- It should actually REFINE the content and output an enhanced version
- Example: When Generator outputs basic analysis, Refiner should enhance it with deeper insights, not just say "could add more detail"
- Other stages need the refined output, not just suggestions

### 3. Semantic Search Not Connected
- Designed semantic search system but it's not being used
- After @codebase indexes the repo, ALL questions should search that index
- Currently using basic file reading instead of intelligent semantic search
- Need to:
  - Extract search terms from EVERY question
  - Search indexed codebase
  - Pass results to all stages in context

### 4. Stages Not Accessing Indexed Data  
- Created codebase_intelligence module but stages can't access it
- Need to pass CodebaseIntelligence instance through pipeline
- All 4 stages should have access to semantic indexed data
- Key integration point: ConsensusPipeline::build_full_context() needs to call CodebaseIntelligence::get_context_for_question()

### 5. Current Implementation Status
- ✅ File reading works (FileAwareGeneratorStage)
- ✅ Anti-hallucination works (Validator catches fake code)
- ❌ @codebase command detection missing
- ❌ Deep scanning not implemented
- ❌ Semantic search not integrated
- ❌ Refiner not refining content

### Implementation Plan:
1. Wire up @codebase in ConsensusEngine::process_streaming()
2. Implement actual AST parsing in ObjectExtractor
3. Fix Refiner to output refined content
4. Connect semantic search to all stages
5. Store indexed data in SQLite tables