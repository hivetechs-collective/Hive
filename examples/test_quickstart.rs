use hive_ai::commands::{handle_quickstart, QuickstartOptions};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();
    
    println!("Testing quickstart command...\n");
    
    let options = QuickstartOptions {
        skip_ide: true,
        skip_server: true,
        silent: false,
        no_auto_start: false,
        force: false,
        clear_cache: false,
    };
    
    match handle_quickstart(options).await {
        Ok(_) => println!("\nQuickstart completed successfully!"),
        Err(e) => eprintln!("\nQuickstart failed: {}", e),
    }
}