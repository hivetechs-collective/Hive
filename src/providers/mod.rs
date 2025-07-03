/// AI Provider Integrations
/// 
/// This module contains integrations with various AI model providers.
/// Currently supports OpenRouter with 323+ models.

pub mod openrouter;

// Re-export OpenRouter types for convenience
pub use openrouter::{
    create_client, create_streaming_client, OpenRouterClient, OpenRouterMessage,
    OpenRouterResponse,
};