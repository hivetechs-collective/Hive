//! Git file system watcher
//! 
//! Monitors git repository for changes and updates UI accordingly

use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::sync::mpsc;
use anyhow::Result;
use tracing::{info, debug, error};

/// Git repository watcher
pub struct GitWatcher {
    _watcher: notify::RecommendedWatcher,
}

/// Events emitted by the git watcher
#[derive(Debug, Clone)]
pub enum GitEvent {
    BranchChanged,
    StatusChanged,
    RemoteChanged,
    ConfigChanged,
    /// File-level status change with affected paths
    FileStatusChanged(Vec<PathBuf>),
}

impl GitWatcher {
    /// Create a new git watcher for a repository
    pub fn new(repo_path: &Path) -> Result<(Self, mpsc::UnboundedReceiver<GitEvent>)> {
        let (tx, rx) = channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        let mut watcher = notify::recommended_watcher(tx)?;
        
        // Watch .git directory for changes
        let git_dir = repo_path.join(".git");
        if git_dir.exists() {
            watcher.watch(&git_dir, RecursiveMode::Recursive)?;
            info!("Watching git directory: {:?}", git_dir);
        }
        
        // Also watch the working directory for file changes
        watcher.watch(repo_path, RecursiveMode::NonRecursive)?;
        
        // Start processing events in background thread
        std::thread::spawn(move || {
            Self::process_events(rx, event_tx, git_dir);
        });
        
        let git_watcher = Self { _watcher: watcher };
        Ok((git_watcher, event_rx))
    }
    
    /// Process file system events and convert to git events
    fn process_events(rx: std::sync::mpsc::Receiver<notify::Result<Event>>, event_tx: mpsc::UnboundedSender<GitEvent>, git_dir: PathBuf) {
        let mut last_event_time = std::time::Instant::now();
        
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    // Skip processing if consensus is running
                    if crate::consensus::pipeline::CONSENSUS_ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
                        debug!("⏸️ Git watcher paused during consensus");
                        continue;
                    }
                    
                    // Debounce events (100ms)
                    if last_event_time.elapsed() < Duration::from_millis(100) {
                        continue;
                    }
                    last_event_time = std::time::Instant::now();
                    
                    if let Some(git_event) = Self::classify_event(&event, &git_dir) {
                        debug!("Git event detected: {:?}", git_event);
                        let _ = event_tx.send(git_event);
                    }
                }
                Ok(Err(e)) => {
                    error!("Watch error: {:?}", e);
                }
                Err(_) => {
                    // Timeout, continue
                }
            }
        }
    }
    
    /// Classify file system event as a git event
    fn classify_event(event: &Event, git_dir: &Path) -> Option<GitEvent> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                for path in &event.paths {
                    // Check if this is a .git directory change
                    if path.starts_with(git_dir) {
                        let relative = path.strip_prefix(git_dir).ok()?;
                        let first_component = relative.components().next()?;
                        
                        match first_component.as_os_str().to_str()? {
                            "HEAD" | "refs" => return Some(GitEvent::BranchChanged),
                            "index" => {
                                // Index changes mean file status changed
                                return Some(GitEvent::FileStatusChanged(vec![]))
                            },
                            "objects" => return Some(GitEvent::StatusChanged),
                            "config" => return Some(GitEvent::ConfigChanged),
                            "FETCH_HEAD" | "ORIG_HEAD" => return Some(GitEvent::RemoteChanged),
                            _ => {}
                        }
                    } else {
                        // Working directory change - track specific files
                        let mut changed_files = Vec::new();
                        for path in &event.paths {
                            if path.is_file() {
                                changed_files.push(path.clone());
                            }
                        }
                        if !changed_files.is_empty() {
                            return Some(GitEvent::FileStatusChanged(changed_files));
                        }
                        return Some(GitEvent::StatusChanged);
                    }
                }
            }
            _ => {}
        }
        
        None
    }
}