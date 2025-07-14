use dioxus::prelude::*;

fn main() {
    // Launch the desktop app
    dioxus_desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    let input_value = use_state(cx, || String::new());
    let messages = use_state(cx, || vec![
        "Welcome to Hive AI Desktop!".to_string(),
        "This is a minimal working version.".to_string(),
    ]);

    cx.render(rsx! {
        style { include_str!("../desktop/styles/mod.rs") }
        div {
            style: "display: flex; flex-direction: column; height: 100vh; background: #1e1e1e; color: #d4d4d4; font-family: -apple-system, system-ui, sans-serif;",

            // Header
            div {
                style: "background: #2d2d30; padding: 10px; border-bottom: 1px solid #3e3e42;",
                h1 {
                    style: "margin: 0; font-size: 16px; font-weight: 500;",
                    "🐝 Hive AI Desktop"
                }
            }

            // Messages area
            div {
                style: "flex: 1; overflow-y: auto; padding: 20px;",
                messages.iter().map(|msg| rsx! {
                    div {
                        style: "margin-bottom: 10px; padding: 10px; background: #2d2d30; border-radius: 4px;",
                        "{msg}"
                    }
                })
            }

            // Input area
            div {
                style: "padding: 20px; border-top: 1px solid #3e3e42;",
                div {
                    style: "display: flex; gap: 10px;",
                    input {
                        style: "flex: 1; padding: 8px 12px; background: #3c3c3c; border: 1px solid #464647; border-radius: 4px; color: #d4d4d4; font-size: 14px;",
                        value: "{input_value}",
                        placeholder: "Type your message...",
                        oninput: move |evt| input_value.set(evt.value.clone()),
                        onkeypress: move |evt| {
                            if evt.key() == Key::Enter && !input_value.is_empty() {
                                let msg = input_value.get().clone();
                                messages.modify(|msgs| {
                                    msgs.push(format!("You: {}", msg));
                                    msgs.push("Hive: I'm a minimal version - full consensus engine coming soon!".to_string());
                                });
                                input_value.set(String::new());
                            }
                        }
                    }
                    button {
                        style: "padding: 8px 16px; background: #007ACC; border: none; border-radius: 4px; color: white; cursor: pointer; font-size: 14px;",
                        onclick: move |_| {
                            if !input_value.is_empty() {
                                let msg = input_value.get().clone();
                                messages.modify(|msgs| {
                                    msgs.push(format!("You: {}", msg));
                                    msgs.push("Hive: I'm a minimal version - full consensus engine coming soon!".to_string());
                                });
                                input_value.set(String::new());
                            }
                        },
                        "Send"
                    }
                }
            }
        }
    })
}