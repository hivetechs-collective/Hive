//! LazyGit Controller - Translates UI actions to LazyGit commands
//! 
//! This module provides a bridge between our GUI buttons and LazyGit,
//! allowing users to perform git operations through familiar UI elements
//! while LazyGit handles the actual git work behind the scenes.

use anyhow::{Result, Context};
use std::time::Duration;
use tracing::{info, debug, warn};
use crate::desktop::terminal_registry::{send_to_terminal, TERMINAL_REGISTRY};

/// Controller for sending commands to LazyGit terminal
pub struct LazyGitController {
    terminal_id: String,
}

impl LazyGitController {
    /// Create a new LazyGit controller for the given terminal
    pub fn new(terminal_id: String) -> Self {
        Self { terminal_id }
    }
    
    /// Stage all changes
    pub async fn stage_all(&self) -> Result<()> {
        info!("🎯 GUI Action: Stage All -> Sending 'a' to LazyGit");
        self.send_keys("a").await?;
        Ok(())
    }
    
    /// Unstage all changes (navigate to staged panel first)
    pub async fn unstage_all(&self) -> Result<()> {
        info!("🎯 GUI Action: Unstage All -> Sending Tab+'a' to LazyGit");
        // Tab to switch to staged files panel
        self.send_keys("\t").await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        // Then 'a' to unstage all
        self.send_keys("a").await?;
        Ok(())
    }
    
    /// Commit with message from our commit box UI
    pub async fn commit(&self, message: &str) -> Result<()> {
        info!("🎯 GUI Action: Commit -> Opening LazyGit commit dialog");
        
        // Press 'c' to open commit dialog
        self.send_keys("c").await?;
        
        // Wait for commit dialog to open
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Clear any existing text first (Ctrl+A, Delete)
        self.send_keys("\x01").await?; // Ctrl+A (select all)
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.send_keys("\x7F").await?; // Delete
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Type the commit message
        debug!("Sending commit message: {}", message);
        self.send_keys(message).await?;
        
        // Wait a bit for message to be typed
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Press Enter to confirm
        self.send_keys("\r").await?;
        
        info!("✅ Commit command sent to LazyGit");
        Ok(())
    }
    
    /// Push to remote
    pub async fn push(&self) -> Result<()> {
        info!("🎯 GUI Action: Push -> Sending 'P' to LazyGit");
        self.send_keys("P").await?;
        Ok(())
    }
    
    /// Pull from remote
    pub async fn pull(&self) -> Result<()> {
        info!("🎯 GUI Action: Pull -> Sending 'p' to LazyGit");
        self.send_keys("p").await?;
        Ok(())
    }
    
    /// Sync (pull then push)
    pub async fn sync(&self) -> Result<()> {
        info!("🎯 GUI Action: Sync -> Performing pull then push");
        
        // First pull
        self.pull().await?;
        
        // Wait for pull to complete (LazyGit will show progress)
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Then push
        self.push().await?;
        
        Ok(())
    }
    
    /// Fetch from remote
    pub async fn fetch(&self) -> Result<()> {
        info!("🎯 GUI Action: Fetch -> Sending 'f' to LazyGit");
        self.send_keys("f").await?;
        Ok(())
    }
    
    /// Create a stash
    pub async fn stash_save(&self, message: &str) -> Result<()> {
        info!("🎯 GUI Action: Stash -> Opening stash menu");
        
        // Press 's' to open stash menu
        self.send_keys("s").await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Press 's' again to save stash
        self.send_keys("s").await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Type stash message if provided
        if !message.is_empty() {
            self.send_keys(message).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.send_keys("\r").await?;
        } else {
            // Just press enter for default message
            self.send_keys("\r").await?;
        }
        
        Ok(())
    }
    
    /// Discard all changes (with confirmation)
    pub async fn discard_all(&self) -> Result<()> {
        info!("🎯 GUI Action: Discard All -> Sending reset command");
        
        // Navigate to files panel
        self.send_keys("2").await?; // Focus files panel
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Select all files and discard
        self.send_keys("a").await?; // Select all
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.send_keys("d").await?; // Discard
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.send_keys("y").await?; // Confirm
        
        Ok(())
    }
    
    /// Refresh LazyGit view
    pub async fn refresh(&self) -> Result<()> {
        info!("🎯 GUI Action: Refresh -> Sending 'R' to LazyGit");
        self.send_keys("R").await?;
        Ok(())
    }
    
    /// Switch to specific panel in LazyGit
    pub async fn focus_panel(&self, panel: LazyGitPanel) -> Result<()> {
        let key = match panel {
            LazyGitPanel::Status => "1",
            LazyGitPanel::Files => "2",
            LazyGitPanel::Branches => "3",
            LazyGitPanel::Commits => "4",
            LazyGitPanel::Stash => "5",
        };
        
        debug!("Focusing LazyGit panel: {:?}", panel);
        self.send_keys(key).await?;
        Ok(())
    }
    
    /// Send raw keystrokes to LazyGit terminal
    async fn send_keys(&self, keys: &str) -> Result<()> {
        // Use the terminal registry to send keys
        let success = send_to_terminal(&self.terminal_id, keys);
        
        if success {
            debug!("Sent keys to LazyGit: {:?}", keys);
            Ok(())
        } else {
            warn!("Failed to send keys to LazyGit terminal: {}", self.terminal_id);
            Err(anyhow::anyhow!("Failed to send keys to LazyGit"))
        }
    }
}

/// LazyGit panel types
#[derive(Debug, Clone, Copy)]
pub enum LazyGitPanel {
    Status,
    Files,
    Branches,
    Commits,
    Stash,
}

/// Helper to check if LazyGit terminal is available
pub fn is_lazygit_available(terminal_id: &str) -> bool {
    if let Ok(registry) = TERMINAL_REGISTRY.lock() {
        registry.contains_key(terminal_id)
    } else {
        false
    }
}