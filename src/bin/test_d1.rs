use anyhow::Result;
use hive_ai::subscription::conversation_gateway::ConversationGateway;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Test both license keys
    let licenses = vec![
        ("verone.lazio@gmail.com", "HIVE-1SXK-DGW5-F15U-K9E1-L3IL"),
        ("sales@hivetechs.io", "HIVE-QACW-NFPX-5RKU-898D-S5RD"),
    ];

    for (email, key) in licenses {
        println!("\n{}", "=".repeat(80));
        println!("Testing {}: {}", email, key);
        println!("{}", "=".repeat(80));

        // Create gateway
        let gateway = match ConversationGateway::new() {
            Ok(g) => g,
            Err(e) => {
                println!("Failed to create gateway: {}", e);
                continue;
            }
        };

        // Test pre-conversation endpoint
        println!("\n1. Testing request_conversation_authorization:");
        match gateway
            .request_conversation_authorization("test_query", key)
            .await
        {
            Ok(auth) => {
                println!("✅ SUCCESS! Got ConversationAuthorization:");
                println!("  user_id: {}", auth.user_id);
                println!("  remaining: {:?}", auth.remaining);
                println!("  limit: {:?}", auth.limit);
                println!("  conversation_token: {}", auth.conversation_token);
                println!("  expires_at: {}", auth.expires_at);
            }
            Err(e) => {
                println!("❌ ERROR: {}", e);
                // Try to get the raw error details
                println!("Full error chain: {:?}", e);
            }
        }

        // Test validate endpoint
        println!("\n2. Testing validate_license_key:");
        match gateway.validate_license_key(key).await {
            Ok(profile) => {
                println!("✅ SUCCESS! Got UserProfile:");
                println!("  user_id: {}", profile.user_id);
                println!("  email: {}", profile.email);
                println!("  tier: {}", profile.tier);
                println!("  daily_limit: {}", profile.daily_limit);
                println!("  features: {:?}", profile.features);
                println!("  is_valid: {}", profile.is_valid);
            }
            Err(e) => {
                println!("❌ ERROR: {}", e);
                println!("Full error chain: {:?}", e);
            }
        }

        // Small delay between tests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(())
}
