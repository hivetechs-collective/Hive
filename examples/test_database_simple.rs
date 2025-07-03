use hive_ai::core::database::{DatabaseManager, DatabaseConfig};
use tempfile::TempDir;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Testing database implementation (simple)...");
    
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_simple.db");
    
    // Configure database
    let config = DatabaseConfig {
        path: db_path.clone(),
        max_connections: 5,
        connection_timeout: Duration::from_secs(30),
        idle_timeout: Duration::from_secs(600),
        enable_wal: true,
        enable_foreign_keys: true,
        cache_size: -1000,
        synchronous: "NORMAL".to_string(),
        journal_mode: "WAL".to_string(),
    };
    
    // Create DatabaseManager directly (without migrations)
    println!("Creating database manager...");
    
    // Let's test just the connection pooling
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    
    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(config.max_connections)
        .build(manager)
        .unwrap();
    
    println!("✓ Connection pool created");
    
    // Test basic connection
    let conn = pool.get().unwrap();
    println!("✓ Got connection from pool");
    
    // Test basic query
    let result: i32 = conn.query_row("SELECT 1 + 1", [], |row| row.get(0)).unwrap();
    println!("✓ Basic query works: 1 + 1 = {}", result);
    
    // Create a simple table
    conn.execute(
        "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)",
        [],
    ).unwrap();
    println!("✓ Created test table");
    
    // Insert data
    conn.execute(
        "INSERT INTO test (name) VALUES (?1)",
        ["Hello, Database!"],
    ).unwrap();
    println!("✓ Inserted test data");
    
    // Query data
    let name: String = conn.query_row(
        "SELECT name FROM test WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap();
    println!("✓ Retrieved data: {}", name);
    
    println!("\nSimple database test completed successfully!");
}