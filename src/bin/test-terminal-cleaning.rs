//! Test terminal cleaning functionality

use hive_ai::desktop::terminal_buffer::TerminalBuffer;

fn main() {
    println!("Testing terminal buffer cleaning...\n");

    // Create a test buffer
    let mut buffer = TerminalBuffer::new();
    
    // Add some test lines with various escape sequences
    let test_lines = vec![
        "Normal text without any escape sequences",
        "[38;2;215;119;87m│[0m This is text with color codes",
        "\x1b[1mBold text\x1b[0m and normal text",
        "[2K[1G[38;2;215;119;87m│[0m  Claude Code output",
        "╭─────────────────────────────────────────╮",
        "│  🐝 HiveTechs Consensus                │",
        "╰─────────────────────────────────────────╯",
        "[?2004l[?2004huser@machine ~ % ",
        "✻ Thinking...",
        "⢿⣷⣯⣟⡿ Processing (42.3s)",
        "[38;5;208mOrange text[0m",
        "\x1b]0;Terminal Title\x07",
        "esc to interrupt | tokens: 1234",
        "[1A[2K[1G[38;2;119;215;87m✓[0m Task completed",
        "? for shortcuts",
        "║ ╔═════════════════════════════════════╗ ║",
        "[][][][][]345[][]",
    ];
    
    // Add lines to buffer
    for line in &test_lines {
        buffer.add_output(line);
    }
    
    println!("Original content:");
    println!("================");
    println!("{}", buffer.get_all_content());
    
    println!("\n\nCleaned content:");
    println!("================");
    println!("{}", buffer.get_cleaned_content());
    
    println!("\n\nTesting specific problematic patterns:");
    println!("=====================================");
    
    let problem_line = "[38;2;215;119;87m│[0m  This should be cleaned";
    let mut test_buffer = TerminalBuffer::new();
    test_buffer.add_output(problem_line);
    
    println!("Input:  '{}'", problem_line);
    println!("Output: '{}'", test_buffer.get_cleaned_content());
}