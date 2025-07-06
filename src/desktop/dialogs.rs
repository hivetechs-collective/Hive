//! Dialog components for the desktop application

use dioxus::prelude::*;

/// About dialog component
#[component]
pub fn AboutDialog(show_about: Signal<bool>) -> Element {
    if !*show_about.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| *show_about.write() = false,
            
            div {
                class: "dialog-box about-dialog",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    h2 { "About Hive Consensus" }
                    button {
                        class: "dialog-close",
                        onclick: move |_| *show_about.write() = false,
                        "×"
                    }
                }
                
                div {
                    class: "dialog-content",
                    div { class: "app-icon", "🐝" }
                    h3 { "Hive Consensus" }
                    p { "Version {env!(\"CARGO_PKG_VERSION\")}" }
                    p { "A VS Code-inspired AI development environment" }
                    
                    div {
                        class: "dialog-features",
                        h4 { "Features:" }
                        ul {
                            li { "✓ Multi-model consensus engine" }
                            li { "✓ Real-time code analysis" }
                            li { "✓ Integrated file explorer" }
                            li { "✓ Syntax highlighting" }
                            li { "✓ Git integration" }
                        }
                    }
                    
                    p {
                        class: "dialog-footer-text",
                        "Built with Rust and Dioxus"
                    }
                }
            }
        }
    }
}

/// Welcome dialog/tab component
#[component]
pub fn WelcomeTab(show_welcome: Signal<bool>) -> Element {
    if !*show_welcome.read() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "welcome-tab",
            
            div {
                class: "welcome-header",
                h1 { "Welcome to Hive Consensus" }
                p { "Get started with your AI-powered development environment" }
            }
            
            div {
                class: "welcome-sections",
                
                div {
                    class: "welcome-section",
                    h3 { "🚀 Quick Start" }
                    button { 
                        class: "welcome-button",
                        "Open Folder"
                    }
                    button { 
                        class: "welcome-button",
                        "Open Recent"
                    }
                    button { 
                        class: "welcome-button",
                        "New File"
                    }
                }
                
                div {
                    class: "welcome-section",
                    h3 { "💡 Tips" }
                    ul {
                        li { "Use Cmd/Ctrl+Shift+P for Command Palette" }
                        li { "Ask the AI assistant anything about your code" }
                        li { "Click files to view and edit them" }
                    }
                }
                
                div {
                    class: "welcome-section",
                    h3 { "📚 Resources" }
                    a { 
                        href: "#",
                        class: "welcome-link",
                        "Documentation"
                    }
                    a { 
                        href: "#",
                        class: "welcome-link",
                        "Keyboard Shortcuts"
                    }
                    a { 
                        href: "#",
                        class: "welcome-link",
                        "Report Issue"
                    }
                }
            }
            
            button {
                class: "welcome-close",
                onclick: move |_| *show_welcome.write() = false,
                "Close Welcome"
            }
        }
    }
}

/// Command palette component
#[component]
pub fn CommandPalette(show_palette: Signal<bool>) -> Element {
    let mut search_query = use_signal(String::new);
    
    if !*show_palette.read() {
        return rsx! {};
    }

    const COMMANDS: &[(&str, &str)] = &[
        ("Open File", "Ctrl+O"),
        ("Open Folder", "Ctrl+K Ctrl+O"),
        ("Save", "Ctrl+S"),
        ("Save As", "Ctrl+Shift+S"),
        ("Find", "Ctrl+F"),
        ("Replace", "Ctrl+H"),
        ("Toggle Terminal", "Ctrl+`"),
        ("Settings", "Ctrl+,"),
    ];

    let filtered_commands: Vec<_> = COMMANDS.iter()
        .filter(|(cmd, _)| {
            (*search_query.read()).is_empty() || 
            cmd.to_lowercase().contains(&(*search_query.read()).to_lowercase())
        })
        .collect();

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| *show_palette.write() = false,
            
            div {
                class: "command-palette",
                onclick: move |e| e.stop_propagation(),
                
                input {
                    class: "command-palette-input",
                    placeholder: "Type a command...",
                    value: "{search_query.read()}",
                    oninput: move |evt| *search_query.write() = evt.value().clone(),
                    autofocus: true,
                }
                
                div {
                    class: "command-palette-results",
                    for (cmd, shortcut) in filtered_commands {
                        div {
                            class: "command-palette-item",
                            onclick: move |_| {
                                println!("Execute command: {}", cmd);
                                *show_palette.write() = false;
                            },
                            span { class: "command-name", "{cmd}" }
                            span { class: "command-shortcut", "{shortcut}" }
                        }
                    }
                }
            }
        }
    }
}

/// CSS styles for dialogs
pub const DIALOG_STYLES: &str = r#"
    .dialog-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }
    
    .dialog-box {
        background: #2d2d30;
        border: 1px solid #3e3e42;
        border-radius: 8px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        max-width: 90%;
        max-height: 90vh;
        overflow: auto;
    }
    
    .about-dialog {
        width: 500px;
    }
    
    .dialog-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 20px;
        border-bottom: 1px solid #3e3e42;
    }
    
    .dialog-header h2 {
        margin: 0;
        font-size: 20px;
        color: #ffffff;
    }
    
    .dialog-close {
        background: none;
        border: none;
        color: #cccccc;
        font-size: 24px;
        cursor: pointer;
        padding: 0;
        width: 30px;
        height: 30px;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .dialog-close:hover {
        color: #ffffff;
        background: #3e3e42;
        border-radius: 4px;
    }
    
    .dialog-content {
        padding: 30px;
        text-align: center;
    }
    
    .app-icon {
        font-size: 64px;
        margin-bottom: 20px;
    }
    
    .dialog-content h3 {
        margin: 10px 0;
        font-size: 24px;
        color: #ffffff;
    }
    
    .dialog-content p {
        margin: 10px 0;
        color: #cccccc;
    }
    
    .dialog-features {
        margin: 30px 0;
        text-align: left;
    }
    
    .dialog-features h4 {
        margin-bottom: 10px;
        color: #ffffff;
    }
    
    .dialog-features ul {
        list-style: none;
        padding: 0;
    }
    
    .dialog-features li {
        padding: 5px 0;
        color: #cccccc;
    }
    
    .dialog-footer-text {
        margin-top: 30px;
        font-size: 12px;
        color: #858585;
    }
    
    /* Welcome tab styles */
    .welcome-tab {
        padding: 40px;
        max-width: 800px;
        margin: 0 auto;
    }
    
    .welcome-header {
        text-align: center;
        margin-bottom: 40px;
    }
    
    .welcome-header h1 {
        font-size: 36px;
        margin-bottom: 10px;
    }
    
    .welcome-sections {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 30px;
        margin-bottom: 40px;
    }
    
    .welcome-section {
        background: #2d2d30;
        padding: 20px;
        border-radius: 8px;
        border: 1px solid #3e3e42;
    }
    
    .welcome-section h3 {
        margin-bottom: 15px;
    }
    
    .welcome-button {
        display: block;
        width: 100%;
        padding: 10px;
        margin: 5px 0;
        background: #007acc;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
    }
    
    .welcome-button:hover {
        background: #005a9e;
    }
    
    .welcome-link {
        display: block;
        padding: 5px 0;
        color: #007acc;
        text-decoration: none;
    }
    
    .welcome-link:hover {
        text-decoration: underline;
    }
    
    .welcome-close {
        display: block;
        margin: 0 auto;
        padding: 10px 30px;
        background: #3e3e42;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
    }
    
    .welcome-close:hover {
        background: #4e4e52;
    }
    
    /* Command palette styles */
    .command-palette {
        background: #252526;
        border: 1px solid #3e3e42;
        border-radius: 8px;
        width: 600px;
        max-height: 400px;
        overflow: hidden;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    }
    
    .command-palette-input {
        width: 100%;
        padding: 15px 20px;
        background: #3c3c3c;
        border: none;
        border-bottom: 1px solid #3e3e42;
        color: #ffffff;
        font-size: 16px;
        outline: none;
    }
    
    .command-palette-results {
        max-height: 350px;
        overflow-y: auto;
    }
    
    .command-palette-item {
        display: flex;
        justify-content: space-between;
        padding: 10px 20px;
        cursor: pointer;
    }
    
    .command-palette-item:hover {
        background: #094771;
    }
    
    .command-name {
        color: #cccccc;
    }
    
    .command-shortcut {
        color: #858585;
        font-size: 12px;
    }
"#;