//! Code improvement command implementation

use anyhow::{anyhow, Result};
use colored::*;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    analysis::FileAnalyzer,
    core::context::ContextBuilder,
    transformation::{
        quick_validate, transform_code, TransformationEngine, TransformationHistory,
        TransformationRequest,
    },
};

/// Handle the improve command
pub async fn handle_improve(
    file_path: PathBuf,
    aspect: String,
    preview: bool,
    apply: bool,
    multi_file: bool,
    context_str: Option<String>,
) -> Result<()> {
    if !preview && !apply {
        return Err(anyhow!(
            "Either --preview or --apply must be specified. Use --preview to see changes first."
        ));
    }

    // Check file exists
    if !file_path.exists() {
        return Err(anyhow!("File does not exist: {}", file_path.display()));
    }

    println!("{}", "🔍 Analyzing code for improvements...".cyan());

    // Create transformation request
    let request = TransformationRequest {
        file_path: file_path.clone(),
        aspect: aspect.clone(),
        context: context_str,
        multi_file,
    };

    // Create context builder
    let context = Arc::new(ContextBuilder::new());

    // Generate transformation using full engine
    let transformation_preview = transform_code(context.clone(), request).await?;

    if preview {
        // Display preview
        println!("\n{}", "=== Transformation Preview ===".bold());
        println!(
            "{}: {}",
            "Description".bold(),
            transformation_preview.transformation.description
        );
        println!(
            "{}: {}",
            "Aspect".bold(),
            transformation_preview.transformation.request.aspect
        );
        println!(
            "{}: {} files modified\n",
            "Impact".bold(),
            transformation_preview.impact.files_modified
        );

        // Risk assessment
        let risk_color = match transformation_preview.impact.risk_level {
            crate::transformation::RiskLevel::Low => "green",
            crate::transformation::RiskLevel::Medium => "yellow",
            crate::transformation::RiskLevel::High => "red",
        };
        println!(
            "{}: {}\n",
            "Risk Level".bold(),
            format!("{:?}", transformation_preview.impact.risk_level).color(risk_color)
        );

        // Show file diffs
        for diff in &transformation_preview.diffs {
            println!("{} {}", "File:".bold(), diff.file_path.display());
            println!(
                "  {} additions, {} deletions\n",
                format!("+{}", diff.additions).green(),
                format!("-{}", diff.deletions).red()
            );
        }

        // Show warnings
        if !transformation_preview.warnings.is_empty() {
            println!("{}", "Warnings:".yellow().bold());
            for warning in &transformation_preview.warnings {
                println!("  ⚠️  {}", warning);
            }
            println!();
        }

        if !apply {
            println!(
                "{}",
                "To apply these changes, run with --apply flag".yellow()
            );
        }
    }

    if apply {
        println!("\n{}", "📝 Applying transformations...".green());

        // Validate transformation first
        if !quick_validate(&transformation_preview.transformation).await? {
            return Err(anyhow!("Transformation validation failed"));
        }

        // Apply the transformation
        let engine = TransformationEngine::new(context).await?;
        engine
            .apply_transformation(&transformation_preview.transformation.id)
            .await?;

        println!(
            "✅ {} Applied successfully!",
            "Transformation".green().bold()
        );
        println!(
            "   Transaction ID: {}",
            transformation_preview.transformation.id
        );
        println!(
            "   Files modified: {}",
            transformation_preview.impact.files_modified
        );

        if transformation_preview.impact.tests_affected {
            println!(
                "\n{} Tests may be affected. Consider running your test suite.",
                "⚠️ ".yellow()
            );
        }

        println!(
            "\n{} Use 'hive undo' to revert these changes if needed.",
            "💡".cyan()
        );
    }

    Ok(())
}

/// Handle the undo command
pub async fn handle_undo() -> Result<()> {
    println!("{}", "↩️  Undoing last transformation...".cyan());

    // Create context and engine
    let context = Arc::new(ContextBuilder::new());
    let engine = TransformationEngine::new(context).await?;

    // Undo the last transformation
    engine.undo().await?;

    println!(
        "✅ {} Successfully undone!",
        "Transformation".green().bold()
    );
    println!("{} Use 'hive redo' to reapply if needed.", "💡".cyan());

    Ok(())
}

/// Handle the redo command
pub async fn handle_redo() -> Result<()> {
    println!("{}", "↪️  Redoing last undone transformation...".cyan());

    // Create context and engine
    let context = Arc::new(ContextBuilder::new());
    let engine = TransformationEngine::new(context).await?;

    // Redo the last undone transformation
    engine.redo().await?;

    println!(
        "✅ {} Successfully redone!",
        "Transformation".green().bold()
    );

    Ok(())
}

/// Handle transformation history command
pub async fn handle_transform_history(limit: usize) -> Result<()> {
    println!("{}", "📜 Transformation History".bold());
    println!("{}", "─".repeat(60));

    // Get transformation history
    let config_dir = dirs::config_dir()
        .map(|d| d.join("hive"))
        .unwrap_or_else(|| std::path::PathBuf::from(".hive"));

    let history = TransformationHistory::new(&config_dir)?;
    let transformations = history.get_history(limit).await;

    if transformations.is_empty() {
        println!("\nNo transformation history found.");
    } else {
        println!(
            "\nShowing last {} transformations:\n",
            transformations.len()
        );

        for (idx, transform) in transformations.iter().enumerate() {
            let status = if transform.applied {
                "✓ Applied".green()
            } else {
                "○ Pending".yellow()
            };
            println!("{:>3}. {} [{}]", idx + 1, transform.id, status);
            println!(
                "     Time: {}",
                transform.timestamp.format("%Y-%m-%d %H:%M:%S")
            );
            println!("     Aspect: {}", transform.request.aspect);
            println!("     File: {}", transform.request.file_path.display());
            println!("     Changes: {} file(s)", transform.changes.len());

            if let Some(tx_id) = &transform.transaction_id {
                println!("     Transaction: {}", tx_id);
            }
            println!();
        }
    }

    Ok(())
}

/// Handle the apply command
pub async fn handle_apply(changes: String, preview: bool, approve: bool) -> Result<()> {
    use serde_json::Value;

    println!("{}", "📝 Processing changes...".cyan());

    // Parse changes (could be a file path or JSON)
    let changes_data = if PathBuf::from(&changes).exists() {
        // Read from file
        tokio::fs::read_to_string(&changes).await?
    } else {
        // Assume it's JSON
        changes
    };

    // Parse as JSON
    let parsed: Value = serde_json::from_str(&changes_data)
        .map_err(|e| anyhow!("Invalid changes format: {}", e))?;

    // Extract transformation ID or changes
    let transformation_id = parsed
        .get("transformation_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing transformation_id in changes"))?;

    if preview {
        // Show preview using existing preview system
        let context = Arc::new(ContextBuilder::new());
        let engine = TransformationEngine::new(context).await?;

        // Get transformation from history
        let config_dir = dirs::config_dir()
            .map(|d| d.join("hive"))
            .unwrap_or_else(|| std::path::PathBuf::from(".hive"));
        let history = TransformationHistory::new(&config_dir)?;

        let transformations = history.get_history(1).await;
        if let Some(transformation) = transformations
            .into_iter()
            .find(|t| t.id == transformation_id)
        {
            let preview = crate::transformation::generate_preview(&transformation).await?;

            // Display preview
            println!("\n{}", "=== Changes Preview ===".bold());
            for diff in &preview.diffs {
                println!("{} {}", "File:".bold(), diff.file_path.display());
                println!(
                    "  {} additions, {} deletions",
                    format!("+{}", diff.additions).green(),
                    format!("-{}", diff.deletions).red()
                );
            }
        }
    }

    if !preview || approve {
        // Apply the changes
        let context = Arc::new(ContextBuilder::new());
        let engine = TransformationEngine::new(context).await?;

        engine.apply_transformation(transformation_id).await?;

        println!("✅ {} Applied successfully!", "Changes".green().bold());
        println!("{} Use 'hive undo' to revert if needed.", "💡".cyan());
    } else if preview && !approve {
        println!(
            "\n{}",
            "To apply these changes, run with --approve flag".yellow()
        );
    }

    Ok(())
}

/// Handle the preview command
pub async fn handle_preview(file: PathBuf, changes: Option<String>) -> Result<()> {
    println!("{}", "🔍 Generating preview...".cyan());

    if !file.exists() {
        return Err(anyhow!("File does not exist: {}", file.display()));
    }

    // Create context and engine
    let context = Arc::new(ContextBuilder::new());

    if let Some(changes_str) = changes {
        // Preview specific changes
        use crate::transformation::{simple_generate_preview, types::*};
        use chrono::Utc;

        // Create a transformation object
        let transformation = Transformation {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request: TransformationRequest {
                file_path: file.clone(),
                aspect: "AI-generated changes".to_string(),
                context: Some(changes_str.clone()),
                multi_file: false,
            },
            changes: vec![], // Would be populated with actual changes
            description: "AI-suggested improvements".to_string(),
            applied: false,
            transaction_id: None,
            confidence: 0.8,
            impact_score: 0.5,
            tags: vec!["ai-generated".to_string()],
        };

        let preview = simple_generate_preview(&transformation).await?;

        // Display preview
        println!("\n{}", "=== Transformation Preview ===".bold());
        for diff in &preview.diffs {
            println!("{} {}", "File:".bold(), diff.file_path.display());
            println!("{}", diff.unified_diff);
        }
    } else {
        // Generate AI suggestions for the file
        let request = TransformationRequest {
            file_path: file.clone(),
            aspect: "general".to_string(),
            context: None,
            multi_file: false,
        };

        let preview = transform_code(context, request).await?;

        // Display preview
        println!("\n{}", "=== AI Suggestions ===".bold());
        println!(
            "{}: {}",
            "Description".bold(),
            preview.transformation.description
        );
        println!(
            "{}: {} files would be modified\n",
            "Impact".bold(),
            preview.impact.files_modified
        );

        for diff in &preview.diffs {
            println!("{} {}", "File:".bold(), diff.file_path.display());
            println!(
                "  {} additions, {} deletions",
                format!("+{}", diff.additions).green(),
                format!("-{}", diff.deletions).red()
            );
        }

        println!(
            "\n{}",
            "To apply these suggestions, use 'hive apply'".yellow()
        );
    }

    Ok(())
}

/// Handle the transform command
pub async fn handle_transform(query: String, target: Option<PathBuf>, safe: bool) -> Result<()> {
    println!("{}", "🤖 AI-guided transformation...".cyan());

    // Determine target files
    let target_path = target.unwrap_or_else(|| PathBuf::from("."));

    if !target_path.exists() {
        return Err(anyhow!("Target does not exist: {}", target_path.display()));
    }

    // Create context
    let context = Arc::new(ContextBuilder::new());

    // If directory, find relevant files
    let files = if target_path.is_dir() {
        // Use repository intelligence to find relevant files
        let analyzer = FileAnalyzer::new();
        analyzer.find_files(&target_path, Some(&query)).await?
    } else {
        vec![target_path]
    };

    if files.is_empty() {
        println!("No relevant files found for transformation.");
        return Ok(());
    }

    println!("Found {} relevant file(s) for transformation", files.len());

    // Process each file
    for file in files {
        println!("\n{} {}", "Processing:".bold(), file.display());

        let request = TransformationRequest {
            file_path: file.clone(),
            aspect: "custom".to_string(),
            context: Some(query.clone()),
            multi_file: false,
        };

        let preview = transform_code(context.clone(), request).await?;

        if safe {
            // Extra validation in safe mode
            if !quick_validate(&preview.transformation).await? {
                println!("{} Skipping due to validation failure", "⚠️ ".yellow());
                continue;
            }
        }

        // Show preview
        println!(
            "  {} changes suggested",
            preview.transformation.changes.len()
        );

        // Apply if approved
        print!("  Apply changes? [y/N] ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            let engine = TransformationEngine::new(context.clone()).await?;
            engine
                .apply_transformation(&preview.transformation.id)
                .await?;
            println!("  ✅ Applied successfully!");
        } else {
            println!("  ⏭️  Skipped");
        }
    }

    println!(
        "\n✅ {} Transformation complete!",
        "AI-guided".green().bold()
    );

    Ok(())
}

/// List available improvement aspects
pub fn list_aspects() {
    println!("{}", "Available improvement aspects:".bold());
    println!();

    let aspects = vec![
        ("error-handling", "Improve error handling and recovery"),
        ("performance", "Optimize for better performance"),
        ("readability", "Enhance code readability and clarity"),
        ("security", "Identify and fix security issues"),
        ("memory", "Optimize memory usage"),
        ("concurrency", "Improve concurrent code safety"),
        ("documentation", "Add or improve documentation"),
        ("testing", "Suggest test improvements"),
        ("refactoring", "General code refactoring"),
        ("best-practices", "Apply language best practices"),
    ];

    for (aspect, description) in aspects {
        println!("  {} - {}", aspect.green().bold(), description);
    }

    println!();
    println!(
        "{} You can also use custom aspects based on your needs.",
        "💡".cyan()
    );
}
