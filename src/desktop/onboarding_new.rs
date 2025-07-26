//! New simplified onboarding dialog that actually closes

use dioxus::prelude::*;
use crate::desktop::dependency_checker::{check_dependencies, install_claude_code, DependencyStatus};

/// Onboarding Dialog - completely rewritten to work properly
#[component]
pub fn OnboardingDialog(
    show_onboarding: Signal<bool>,
    openrouter_key: Signal<String>,
    hive_key: Signal<String>,
    current_step: Signal<i32>
) -> Element {
    // CRITICAL: Check visibility inside the component and return None if hidden
    let visible = show_onboarding();

    if !visible {
        tracing::debug!("OnboardingDialog: Not visible, returning None");
        return rsx! {};
    }

    tracing::info!("OnboardingDialog: Rendering (visible=true)");

    // Local state
    let mut temp_openrouter_key = use_signal(|| String::new());
    let mut temp_hive_key = use_signal(|| String::new());
    let mut dependencies = use_signal(|| Vec::<DependencyStatus>::new());
    let mut dependency_check_done = use_signal(|| false);
    let mut installing_claude = use_signal(|| false);
    let mut install_progress = use_signal(|| String::new());

    // Simple close function
    let mut close = move || {
        tracing::info!("CLOSING DIALOG - setting show_onboarding to false");
        show_onboarding.set(false);
    };

    rsx! {
        div {
            class: "dialog-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| close(),

            div {
                class: "dialog onboarding-dialog",
                style: "background: #2d2d30; border: 1px solid #3e3e42; border-radius: 8px; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4); width: 600px; overflow: hidden;",
                onclick: move |evt| evt.stop_propagation(),

                // Header with close button
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 20px; border-bottom: 1px solid #3e3e42;",
                    h2 {
                        style: "margin: 0; font-size: 20px; color: #ffffff;",
                        "🐝 Welcome to Hive Consensus"
                    }
                    button {
                        style: "background: none; border: none; color: #cccccc; font-size: 24px; cursor: pointer; padding: 0; width: 30px; height: 30px;",
                        onclick: move |_| close(),
                        "×"
                    }
                }

                // Content
                div {
                    style: "padding: 30px;",

                    match current_step() {
                        1 => rsx! {
                            div {
                                h3 {
                                    style: "margin: 0 0 15px 0; color: #ffffff;",
                                    "Let's get you started!"
                                }
                                p {
                                    style: "color: #cccccc;",
                                    "Hive Consensus uses advanced AI models to provide the best possible responses."
                                }
                            }
                        },
                        2 => rsx! {
                            div {
                                h3 {
                                    style: "margin: 0 0 15px 0; color: #ffffff;",
                                    "🔍 Checking Dependencies"
                                }
                                
                                // Run dependency check on first render of this step
                                if !dependency_check_done() {
                                    div {
                                        onmounted: move |_| {
                                            spawn(async move {
                                                let deps = check_dependencies().await;
                                                dependencies.set(deps);
                                                dependency_check_done.set(true);
                                            });
                                        },
                                        p {
                                            style: "color: #cccccc;",
                                            "Checking system requirements..."
                                        }
                                    }
                                } else {
                                    // Show dependency status
                                    div {
                                        style: "margin-top: 20px;",
                                        for dep in dependencies() {
                                            div {
                                                style: "margin-bottom: 15px; padding: 10px; background: #3c3c3c; border-radius: 4px;",
                                                div {
                                                    style: "display: flex; justify-content: space-between; align-items: center;",
                                                    span {
                                                        style: "color: #ffffff; font-weight: bold;",
                                                        "{dep.name}"
                                                    }
                                                    if dep.installed {
                                                        span {
                                                            style: "color: #4ec9b0;",
                                                            "✅ Installed"
                                                        }
                                                    } else {
                                                        span {
                                                            style: "color: #f48771;",
                                                            "❌ Not installed"
                                                        }
                                                    }
                                                }
                                                if let Some(version) = &dep.version {
                                                    p {
                                                        style: "color: #8b8b8b; margin: 5px 0 0 0; font-size: 12px;",
                                                        "{version}"
                                                    }
                                                }
                                                if !dep.installed && dep.name == "Claude Code" && !installing_claude() {
                                                    button {
                                                        style: "margin-top: 10px; padding: 6px 12px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                                        onclick: move |_| {
                                                            installing_claude.set(true);
                                                            spawn(async move {
                                                                use std::sync::{Arc, Mutex};
                                                                let progress_ref = Arc::new(Mutex::new(install_progress.clone()));
                                                                match install_claude_code(move |msg| {
                                                                    if let Ok(mut progress) = progress_ref.lock() {
                                                                        progress.set(msg);
                                                                    }
                                                                }).await {
                                                                    Ok(_) => {
                                                                        // Refresh dependencies
                                                                        let deps = check_dependencies().await;
                                                                        dependencies.set(deps);
                                                                        installing_claude.set(false);
                                                                    }
                                                                    Err(e) => {
                                                                        install_progress.set(format!("Error: {}", e));
                                                                        installing_claude.set(false);
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        "Install Automatically"
                                                    }
                                                }
                                            }
                                        }
                                        
                                        if installing_claude() {
                                            div {
                                                style: "margin-top: 20px; padding: 15px; background: #2d2d30; border: 1px solid #3e3e42; border-radius: 4px;",
                                                p {
                                                    style: "color: #cccccc; margin: 0;",
                                                    "📦 {install_progress}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        3 => rsx! {
                            div {
                                h3 {
                                    style: "margin: 0 0 15px 0; color: #ffffff;",
                                    "Configure Your License (Optional)"
                                }
                                input {
                                    style: "width: 100%; padding: 8px 12px; background: #3c3c3c; border: 1px solid #3e3e42; border-radius: 4px; color: #ffffff;",
                                    r#type: "password",
                                    value: "{temp_hive_key}",
                                    placeholder: "hive-xxxx-xxxx-xxxx",
                                    oninput: move |evt| temp_hive_key.set(evt.value()),
                                }
                            }
                        },
                        4 => rsx! {
                            div {
                                h3 {
                                    style: "margin: 0 0 15px 0; color: #ffffff;",
                                    "Configure Your OpenRouter API Key"
                                }
                                input {
                                    style: "width: 100%; padding: 8px 12px; background: #3c3c3c; border: 1px solid #3e3e42; border-radius: 4px; color: #ffffff;",
                                    r#type: "password",
                                    value: "{temp_openrouter_key}",
                                    placeholder: "sk-or-v1-...",
                                    oninput: move |evt| temp_openrouter_key.set(evt.value()),
                                }
                            }
                        },
                        5 => rsx! {
                            div {
                                h3 {
                                    style: "margin: 0 0 15px 0; color: #ffffff;",
                                    "Choose a Profile"
                                }
                                p {
                                    style: "color: #cccccc;",
                                    "You can configure profiles later in settings."
                                }
                            }
                        },
                        6 => rsx! {
                            div {
                                h3 {
                                    style: "margin: 0 0 15px 0; color: #ffffff;",
                                    "🎉 You're all set!"
                                }
                                p {
                                    style: "color: #cccccc;",
                                    "Click 'Get Started' to begin using Hive Consensus with Claude Code integration."
                                }
                            }
                        },
                        _ => rsx! {
                            div {
                                p { "Loading..." }
                            }
                        }
                    }
                }

                // Footer
                div {
                    style: "display: flex; justify-content: space-between; gap: 10px; padding: 20px; border-top: 1px solid #3e3e42; background: #252526;",

                    // Back button
                    if current_step() > 1 {
                        button {
                            style: "padding: 8px 16px; background: #3e3e42; color: #cccccc; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| {
                                let step = current_step();
                                if step > 1 {
                                    current_step.set(step - 1);
                                }
                            },
                            "Back"
                        }
                    } else {
                        div {} // Empty spacer
                    }

                    // Skip button for step 3 (Hive key)
                    if current_step() == 3 {
                        button {
                            style: "padding: 8px 16px; background: #3e3e42; color: #cccccc; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| current_step.set(4),
                            "Skip"
                        }
                    }

                    // Next/Get Started button
                    button {
                        style: "padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let step = current_step();

                            match step {
                                1..=5 => {
                                    // Save data if needed at each step
                                    if step == 3 && !temp_hive_key().is_empty() {
                                        hive_key.set(temp_hive_key());
                                    } else if step == 4 && !temp_openrouter_key().is_empty() {
                                        openrouter_key.set(temp_openrouter_key());
                                    }

                                    current_step.set(step + 1);
                                },
                                6 => {
                                    // Final step - close the dialog
                                    tracing::info!("Get Started clicked - CLOSING DIALOG");

                                    // Save completion status
                                    spawn(async move {
                                        if let Err(e) = crate::desktop::simple_db::mark_onboarding_complete() {
                                            tracing::error!("Failed to mark onboarding complete: {}", e);
                                        }
                                    });

                                    // CLOSE THE DIALOG
                                    close();
                                },
                                _ => {}
                            }
                        },
                        if current_step() < 5 { "Next" } else { "Get Started" }
                    }
                }
            }
        }
    }
}