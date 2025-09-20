// Example: Beautiful Curator Result Display in TUI
// Demonstrates the enhanced visual presentation of consensus results

use crate::consensus::formatted_result::*;
use crate::consensus::types::*;
use chrono::Utc;

fn main() {
    // Create sample metadata (simulating a real consensus run)
    let metadata = ResponseMetadata {
        total_tokens: 1250,
        cost: 0.0342,
        duration_ms: 2840,
        models_used: vec![
            "anthropic/claude-3-sonnet-20240229".to_string(),
            "openai/gpt-4-turbo-preview".to_string(),
            "google/gemini-pro".to_string(),
            "meta-llama/llama-2-70b-chat".to_string(),
        ],
    };

    // Create sample stage results
    let stages = vec![
        create_stage_result(
            "generator",
            "anthropic/claude-3-sonnet-20240229",
            320,
            0.0089,
            0.8,
            720,
        ),
        create_stage_result(
            "refiner",
            "openai/gpt-4-turbo-preview",
            380,
            0.0124,
            0.92,
            880,
        ),
        create_stage_result("validator", "google/gemini-pro", 290, 0.0067, 0.87, 640),
        create_stage_result(
            "curator",
            "meta-llama/llama-2-70b-chat",
            260,
            0.0062,
            0.95,
            600,
        ),
    ];

    // Sample curator output (markdown-formatted response)
    let curator_content = r#"## 📋 Summary

This response provides a comprehensive solution for implementing authentication in your React application.

Key benefits:
- Secure JWT-based authentication
- Role-based access control
- Persistent login sessions
- Comprehensive error handling

## 💻 Implementation

Here's the complete authentication implementation:

```javascript
// AuthContext.js
import React, { createContext, useContext, useState, useEffect } from 'react';

const AuthContext = createContext();

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
```

## 🔒 Security Considerations

IMPORTANT: Always validate tokens on the server side and implement proper CORS policies.

## 🎯 Next Steps

1. Set up JWT token validation middleware
2. Implement role-based route protection
3. Add refresh token mechanism
4. Configure secure cookie settings
5. Set up proper error boundaries"#;

    // Create formatted result
    let formatted_result =
        FormattedConsensusResult::from_curator_output(curator_content, metadata, stages);

    // Display the beautiful formatted result
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║                     BEAUTIFUL CURATOR RESULT DEMO                       ║");
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();

    println!("{}", formatted_result.format_for_display());

    println!("\n\n🎨 This demonstrates the beautiful visual presentation that users will see");
    println!("   in the TUI when consensus completes, with:");
    println!("   • Executive summary boxes with key points and action items");
    println!("   • Visual section separators and emoji icons");
    println!("   • Performance metrics with progress bars");
    println!("   • Cost breakdown with currency highlighting");
    println!("   • 4-stage journey visualization");
    println!("   • Confidence scoring with visual indicators");
}

fn create_stage_result(
    stage: &str,
    model: &str,
    tokens: u32,
    cost: f64,
    quality: f64,
    duration_ms: u64,
) -> StageResult {
    StageResult {
        stage_id: format!("{}-{}", stage, Utc::now().timestamp()),
        stage_name: stage.to_string(),
        question: "How do I implement authentication in React?".to_string(),
        answer: format!(
            "Sample {} response for authentication implementation",
            stage
        ),
        model: model.to_string(),
        conversation_id: "demo-conversation-123".to_string(),
        timestamp: Utc::now(),
        usage: Some(TokenUsage {
            prompt_tokens: tokens / 3,
            completion_tokens: (tokens * 2) / 3,
            total_tokens: tokens,
        }),
        analytics: Some(StageAnalytics {
            duration: duration_ms as f64 / 1000.0,
            cost,
            provider: model.split('/').next().unwrap_or("unknown").to_string(),
            model_internal_id: model.to_string(),
            quality_score: quality,
            error_count: 0,
            fallback_used: false,
            rate_limit_hit: false,
            retry_count: 0,
            start_time: Utc::now() - chrono::Duration::milliseconds(duration_ms as i64),
            end_time: Utc::now(),
            memory_usage: Some(1024 * 1024 * 64), // 64MB
            features: AnalyticsFeatures {
                streaming: true,
                routing_variant: "standard".to_string(),
                optimization_applied: Some(true),
            },
        }),
    }
}
