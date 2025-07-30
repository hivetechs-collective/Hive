//! Integration tests for the repository dropdown component

#[cfg(test)]
mod tests {
    use hive::desktop::git::{
        RepositorySelectorState, RepositoryInfo, RepositoryStatus, UpstreamStatus,
    };
    use std::path::PathBuf;

    #[test]
    fn test_repository_selector_state() {
        let mut state = RepositorySelectorState::default();
        
        // Test adding repositories
        let repo1 = RepositoryInfo {
            path: PathBuf::from("/test/repo1"),
            name: "repo1".to_string(),
            current_branch: Some("main".to_string()),
            status: RepositoryStatus::Clean,
            upstream_status: None,
        };
        
        let repo2 = RepositoryInfo {
            path: PathBuf::from("/test/repo2"),
            name: "repo2".to_string(),
            current_branch: Some("develop".to_string()),
            status: RepositoryStatus::HasChanges,
            upstream_status: Some(UpstreamStatus {
                ahead: 3,
                behind: 1,
                has_upstream: true,
            }),
        };
        
        state.add_repository(repo1.clone());
        state.add_repository(repo2.clone());
        
        assert_eq!(state.repositories.len(), 2);
        assert_eq!(state.current_repo_index, Some(0));
        
        // Test current repository
        let current = state.current_repository();
        assert!(current.is_some());
        assert_eq!(current.unwrap().name, "repo1");
        
        // Test switching repository
        state.set_current_repository(&PathBuf::from("/test/repo2"));
        assert_eq!(state.current_repo_index, Some(1));
        
        let current = state.current_repository();
        assert!(current.is_some());
        assert_eq!(current.unwrap().name, "repo2");
        
        // Test removing repository
        state.remove_repository(&PathBuf::from("/test/repo1"));
        assert_eq!(state.repositories.len(), 1);
        assert_eq!(state.current_repo_index, Some(0)); // Should adjust index
    }
    
    #[test]
    fn test_repository_status_display() {
        // Test status icons
        assert_eq!(RepositoryStatus::Clean.icon(), "✓");
        assert_eq!(RepositoryStatus::HasChanges.icon(), "●");
        assert_eq!(RepositoryStatus::HasConflicts.icon(), "⚠");
        assert_eq!(RepositoryStatus::NotRepository.icon(), "⭘");
        
        // Test status colors
        assert_eq!(RepositoryStatus::Clean.color(), "#28a745");
        assert_eq!(RepositoryStatus::HasChanges.color(), "#ffc107");
        assert_eq!(RepositoryStatus::HasConflicts.color(), "#dc3545");
        assert_eq!(RepositoryStatus::NotRepository.color(), "#6c757d");
    }
    
    #[test]
    fn test_repository_info_creation() {
        let repo = RepositoryInfo {
            path: PathBuf::from("/Users/dev/projects/test"),
            name: "test".to_string(),
            current_branch: Some("feature/test".to_string()),
            status: RepositoryStatus::HasChanges,
            upstream_status: Some(UpstreamStatus {
                ahead: 5,
                behind: 2,
                has_upstream: true,
            }),
        };
        
        assert_eq!(repo.name, "test");
        assert_eq!(repo.current_branch, Some("feature/test".to_string()));
        assert_eq!(repo.status, RepositoryStatus::HasChanges);
        assert!(repo.upstream_status.is_some());
        
        let upstream = repo.upstream_status.unwrap();
        assert_eq!(upstream.ahead, 5);
        assert_eq!(upstream.behind, 2);
        assert!(upstream.has_upstream);
    }
    
    #[test]
    fn test_empty_repository_selector() {
        let state = RepositorySelectorState::default();
        
        assert!(state.repositories.is_empty());
        assert!(state.current_repo_index.is_none());
        assert!(state.current_repository().is_none());
    }
}

// Helper trait implementations for testing
impl RepositoryStatus {
    fn icon(&self) -> &'static str {
        match self {
            RepositoryStatus::Clean => "✓",
            RepositoryStatus::HasChanges => "●",
            RepositoryStatus::HasConflicts => "⚠",
            RepositoryStatus::NotRepository => "⭘",
            RepositoryStatus::Error(_) => "⚠",
        }
    }
    
    fn color(&self) -> &'static str {
        match self {
            RepositoryStatus::Clean => "#28a745",
            RepositoryStatus::HasChanges => "#ffc107",
            RepositoryStatus::HasConflicts => "#dc3545",
            RepositoryStatus::NotRepository => "#6c757d",
            RepositoryStatus::Error(_) => "#dc3545",
        }
    }
}