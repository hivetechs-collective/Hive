//! Semantic Fingerprinting for deduplication and similarity search
//!
//! This module provides semantic fingerprinting capabilities to identify
//! duplicate or similar facts based on meaning rather than exact text matching.

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// A semantic fingerprint representing the meaning of text
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticFingerprint(pub Vec<u8>);

impl SemanticFingerprint {
    /// Create from raw bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl ToString for SemanticFingerprint {
    fn to_string(&self) -> String {
        self.to_hex()
    }
}

/// Creates semantic fingerprints for text deduplication
pub struct SemanticFingerprinter {
    // TODO: In future, this could use AI embeddings for true semantic similarity
    // For now, we'll use a simple hash-based approach
}

impl SemanticFingerprinter {
    /// Create a new fingerprinter
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    /// Generate a semantic fingerprint for text
    pub async fn fingerprint(&self, text: &str) -> Result<SemanticFingerprint> {
        // TODO: In future, use AI models to generate semantic embeddings
        // For now, use a simple normalized hash approach

        // Normalize text: lowercase, remove extra whitespace
        let normalized = text
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Generate hash
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        let result = hasher.finalize();

        Ok(SemanticFingerprint(result.to_vec()))
    }

    /// Calculate similarity between two fingerprints
    pub fn similarity(&self, fp1: &SemanticFingerprint, fp2: &SemanticFingerprint) -> f32 {
        // TODO: Once we have embeddings, this would be cosine similarity
        // For now, exact match only
        if fp1 == fp2 {
            1.0
        } else {
            0.0
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fingerprint_generation() {
        let fingerprinter = SemanticFingerprinter::new().await.unwrap();

        // Same content should produce same fingerprint
        let fp1 = fingerprinter.fingerprint("Hello World").await.unwrap();
        let fp2 = fingerprinter.fingerprint("hello world").await.unwrap();
        assert_eq!(fp1, fp2);

        // Different content should produce different fingerprint
        let fp3 = fingerprinter.fingerprint("Goodbye World").await.unwrap();
        assert_ne!(fp1, fp3);
    }

    #[tokio::test]
    async fn test_normalization() {
        let fingerprinter = SemanticFingerprinter::new().await.unwrap();

        // Extra whitespace should be normalized
        let fp1 = fingerprinter.fingerprint("Hello   World").await.unwrap();
        let fp2 = fingerprinter.fingerprint("Hello World").await.unwrap();
        assert_eq!(fp1, fp2);
    }
}
