//! Memory pools for zero-allocation consensus processing
//!
//! Implements object pooling to reduce allocations and improve performance.
//! Based on the CONSENSUS_ARCHITECTURE_2025.md design.

use bytes::{Bytes, BytesMut};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// Token object that can be reused from pool
#[derive(Debug, Clone)]
pub struct PooledToken {
    pub content: String,
    pub stage: Option<crate::consensus::types::Stage>,
    pub timestamp: std::time::Instant,
}

impl Default for PooledToken {
    fn default() -> Self {
        Self {
            content: String::with_capacity(256),
            stage: None,
            timestamp: std::time::Instant::now(),
        }
    }
}

impl PooledToken {
    /// Reset token for reuse
    pub fn reset(&mut self) {
        self.content.clear();
        self.stage = None;
        self.timestamp = std::time::Instant::now();
    }
}

/// Pool of reusable token objects
pub struct TokenPool {
    pool: Arc<Mutex<VecDeque<PooledToken>>>,
    max_size: usize,
}

impl TokenPool {
    /// Create a new token pool with specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut pool = VecDeque::with_capacity(capacity);

        // Pre-allocate tokens
        for _ in 0..capacity / 2 {
            pool.push_back(PooledToken::default());
        }

        Self {
            pool: Arc::new(Mutex::new(pool)),
            max_size: capacity,
        }
    }

    /// Acquire a token from the pool
    pub fn acquire(&self) -> PooledTokenGuard {
        let mut pool = self.pool.lock();

        let mut token = pool.pop_front().unwrap_or_else(PooledToken::default);
        token.reset();

        PooledTokenGuard {
            token: Some(token),
            pool: self.pool.clone(),
            max_size: self.max_size,
        }
    }

    /// Get current pool size
    pub fn size(&self) -> usize {
        self.pool.lock().len()
    }
}

/// RAII guard for pooled tokens
pub struct PooledTokenGuard {
    token: Option<PooledToken>,
    pool: Arc<Mutex<VecDeque<PooledToken>>>,
    max_size: usize,
}

impl PooledTokenGuard {
    /// Get mutable reference to the token
    pub fn as_mut(&mut self) -> &mut PooledToken {
        self.token.as_mut().expect("Token already returned")
    }

    /// Get reference to the token
    pub fn as_ref(&self) -> &PooledToken {
        self.token.as_ref().expect("Token already returned")
    }
}

impl Drop for PooledTokenGuard {
    fn drop(&mut self) {
        if let Some(mut token) = self.token.take() {
            let mut pool = self.pool.lock();

            // Only return to pool if under capacity
            if pool.len() < self.max_size {
                token.reset();
                pool.push_back(token);
            }
        }
    }
}

/// Pool for byte buffers
pub struct BufferPool {
    pool: Arc<Mutex<VecDeque<BytesMut>>>,
    buffer_size: usize,
    max_count: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(buffer_size: usize, max_count: usize) -> Self {
        let mut pool = VecDeque::with_capacity(max_count);

        // Pre-allocate some buffers
        for _ in 0..max_count / 4 {
            pool.push_back(BytesMut::with_capacity(buffer_size));
        }

        Self {
            pool: Arc::new(Mutex::new(pool)),
            buffer_size,
            max_count,
        }
    }

    /// Acquire a buffer from the pool
    pub fn acquire(&self) -> BufferGuard {
        let mut pool = self.pool.lock();

        let mut buffer = pool
            .pop_front()
            .unwrap_or_else(|| BytesMut::with_capacity(self.buffer_size));

        buffer.clear();

        BufferGuard {
            buffer: Some(buffer),
            pool: self.pool.clone(),
            max_count: self.max_count,
        }
    }
}

/// RAII guard for pooled buffers
pub struct BufferGuard {
    buffer: Option<BytesMut>,
    pool: Arc<Mutex<VecDeque<BytesMut>>>,
    max_count: usize,
}

impl BufferGuard {
    /// Get mutable reference to buffer
    pub fn as_mut(&mut self) -> &mut BytesMut {
        self.buffer.as_mut().expect("Buffer already returned")
    }

    /// Convert to Bytes (consumes the guard)
    pub fn freeze(mut self) -> Bytes {
        self.buffer
            .take()
            .expect("Buffer already returned")
            .freeze()
    }
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        if let Some(mut buffer) = self.buffer.take() {
            let mut pool = self.pool.lock();

            if pool.len() < self.max_count {
                buffer.clear();
                pool.push_back(buffer);
            }
        }
    }
}

/// Global memory pools for the consensus system
pub struct MemoryPools {
    pub tokens: TokenPool,
    pub buffers: BufferPool,
    pub small_strings: StringPool,
}

impl MemoryPools {
    /// Create all memory pools
    pub fn new() -> Self {
        Self {
            tokens: TokenPool::new(1000),
            buffers: BufferPool::new(8192, 100),
            small_strings: StringPool::new(256, 500),
        }
    }
}

/// Pool for small strings
pub struct StringPool {
    pool: Arc<Mutex<VecDeque<String>>>,
    capacity: usize,
    max_count: usize,
}

impl StringPool {
    pub fn new(capacity: usize, max_count: usize) -> Self {
        let mut pool = VecDeque::with_capacity(max_count);

        // Pre-allocate strings
        for _ in 0..max_count / 4 {
            pool.push_back(String::with_capacity(capacity));
        }

        Self {
            pool: Arc::new(Mutex::new(pool)),
            capacity,
            max_count,
        }
    }

    pub fn acquire(&self) -> StringGuard {
        let mut pool = self.pool.lock();

        let mut string = pool
            .pop_front()
            .unwrap_or_else(|| String::with_capacity(self.capacity));

        string.clear();

        StringGuard {
            string: Some(string),
            pool: self.pool.clone(),
            max_count: self.max_count,
        }
    }
}

pub struct StringGuard {
    string: Option<String>,
    pool: Arc<Mutex<VecDeque<String>>>,
    max_count: usize,
}

impl StringGuard {
    pub fn as_mut(&mut self) -> &mut String {
        self.string.as_mut().expect("String already returned")
    }

    pub fn take(mut self) -> String {
        self.string.take().expect("String already returned")
    }
}

impl Drop for StringGuard {
    fn drop(&mut self) {
        if let Some(mut string) = self.string.take() {
            let mut pool = self.pool.lock();

            if pool.len() < self.max_count {
                string.clear();
                pool.push_back(string);
            }
        }
    }
}

// Lazy static global pools
use once_cell::sync::Lazy;

pub static GLOBAL_POOLS: Lazy<MemoryPools> = Lazy::new(MemoryPools::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_pool() {
        let pool = TokenPool::new(10);

        // Acquire and release tokens
        {
            let mut guard = pool.acquire();
            guard.as_mut().content = "test".to_string();
        }

        // Token should be returned to pool
        assert!(pool.size() > 0);
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(1024, 5);

        let mut guard = pool.acquire();
        guard.as_mut().extend_from_slice(b"hello");

        let bytes = guard.freeze();
        assert_eq!(&bytes[..], b"hello");
    }
}
