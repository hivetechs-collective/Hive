use tempfile::TempDir;

// Import the database types directly
use hive_ai::core::database_working::{
    initialize_database, DatabaseConfig, 
    User, Conversation, Message,
    get_database
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing working database implementation...\n");
    
    // Create temporary directory
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_working.db");
    println!("Database path: {:?}", db_path);
    
    // Configure database
    let config = DatabaseConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    
    // Initialize database
    println!("Initializing database...");
    initialize_database(Some(config)).await?;
    println!("✓ Database initialized");
    
    // Test health check
    let db = get_database().await?;
    let health = db.health_check().await?;
    println!("\n✓ Database health check:");
    println!("  - Healthy: {}", health.healthy);
    println!("  - WAL mode: {}", health.wal_mode_active);
    println!("  - Foreign keys: {}", health.foreign_keys_enabled);
    println!("  - Response time: {:?}", health.response_time);
    
    // Create a user
    println!("\nCreating user...");
    let user = User::create(
        Some("test@example.com".to_string()),
        Some("test-license-key".to_string()),
    ).await?;
    println!("✓ User created with ID: {}", user.id);
    
    // Find user by ID
    let found_user = User::find_by_id(&user.id).await?;
    assert!(found_user.is_some());
    println!("✓ User found by ID");
    
    // Create a conversation
    println!("\nCreating conversation...");
    let conversation = Conversation::create(
        Some(user.id.clone()),
        None,
    ).await?;
    println!("✓ Conversation created with ID: {}", conversation.id);
    
    // Create messages
    println!("\nCreating messages...");
    let msg1 = Message::create(
        conversation.id.clone(),
        "user".to_string(),
        "What is Rust?".to_string(),
        None,
        None,
    ).await?;
    println!("✓ Message 1 created");
    
    let msg2 = Message::create(
        conversation.id.clone(),
        "assistant".to_string(),
        "Rust is a systems programming language focused on safety, speed, and concurrency.".to_string(),
        Some("generator".to_string()),
        Some("claude-3-opus".to_string()),
    ).await?;
    println!("✓ Message 2 created");
    
    // Retrieve messages
    println!("\nRetrieving messages...");
    let messages = Message::find_by_conversation(&conversation.id).await?;
    println!("✓ Retrieved {} messages:", messages.len());
    
    for (i, msg) in messages.iter().enumerate() {
        println!("  {}. [{}] {}", i + 1, msg.role, msg.content);
    }
    
    // Get statistics
    println!("\nDatabase statistics:");
    let stats = db.get_statistics().await?;
    println!("  - Users: {}", stats.user_count);
    println!("  - Conversations: {}", stats.conversation_count);
    println!("  - Messages: {}", stats.message_count);
    println!("  - Pool size: {}", stats.pool_size);
    println!("  - Idle connections: {}", stats.idle_connections);
    
    println!("\n✅ All database tests passed successfully!");
    Ok(())
}