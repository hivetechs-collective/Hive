//! Consensus Cancellation System
//! 
//! Provides graceful cancellation support for long-running consensus operations.
//! Users can interrupt the consensus pipeline at any stage without losing data.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;
use anyhow::Result;

/// Cancellation token for consensus operations
#[derive(Debug, Clone)]
pub struct CancellationToken {
    /// Internal cancellation flag
    cancelled: Arc<AtomicBool>,
    /// Broadcast sender for cancellation notifications
    sender: broadcast::Sender<CancellationReason>,
}

/// Receiver for cancellation notifications
pub type CancellationReceiver = broadcast::Receiver<CancellationReason>;

/// Reason for cancellation
#[derive(Debug, Clone)]
pub enum CancellationReason {
    /// User requested cancellation
    UserRequested,
    /// Timeout exceeded
    Timeout,
    /// System shutdown
    SystemShutdown,
    /// Error condition requiring cancellation
    Error(String),
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(16);
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            sender,
        }
    }

    /// Check if cancellation has been requested
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Cancel the operation with a specific reason
    pub fn cancel(&self, reason: CancellationReason) {
        if !self.cancelled.swap(true, Ordering::Relaxed) {
            // Only send notification on first cancellation
            let _ = self.sender.send(reason);
        }
    }

    /// Subscribe to cancellation notifications
    pub fn subscribe(&self) -> CancellationReceiver {
        self.sender.subscribe()
    }

    /// Throw cancellation error if cancelled
    pub fn throw_if_cancelled(&self) -> Result<()> {
        if self.is_cancelled() {
            anyhow::bail!("Operation was cancelled");
        }
        Ok(())
    }

    /// Create a child token that's cancelled when this token is cancelled
    pub fn child(&self) -> Self {
        let child = Self::new();
        
        // If parent is already cancelled, cancel child immediately
        if self.is_cancelled() {
            child.cancel(CancellationReason::UserRequested);
        } else {
            // Listen for parent cancellation
            let child_clone = child.clone();
            let mut receiver = self.subscribe();
            tokio::spawn(async move {
                if let Ok(reason) = receiver.recv().await {
                    child_clone.cancel(reason);
                }
            });
        }
        
        child
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Cancellable operation trait
#[async_trait::async_trait]
pub trait CancellableOperation {
    type Output;
    type Error;

    /// Execute the operation with cancellation support
    async fn execute_with_cancellation(
        &self,
        cancellation_token: &CancellationToken,
    ) -> Result<Self::Output, Self::Error>;
}

/// Utility for checking cancellation at regular intervals
pub struct CancellationChecker {
    token: CancellationToken,
    last_check: std::time::Instant,
    check_interval: std::time::Duration,
}

impl CancellationChecker {
    /// Create a new cancellation checker
    pub fn new(token: CancellationToken, check_interval: std::time::Duration) -> Self {
        Self {
            token,
            last_check: std::time::Instant::now(),
            check_interval,
        }
    }

    /// Check for cancellation if enough time has passed
    pub fn check_if_due(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_check) >= self.check_interval {
            self.last_check = now;
            self.token.throw_if_cancelled()?;
        }
        Ok(())
    }

    /// Force a cancellation check regardless of timing
    pub fn force_check(&self) -> Result<()> {
        self.token.throw_if_cancelled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel(CancellationReason::UserRequested);
        assert!(token.is_cancelled());
        assert!(token.throw_if_cancelled().is_err());
    }

    #[tokio::test]
    async fn test_cancellation_notification() {
        let token = CancellationToken::new();
        let mut receiver = token.subscribe();

        // Cancel in background
        let token_clone = token.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            token_clone.cancel(CancellationReason::UserRequested);
        });

        // Wait for notification
        let reason = receiver.recv().await.unwrap();
        matches!(reason, CancellationReason::UserRequested);
    }

    #[tokio::test]
    async fn test_child_token() {
        let parent = CancellationToken::new();
        let child = parent.child();

        assert!(!parent.is_cancelled());
        assert!(!child.is_cancelled());

        parent.cancel(CancellationReason::UserRequested);
        
        // Give child token time to receive parent cancellation
        sleep(Duration::from_millis(10)).await;
        
        assert!(parent.is_cancelled());
        assert!(child.is_cancelled());
    }
}