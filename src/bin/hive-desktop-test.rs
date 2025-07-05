//! Simple test desktop app with visible content

use dioxus::prelude::*;

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(dioxus::desktop::Config::new().with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Hive AI Desktop Test")
                .with_inner_size(dioxus::desktop::LogicalSize::new(800, 600))
        ))
        .launch(app);
}

fn app() -> Element {
    rsx! {
        div {
            style: "background-color: #1e1e1e; color: white; padding: 20px; height: 100vh; font-family: system-ui;",
            h1 { 
                style: "color: #007acc; margin-bottom: 20px;",
                "🐝 Hive AI Desktop - Test Version" 
            }
            div {
                style: "background-color: #252526; padding: 20px; border-radius: 8px; margin-bottom: 20px;",
                p { "✅ Window is rendering correctly!" }
                p { "✅ Styles are being applied!" }
                p { "✅ You should see this text!" }
            }
            div {
                style: "background-color: #3c3c3c; padding: 20px; border-radius: 8px;",
                p { style: "color: #4ec9b0;", "If you can see this, the desktop app is working." }
                p { style: "color: #9cdcfe;", "The main app might have more complex styling issues." }
                button {
                    style: "background-color: #0e639c; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer; margin-top: 10px;",
                    onclick: move |_| println!("Button clicked!"),
                    "Click Me!"
                }
            }
        }
    }
}