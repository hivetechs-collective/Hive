//! New simplified onboarding dialog that actually closes

use dioxus::prelude::*;

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
        return None;
    }
    
    tracing::info!("OnboardingDialog: Rendering (visible=true)");
    
    // Local state
    let mut temp_openrouter_key = use_signal(|| String::new());
    let mut temp_hive_key = use_signal(|| String::new());
    
    // Simple close function
    let close = move || {
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
                        3 => rsx! {
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
                        4 => rsx! {
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
                        5 => rsx! {
                            div {
                                h3 { 
                                    style: "margin: 0 0 15px 0; color: #ffffff;",
                                    "🎉 You're all set!" 
                                }
                                p { 
                                    style: "color: #cccccc;",
                                    "Click 'Get Started' to begin using Hive Consensus." 
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
                    
                    // Skip button for step 2
                    if current_step() == 2 {
                        button {
                            style: "padding: 8px 16px; background: #3e3e42; color: #cccccc; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| current_step.set(3),
                            "Skip"
                        }
                    }
                    
                    // Next/Get Started button
                    button {
                        style: "padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let step = current_step();
                            
                            match step {
                                1..=4 => {
                                    // Save data if needed at each step
                                    if step == 2 && !temp_hive_key().is_empty() {
                                        hive_key.set(temp_hive_key());
                                    } else if step == 3 && !temp_openrouter_key().is_empty() {
                                        openrouter_key.set(temp_openrouter_key());
                                    }
                                    
                                    current_step.set(step + 1);
                                },
                                5 => {
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