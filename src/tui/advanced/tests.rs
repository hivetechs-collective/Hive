//! Tests for Advanced TUI Repository Selector Integration

#[cfg(test)]
mod tests {
    use super::super::{AdvancedTuiApp, PanelType};
    use crate::desktop::events::EventType;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_advanced_tui_app_creation() {
        let app = AdvancedTuiApp::new().await;
        assert!(app.is_ok(), "Should be able to create AdvancedTuiApp");
        
        let app = app.unwrap();
        assert_eq!(app.active_panel, PanelType::Explorer);
        assert!(!app.repository_selector.is_open());
    }

    #[tokio::test]
    async fn test_repository_selector_integration() {
        let mut app = AdvancedTuiApp::new().await.unwrap();
        
        // Test repository selector toggle
        assert!(!app.repository_selector.is_open());
        app.repository_selector.toggle();
        assert!(app.repository_selector.is_open());
        
        // Test workspace state access
        let workspace_state = app.get_workspace_state().await;
        let workspace = workspace_state.lock().await;
        
        // Should have initialized with current directory
        assert!(!workspace.root_path.as_os_str().is_empty());
    }

    #[tokio::test]
    async fn test_event_bus_integration() {
        let app = AdvancedTuiApp::new().await.unwrap();
        let event_bus = app.get_event_bus();
        
        // Test that event bus is initialized
        let subscriber_count = event_bus.subscriber_count(&EventType::RepositoryChanged).await;
        assert_eq!(subscriber_count, 1, "Should have one repository change subscriber");
    }

    #[tokio::test]
    async fn test_workspace_initialization() {
        let mut app = AdvancedTuiApp::new().await.unwrap();
        
        // Test advanced discovery service initialization
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let result = app.initialize_discovery_service(workspace_root).await;
        
        assert!(result.is_ok(), "Should initialize discovery service successfully");
        assert!(app.discovery_service.is_some(), "Discovery service should be set");
    }

    #[tokio::test]
    async fn test_repository_selector_update_from_workspace() {
        let mut app = AdvancedTuiApp::new().await.unwrap();
        
        // Initially should have some repositories discovered
        let initial_count = {
            let workspace = app.workspace_state.lock().await;
            workspace.repositories.len()
        };
        
        // Update repository selector
        let result = app.update_repository_selector().await;
        assert!(result.is_ok(), "Should update repository selector successfully");
        
        // Repository selector should be updated with workspace repositories
        assert_eq!(app.repository_selector.visible_count(), initial_count);
    }

    #[tokio::test]
    async fn test_repository_switching_workflow() {
        let mut app = AdvancedTuiApp::new().await.unwrap();
        
        // Ensure we have repositories to work with
        {
            let workspace = app.workspace_state.lock().await;
            if workspace.repositories.is_empty() {
                return; // Skip test if no repositories found
            }
        }
        
        // Update repository selector
        app.update_repository_selector().await.unwrap();
        
        // Test current repository tracking
        let current_repo = app.repository_selector.current_repository();
        
        if let Some(current_path) = current_repo {
            // Verify that the workspace state matches
            let workspace = app.workspace_state.lock().await;
            assert_eq!(workspace.active_repository.as_ref(), Some(current_path));
        }
    }

    #[tokio::test]
    async fn test_panel_cycling_with_repository_selector() {
        let mut app = AdvancedTuiApp::new().await.unwrap();
        
        // Test cycling through panels including repository selector
        assert_eq!(app.active_panel, PanelType::Explorer);
        
        app.cycle_active_panel();
        assert_eq!(app.active_panel, PanelType::Editor);
        
        app.cycle_active_panel();
        assert_eq!(app.active_panel, PanelType::Terminal);
        
        app.cycle_active_panel();
        assert_eq!(app.active_panel, PanelType::ConsensusProgress);
        
        app.cycle_active_panel();
        assert_eq!(app.active_panel, PanelType::Problems);
        
        app.cycle_active_panel();
        assert_eq!(app.active_panel, PanelType::Explorer);
        
        // Test repository selector panel handling
        app.active_panel = PanelType::RepositorySelector;
        app.cycle_active_panel();
        assert_eq!(app.active_panel, PanelType::Explorer);
    }
}