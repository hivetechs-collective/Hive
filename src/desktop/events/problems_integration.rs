//! Problems Panel Integration with Event Bus
//!
//! Example of how to integrate the event bus with the Problems panel

use super::{event_bus, Event, EventType, Problem, ProblemSeverity};
use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;

/// Publish problems update when diagnostics change
pub async fn publish_problems_update(
    added: Vec<Problem>,
    removed: Vec<Problem>,
    total_count: usize,
) -> Result<()> {
    let bus = event_bus();
    let event = Event::problems_updated(added, removed, total_count);
    bus.publish_async(event).await?;
    Ok(())
}

/// Example: Build system integration
pub async fn setup_build_problems_publisher() -> Result<()> {
    let bus = event_bus();

    // Subscribe to build completed events
    bus.subscribe_async(EventType::BuildCompleted, |event| async move {
        if let super::EventPayload::BuildResult {
            success,
            errors,
            warnings,
            ..
        } = event.payload
        {
            let mut problems = Vec::new();

            // Convert build errors to problems
            for (idx, error) in errors.into_iter().enumerate() {
                // Parse error message for file location
                // In real implementation, this would parse actual compiler output
                let problem = Problem {
                    id: format!("build-error-{}", idx),
                    severity: ProblemSeverity::Error,
                    source: "rustc".to_string(),
                    file_path: PathBuf::from("src/main.rs"), // Would be parsed from error
                    line: 1,                                 // Would be parsed
                    column: 1,                               // Would be parsed
                    message: error,
                    code: None,
                };
                problems.push(problem);
            }

            // Convert build warnings to problems
            for (idx, warning) in warnings.into_iter().enumerate() {
                let problem = Problem {
                    id: format!("build-warning-{}", idx),
                    severity: ProblemSeverity::Warning,
                    source: "rustc".to_string(),
                    file_path: PathBuf::from("src/lib.rs"), // Would be parsed
                    line: 1,
                    column: 1,
                    message: warning,
                    code: None,
                };
                problems.push(problem);
            }

            if !problems.is_empty() {
                publish_problems_update(problems, vec![], problems.len()).await?;
            }
        }
        Ok(())
    })
    .await;

    Ok(())
}

/// Example: Language server integration
pub mod lsp_integration {
    use super::*;

    /// Handle diagnostics from language server
    pub async fn handle_lsp_diagnostics(
        file_path: PathBuf,
        diagnostics: Vec<LspDiagnostic>,
    ) -> Result<()> {
        let mut added = Vec::new();
        let mut removed = Vec::new();

        // Convert LSP diagnostics to problems
        for diag in diagnostics {
            let severity = match diag.severity {
                1 => ProblemSeverity::Error,
                2 => ProblemSeverity::Warning,
                3 => ProblemSeverity::Information,
                4 => ProblemSeverity::Hint,
                _ => ProblemSeverity::Information,
            };

            let problem = Problem {
                id: format!("lsp-{}-{}-{}", file_path.display(), diag.line, diag.column),
                severity,
                source: "rust-analyzer".to_string(),
                file_path: file_path.clone(),
                line: diag.line,
                column: diag.column,
                message: diag.message,
                code: diag.code,
            };

            added.push(problem);
        }

        // In real implementation, would track previous diagnostics to compute removed

        let total_count = added.len(); // Would be actual total across all files
        publish_problems_update(added, removed, total_count).await?;

        Ok(())
    }

    /// Mock LSP diagnostic type
    pub struct LspDiagnostic {
        pub severity: u8,
        pub line: usize,
        pub column: usize,
        pub message: String,
        pub code: Option<String>,
    }
}

/// Example: File watcher integration for linting
pub async fn setup_lint_on_save() -> Result<()> {
    let bus = event_bus();

    bus.subscribe_async(EventType::FileChanged, |event| async move {
        if let super::EventPayload::FileChange { path, change_type } = &event.payload {
            if matches!(change_type, super::FileChangeType::Modified) {
                if path.extension().map_or(false, |ext| ext == "rs") {
                    debug!("Running linter on {:?}", path);

                    // In real implementation:
                    // 1. Run clippy or other linter
                    // 2. Parse output
                    // 3. Convert to problems
                    // 4. Publish problems update

                    // Example:
                    // let lint_output = run_clippy(path).await?;
                    // let problems = parse_clippy_output(lint_output);
                    // publish_problems_update(problems, vec![], total).await?;
                }
            }
        }
        Ok(())
    })
    .await;

    Ok(())
}
