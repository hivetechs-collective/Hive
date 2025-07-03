use hive_ai::core::database_working::{initialize_database, DatabaseConfig, User, Conversation, Message};
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    println!("Testing database implementation...");
    
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Configure database
    let config = DatabaseConfig {
        path: db_path.clone(),
        ..Default::default()
    };
    
    // Initialize database
    initialize_database(Some(config)).await.unwrap();
    println!("✓ Database initialized");
    
    // Create a user
    let user = User::create(
        Some("test@example.com".to_string()),
        Some("test-license".to_string()),
    ).await.unwrap();
    println!("✓ User created: {}", user.id);
    
    // Create a conversation
    let conversation = Conversation::create(
        Some(user.id.clone()),
        None,
    ).await.unwrap();
    println!("✓ Conversation created: {}", conversation.id);
    
    // Create messages
    let msg1 = Message::create(
        conversation.id.clone(),
        "user".to_string(),
        "What is Rust?".to_string(),
        None,
        None,
    ).await.unwrap();
    println!("✓ Message 1 created: {}", msg1.id);
    
    let msg2 = Message::create(
        conversation.id.clone(),
        "assistant".to_string(),
        "Rust is a systems programming language.".to_string(),
        Some("generator".to_string()),
        Some("claude-3-opus".to_string()),
    ).await.unwrap();
    println!("✓ Message 2 created: {}", msg2.id);
    
    // Retrieve messages
    let messages = Message::find_by_conversation(&conversation.id).await.unwrap();
    println!("✓ Retrieved {} messages", messages.len());
    
    for (i, msg) in messages.iter().enumerate() {
        println!("  Message {}: {} - {}", i + 1, msg.role, msg.content);
    }
    
    println!("\nDatabase test completed successfully!");
}