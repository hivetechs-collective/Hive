#!/bin/bash

# Create a temporary standalone desktop app
cat > /tmp/hive_desktop_standalone.rs << 'EOF'
use dioxus::prelude::*;

fn main() {
    dioxus_desktop::launch_cfg(
        app,
        dioxus_desktop::Config::new()
            .with_window(dioxus_desktop::WindowBuilder::new()
                .with_title("Hive AI Desktop")
                .with_inner_size(dioxus_desktop::LogicalSize::new(1200.0, 800.0))
            )
    );
}

fn app(cx: Scope) -> Element {
    let input_value = use_state(cx, || String::new());
    let messages = use_state(cx, || vec![
        "🐝 Welcome to Hive AI Desktop!".to_string(),
        "This is a minimal working version while we fix the compilation issues.".to_string(),
    ]);

    cx.render(rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; background: #1e1e1e; color: #d4d4d4; font-family: -apple-system, system-ui, sans-serif;",
            
            // Header
            div {
                style: "background: #2d2d30; padding: 10px 20px; border-bottom: 1px solid #3e3e42; display: flex; align-items: center; justify-content: space-between;",
                div {
                    style: "display: flex; align-items: center; gap: 10px;",
                    h1 { 
                        style: "margin: 0; font-size: 16px; font-weight: 500;",
                        "🐝 Hive AI Desktop" 
                    }
                    span {
                        style: "color: #858585; font-size: 12px;",
                        "v2.0.2 (Rust Edition)"
                    }
                }
                div {
                    style: "display: flex; gap: 10px;",
                    span {
                        style: "color: #4EC9B0;",
                        "✓ Ready"
                    }
                }
            }
            
            // Main content
            div {
                style: "flex: 1; display: flex;",
                
                // Sidebar
                div {
                    style: "width: 200px; background: #252526; border-right: 1px solid #3e3e42; padding: 10px;",
                    div {
                        style: "color: #858585; font-size: 11px; text-transform: uppercase; margin-bottom: 10px;",
                        "Explorer"
                    }
                    div {
                        style: "color: #cccccc; font-size: 13px; padding: 5px; cursor: pointer;",
                        "📁 Project Files"
                    }
                    div {
                        style: "color: #cccccc; font-size: 13px; padding: 5px; cursor: pointer;",
                        "🔍 Search"
                    }
                    div {
                        style: "color: #cccccc; font-size: 13px; padding: 5px; cursor: pointer;",
                        "⚙️ Settings"
                    }
                }
                
                // Chat area
                div {
                    style: "flex: 1; display: flex; flex-direction: column;",
                    
                    // Messages
                    div {
                        style: "flex: 1; overflow-y: auto; padding: 20px;",
                        messages.iter().map(|msg| rsx! {
                            div {
                                style: "margin-bottom: 10px; padding: 12px 16px; background: #2d2d30; border-radius: 4px; border-left: 3px solid #007ACC;",
                                "{msg}"
                            }
                        })
                    }
                    
                    // Input area
                    div {
                        style: "padding: 20px; background: #252526; border-top: 1px solid #3e3e42;",
                        div {
                            style: "display: flex; gap: 10px;",
                            input {
                                style: "flex: 1; padding: 10px 14px; background: #3c3c3c; border: 1px solid #464647; border-radius: 4px; color: #d4d4d4; font-size: 14px; outline: none;",
                                value: "{input_value}",
                                placeholder: "Ask Hive anything...",
                                oninput: move |evt| input_value.set(evt.value.clone()),
                                onkeypress: move |evt| {
                                    if evt.key() == Key::Enter && !input_value.is_empty() {
                                        let msg = input_value.get().clone();
                                        messages.modify(|msgs| {
                                            msgs.push(format!("You: {}", msg));
                                            msgs.push("🐝 Hive: I'm currently in minimal mode while the full consensus engine is being fixed. The VS Code-style interface is working though!".to_string());
                                        });
                                        input_value.set(String::new());
                                    }
                                }
                            }
                            button {
                                style: "padding: 10px 20px; background: #007ACC; border: none; border-radius: 4px; color: white; cursor: pointer; font-size: 14px; font-weight: 500;",
                                onclick: move |_| {
                                    if !input_value.is_empty() {
                                        let msg = input_value.get().clone();
                                        messages.modify(|msgs| {
                                            msgs.push(format!("You: {}", msg));
                                            msgs.push("🐝 Hive: I'm currently in minimal mode while the full consensus engine is being fixed. The VS Code-style interface is working though!".to_string());
                                        });
                                        input_value.set(String::new());
                                    }
                                },
                                "Send"
                            }
                        }
                    }
                }
                
                // Right panel (consensus stages)
                div {
                    style: "width: 300px; background: #252526; border-left: 1px solid #3e3e42; padding: 20px;",
                    div {
                        style: "color: #858585; font-size: 11px; text-transform: uppercase; margin-bottom: 15px;",
                        "Consensus Pipeline"
                    }
                    div {
                        style: "margin-bottom: 15px;",
                        div {
                            style: "display: flex; align-items: center; gap: 10px; margin-bottom: 8px;",
                            div {
                                style: "width: 20px; height: 20px; background: #4EC9B0; border-radius: 50%;",
                            }
                            span { style: "color: #cccccc; font-size: 13px;", "Generator" }
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 10px; margin-bottom: 8px;",
                            div {
                                style: "width: 20px; height: 20px; background: #569CD6; border-radius: 50%;",
                            }
                            span { style: "color: #cccccc; font-size: 13px;", "Refiner" }
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 10px; margin-bottom: 8px;",
                            div {
                                style: "width: 20px; height: 20px; background: #CE9178; border-radius: 50%;",
                            }
                            span { style: "color: #cccccc; font-size: 13px;", "Validator" }
                        }
                        div {
                            style: "display: flex; align-items: center; gap: 10px;",
                            div {
                                style: "width: 20px; height: 20px; background: #D7BA7D; border-radius: 50%;",
                            }
                            span { style: "color: #cccccc; font-size: 13px;", "Curator" }
                        }
                    }
                }
            }
            
            // Status bar
            div {
                style: "background: #007ACC; padding: 5px 20px; display: flex; justify-content: space-between; align-items: center; font-size: 12px;",
                div {
                    style: "display: flex; gap: 20px;",
                    span { "Rust Desktop Mode" }
                    span { "UTF-8" }
                }
                div {
                    span { "Ready" }
                }
            }
        }
    })
}
EOF

# Create a temporary Cargo.toml
cat > /tmp/Cargo.toml << 'EOF'
[package]
name = "hive-desktop-standalone"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = { version = "0.5", features = ["desktop"] }
dioxus-desktop = "0.5"
EOF

# Build and run
cd /tmp
cargo run --release