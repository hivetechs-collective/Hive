//! Test the problems panel integration
//!
//! This example demonstrates the problems panel functionality
//! including git integration, build errors, and navigation.

use crate::tui::advanced::problems::{
    Problem, ProblemCategory, ProblemFilter, ProblemSeverity, ProblemsPanel,
};
use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing Problems Panel Integration");

    // Test 1: Create problems panel
    println!("\n1. Creating problems panel...");
    let current_dir = std::env::current_dir()?;
    let mut problems_panel = ProblemsPanel::new(Some(&current_dir));
    println!("✅ Problems panel created successfully");

    // Test 2: Update problems from workspace
    println!("\n2. Scanning workspace for problems...");
    problems_panel.update_problems(&current_dir).await?;
    let summary = problems_panel.get_summary();
    println!("✅ Found {} total problems:", summary.total);
    println!("   - {} errors", summary.errors);
    println!("   - {} warnings", summary.warnings);
    println!("   - {} infos", summary.infos);

    // Test 3: Test filtering
    println!("\n3. Testing problem filtering...");

    // Filter by errors only
    problems_panel.set_filter(ProblemFilter::ErrorsOnly);
    let errors_only = problems_panel.filtered_problems();
    println!("✅ Errors only: {} problems", errors_only.len());

    // Filter by warnings only
    problems_panel.set_filter(ProblemFilter::WarningsOnly);
    let warnings_only = problems_panel.filtered_problems();
    println!("✅ Warnings only: {} problems", warnings_only.len());

    // Filter by git conflicts
    problems_panel.set_filter(ProblemFilter::Category(ProblemCategory::GitConflict));
    let conflicts = problems_panel.filtered_problems();
    println!("✅ Git conflicts: {} problems", conflicts.len());

    // Reset to all
    problems_panel.set_filter(ProblemFilter::All);

    // Test 4: Test navigation
    println!("\n4. Testing problem navigation...");
    if let Some(location) = problems_panel.navigate_to_selected() {
        println!("✅ Navigation target: {}", location.file_path.display());
        if let Some(line) = location.line {
            println!("   Line: {}", line);
        }
        if let Some(column) = location.column {
            println!("   Column: {}", column);
        }
    } else {
        println!("ℹ️  No problems selected for navigation");
    }

    // Test 5: Show detailed problem info
    println!("\n5. Problem details:");
    let all_problems = problems_panel.filtered_problems();
    for (i, problem) in all_problems.iter().take(5).enumerate() {
        println!(
            "   {}. {} {}: {}",
            i + 1,
            problem.severity.icon(),
            problem.category.display_name(),
            problem.message
        );
        if let Some(path) = &problem.file_path {
            print!("      📁 {}", path.display());
            if let Some(line) = problem.line {
                print!(":{}", line);
                if let Some(col) = problem.column {
                    print!(":{}", col);
                }
            }
            println!();
        }
        if let Some(context) = &problem.context {
            println!("      💡 {}", context);
        }
        println!();
    }

    println!("🎉 Problems panel integration test completed successfully!");
    Ok(())
}
