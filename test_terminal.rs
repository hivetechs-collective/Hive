use std::io::{self, Write};

fn main() {
    // Test basic ANSI color support
    println!("Testing terminal ANSI support...\n");
    
    // Test colors
    print!("\x1b[31mRed text\x1b[0m ");
    print!("\x1b[32mGreen text\x1b[0m ");
    print!("\x1b[34mBlue text\x1b[0m\n");
    
    // Test cursor movement
    print!("\x1b[2J\x1b[H"); // Clear screen and move to top
    io::stdout().flush().unwrap();
    
    println!("If you see colored text above, ANSI codes work!");
    println!("Terminal: {:?}", std::env::var("TERM"));
    println!("Columns: {:?}", std::env::var("COLUMNS"));
    
    // Test raw mode capability
    match crossterm::terminal::enable_raw_mode() {
        Ok(_) => {
            println!("✅ Raw mode supported");
            let _ = crossterm::terminal::disable_raw_mode();
        }
        Err(e) => println!("❌ Raw mode error: {}", e),
    }
}