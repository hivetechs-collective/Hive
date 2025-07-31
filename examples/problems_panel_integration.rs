//! Problems Panel Integration Example
//!
//! Demonstrates how to integrate the Problems Panel with build systems,
//! providing real-time updates, navigation, and status bar integration.

use anyhow::Result;
use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, Level};

// Import the problems panel components
use hive::desktop::problems::{
    BuildSystemIntegration, 
    BuildTool,
    ProblemNavigationHandler,
    ProblemsPanel,
    ProblemsState,
    ProblemsStatusBar,
    ProblemsUpdateManager,
    EditorIntegration,
    UpdateEvent,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting Problems Panel Integration Example");

    // Set up the workspace path
    let workspace_path = std::env::current_dir()?;
    info!("📁 Workspace: {}", workspace_path.display());

    // Initialize the integration components
    let example = ProblemsIntegrationExample::new(workspace_path).await?;
    
    // Run the example
    example.run().await?;

    Ok(())
}

/// Complete problems panel integration example
struct ProblemsIntegrationExample {
    workspace_path: PathBuf,
    build_integration: Arc<RwLock<BuildSystemIntegration>>,
    navigation_handler: Arc<ProblemNavigationHandler>,
    update_manager: Arc<RwLock<ProblemsUpdateManager>>,
    problems_state: Arc<RwLock<ProblemsState>>,
}

impl ProblemsIntegrationExample {
    /// Create new integration example
    async fn new(workspace_path: PathBuf) -> Result<Self> {
        info!("🔧 Initializing Problems Panel Integration...");

        // Initialize build system integration
        let mut build_integration = BuildSystemIntegration::new(workspace_path.clone());
        build_integration.initialize().await?;
        let build_integration = Arc::new(RwLock::new(build_integration));

        // Initialize navigation handler
        let mut navigation_handler = ProblemNavigationHandler::new(workspace_path.clone());
        navigation_handler.set_editor_integration(EditorIntegration::VSCode);
        let navigation_handler = Arc::new(navigation_handler);

        // Initialize update manager
        let (mut update_manager, mut update_receiver) = ProblemsUpdateManager::new(
            workspace_path.clone(),
            build_integration.clone(),
            navigation_handler.clone(),
        ).await?;
        update_manager.initialize().await?;
        let update_manager = Arc::new(RwLock::new(update_manager));

        // Initialize problems state
        let problems_state = Arc::new(RwLock::new(ProblemsState::default()));

        // Start update event processing loop
        let update_manager_clone = update_manager.clone();
        let problems_state_clone = problems_state.clone();
        tokio::spawn(async move {
            Self::process_update_events(update_receiver, update_manager_clone, problems_state_clone).await;
        });

        info!("✅ Problems Panel Integration initialized");

        Ok(Self {
            workspace_path,
            build_integration,
            navigation_handler,
            update_manager,
            problems_state,
        })
    }

    /// Process update events
    async fn process_update_events(
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<UpdateEvent>,
        update_manager: Arc<RwLock<ProblemsUpdateManager>>,
        problems_state: Arc<RwLock<ProblemsState>>,
    ) {
        while let Some(event) = receiver.recv().await {
            match &event {
                UpdateEvent::BuildCompleted { tool, problems, success, duration } => {
                    info!("🔧 Build completed: {:?} - {} problems in {:?}", 
                          tool, problems.len(), duration);
                    
                    // Update problems state
                    {
                        let mut state = problems_state.write().await;
                        // Clear previous problems from this tool source
                        // Add new problems...
                        // This would be more sophisticated in a real implementation
                    }
                }
                UpdateEvent::FileChanged { paths, change_type } => {
                    info!("📝 Files changed: {:?} (type: {:?})", paths.len(), change_type);
                }
                _ => {}
            }

            // Handle the event through the update manager
            if let Ok(mut manager) = update_manager.try_write() {
                if let Err(e) = manager.handle_update_event(event).await {
                    eprintln!("Error handling update event: {}", e);
                }
            }
        }
    }

    /// Run the integration example
    async fn run(&self) -> Result<()> {
        info!("🎯 Running Problems Panel Integration Example");

        // Demonstrate initial build check
        self.demo_initial_build_check().await?;

        // Demonstrate real-time monitoring
        self.demo_real_time_monitoring().await?;

        // Demonstrate navigation
        self.demo_navigation().await?;

        // Keep running for demonstration
        info!("🔄 Monitoring for changes... (Press Ctrl+C to exit)");
        tokio::signal::ctrl_c().await?;
        info!("👋 Shutting down");

        Ok(())
    }

    /// Demonstrate initial build check
    async fn demo_initial_build_check(&self) -> Result<()> {
        info!("🔍 Demonstrating initial build check...");

        let mut build_integration = self.build_integration.write().await;
        
        // Trigger builds for detected tools
        build_integration.refresh_all().await?;
        
        // Display results
        let stats = build_integration.get_build_stats();
        info!("📊 Build Statistics:");
        info!("  - Total monitors: {}", stats.total_monitors);
        info!("  - Active monitors: {}", stats.active_monitors);
        info!("  - Total problems: {}", stats.total_problems);
        info!("  - Errors: {}", stats.total_errors);
        info!("  - Warnings: {}", stats.total_warnings);

        Ok(())
    }

    /// Demonstrate real-time monitoring
    async fn demo_real_time_monitoring(&self) -> Result<()> {
        info!("⏱️ Demonstrating real-time monitoring...");

        // Force a refresh to trigger updates
        {
            let update_manager = self.update_manager.read().await;
            update_manager.force_refresh().await?;
        }

        // Show current settings
        {
            let update_manager = self.update_manager.read().await;
            let settings = update_manager.get_settings().await;
            info!("⚙️ Update Settings:");
            info!("  - Auto build on save: {}", settings.auto_build_on_save);
            info!("  - Auto refresh: {}", settings.auto_refresh_enabled);
            info!("  - File debounce: {:?}", settings.frequency.file_watch_debounce);
        }

        Ok(())
    }

    /// Demonstrate navigation functionality
    async fn demo_navigation(&self) -> Result<()> {
        info!("🧭 Demonstrating navigation functionality...");

        // Get current problems
        let problems_state = self.problems_state.read().await;
        let all_problems: Vec<_> = problems_state.problems.values()
            .flat_map(|problems| problems.iter())
            .collect();

        if let Some(first_problem) = all_problems.first() {
            info!("📍 Attempting to navigate to first problem: {}", first_problem.message);
            
            match self.navigation_handler.navigate_to_problem(first_problem).await {
                Ok(result) => {
                    info!("✅ Navigation result: {:?}", result);
                }
                Err(e) => {
                    info!("❌ Navigation failed: {}", e);
                }
            }
        } else {
            info!("ℹ️ No problems found to navigate to");
        }

        Ok(())
    }
}

/// Example Dioxus component showing how to use the problems panel in a UI
#[component]
fn ProblemsIntegrationUI() -> Element {
    // State for problems
    let mut problems_state = use_signal(|| ProblemsState::default());
    let mut show_problems_panel = use_signal(|| false);
    let mut show_status_details = use_signal(|| false);

    // Mock build stats (in real app, this would come from the integration)
    let build_stats = use_signal(|| hive::desktop::problems::BuildStats {
        total_monitors: 3,
        active_monitors: 2,
        total_problems: 5,
        total_errors: 2,
        total_warnings: 3,
        total_info: 0,
        total_hints: 0,
    });

    let active_builds = use_signal(|| vec![BuildTool::Cargo, BuildTool::Clippy]);

    rsx! {
        div {
            class: "problems-integration-demo",
            style: "height: 100vh; display: flex; flex-direction: column; font-family: 'Segoe UI', sans-serif;",

            // Header
            div {
                class: "demo-header",
                style: "padding: 16px; background: #2d2d30; color: white; border-bottom: 1px solid #454545;",
                h1 { "Problems Panel Integration Demo" }
                p { "Demonstrating build system integration, real-time updates, and navigation" }
            }

            // Main content area
            div {
                class: "demo-content",
                style: "flex: 1; display: flex;",

                // Left panel - code/file area simulation
                div {
                    class: "code-area",
                    style: "flex: 1; background: #1e1e1e; color: #d4d4d4; padding: 16px;",
                    
                    h3 { "Simulated Code Editor" }
                    p { "In a real application, this would be the code editor." }
                    p { "Problems panel integration would provide:" }
                    ul {
                        li { "Click-to-navigate from problems to code locations" }
                        li { "Real-time error highlighting" }
                        li { "Quick fixes and suggestions" }
                        li { "Keyboard shortcuts for navigation" }
                    }

                    button {
                        onclick: move |_| {
                            show_problems_panel.set(!show_problems_panel());
                        },
                        style: "margin-top: 16px; padding: 8px 16px; background: #0e639c; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        if *show_problems_panel.read() { "Hide Problems Panel" } else { "Show Problems Panel" }
                    }
                }

                // Problems panel (shown when toggled)
                if *show_problems_panel.read() {
                    div {
                        class: "problems-panel-container",
                        style: "width: 400px; border-left: 1px solid #454545;",
                        
                        ProblemsPanel {
                            state: problems_state,
                            on_problem_select: move |problem_id: String| {
                                println!("Selected problem: {}", problem_id);
                            },
                            on_problem_navigate: move |(path, line, column): (PathBuf, u32, u32)| {
                                println!("Navigate to: {}:{}:{}", path.display(), line, column);
                            },
                        }
                    }
                }
            }

            // Status bar
            div {
                class: "demo-status-bar",
                style: "height: 24px; background: #007acc; color: white; display: flex; align-items: center; justify-content: space-between; padding: 0 16px; font-size: 12px;",
                
                div {
                    style: "display: flex; align-items: center; gap: 16px;",
                    span { "Ready" }
                    span { "Line 1, Column 1" }
                }

                ProblemsStatusBar {
                    problems_state,
                    build_stats,
                    active_builds,
                    on_click: move |_| {
                        show_problems_panel.set(!show_problems_panel());
                    }
                }
            }
        }
    }
}

/// CSS styles for the demo
const DEMO_STYLES: &str = r#"
.problems-integration-demo {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

.demo-header h1 {
    margin: 0 0 8px 0;
    font-size: 24px;
    font-weight: 600;
}

.demo-header p {
    margin: 0;
    opacity: 0.8;
    font-size: 14px;
}

.code-area h3 {
    color: #4ec9b0;
    margin-top: 0;
}

.code-area ul {
    color: #ce9178;
}

.code-area li {
    margin: 4px 0;
}

.demo-status-bar {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_problems_integration_initialization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        // Create a minimal Rust project structure
        std::fs::create_dir_all(workspace_path.join("src")).unwrap();
        std::fs::write(
            workspace_path.join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#
        ).unwrap();

        std::fs::write(
            workspace_path.join("src/main.rs"),
            r#"
fn main() {
    println!("Hello, world!");
}
"#
        ).unwrap();

        let example = ProblemsIntegrationExample::new(workspace_path).await;
        assert!(example.is_ok());
    }

    #[tokio::test]
    async fn test_build_integration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        // Create Cargo.toml
        std::fs::write(
            workspace_path.join("Cargo.toml"),
            r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#
        ).unwrap();

        let mut build_integration = BuildSystemIntegration::new(workspace_path);
        build_integration.initialize().await.unwrap();

        let stats = build_integration.get_build_stats();
        assert!(stats.total_monitors > 0);
    }
}