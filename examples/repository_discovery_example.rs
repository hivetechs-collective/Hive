//! Example of using the Repository Discovery Service
//!
//! This example demonstrates how to use the repository discovery service
//! to scan for repositories with various configuration options.

use std::path::PathBuf;
use hive::core::config::HiveConfig;
use hive::desktop::workspace::{
    RepositoryDiscoveryService, DiscoveryConfig, ScanningMode, ProjectType
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🐝 Hive AI Repository Discovery Example");
    println!("======================================");
    
    // Example 1: Basic discovery with default configuration
    println!("\n📁 Example 1: Basic Repository Discovery");
    let discovery_service = RepositoryDiscoveryService::new(DiscoveryConfig::default());
    
    let repositories = discovery_service.discover_repositories().await?;
    println!("Found {} repositories:", repositories.len());
    
    for repo in &repositories {
        println!("  📦 {} ({:?})", repo.basic_info.name, repo.basic_info.path);
        if let Some(project_info) = &repo.project_info {
            println!("      Type: {:?}", project_info.project_type);
            if let Some(name) = &project_info.name {
                println!("      Name: {}", name);
            }
        }
        println!("      Branch: {:?}", repo.git_status.current_branch);
        println!("      Confidence: {:.1}%", repo.discovery.confidence * 100.0);
    }
    
    // Example 2: Custom configuration with shallow scanning
    println!("\n🔍 Example 2: Shallow Scanning Mode");
    let shallow_config = DiscoveryConfig {
        scanning_mode: ScanningMode::Shallow,
        scan_paths: vec![
            std::env::current_dir()?,
        ],
        max_repositories: Some(5),
        ..Default::default()
    };
    
    let shallow_service = RepositoryDiscoveryService::new(shallow_config);
    let shallow_repos = shallow_service.discover_repositories().await?;
    
    println!("Shallow scan found {} repositories in current directory", shallow_repos.len());
    
    // Example 3: Filter repositories by project type
    println!("\n🦀 Example 3: Filter by Project Type");
    let rust_repos: Vec<_> = repositories.iter()
        .filter(|repo| {
            repo.project_info
                .as_ref()
                .map(|info| info.project_type == ProjectType::Rust)
                .unwrap_or(false)
        })
        .collect();
    
    println!("Found {} Rust repositories:", rust_repos.len());
    for repo in rust_repos {
        println!("  🦀 {}", repo.basic_info.name);
        if let Some(project_info) = &repo.project_info {
            if let Some(version) = &project_info.version {
                println!("      Version: {}", version);
            }
        }
    }
    
    // Example 4: Using with Hive configuration
    println!("\n⚙️  Example 4: Using Hive Configuration");
    let hive_config = HiveConfig::default();
    let config_service = RepositoryDiscoveryService::from_hive_config(&hive_config);
    
    let stats = config_service.get_stats().await;
    println!("Discovery service statistics:");
    println!("  Total scans: {}", stats.total_scans);
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    
    // Example 5: Repository metadata inspection
    println!("\n📊 Example 5: Repository Metadata");
    if let Some(repo) = repositories.first() {
        println!("Detailed info for repository: {}", repo.basic_info.name);
        println!("  Path: {:?}", repo.basic_info.path);
        println!("  Discovery method: {:?}", repo.discovery.discovery_method);
        println!("  Last refreshed: {}", repo.discovery.last_refreshed);
        println!("  Is accessible: {}", repo.discovery.is_accessible);
        
        if let Some(upstream) = &repo.git_status.upstream {
            println!("  Upstream: {}/{}", upstream.remote_name, upstream.branch_name);
            println!("  Ahead: {}, Behind: {}", upstream.ahead, upstream.behind);
        }
        
        println!("  Working directory status:");
        let wd = &repo.git_status.working_dir_status;
        println!("    Modified: {}, Staged: {}, Untracked: {}", 
                 wd.modified_files, wd.staged_files, wd.untracked_files);
        
        if !repo.git_status.remotes.is_empty() {
            println!("  Remotes:");
            for remote in &repo.git_status.remotes {
                println!("    {}: {}", remote.name, remote.url);
            }
        }
    }
    
    println!("\n✅ Repository discovery example completed!");
    
    Ok(())
}