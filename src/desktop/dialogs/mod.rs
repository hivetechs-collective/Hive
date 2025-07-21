//! Desktop UI Dialog Components

pub mod operation_confirmation;

pub use operation_confirmation::OperationConfirmationDialog;

// Create stub components for missing dialogs until they are properly implemented
use dioxus::prelude::*;
use dioxus::events::Key;

// Add the OnboardingDialog
#[component]
pub fn OnboardingDialog(
    show_onboarding: Signal<bool>,
    openrouter_key: Signal<String>,
    hive_key: Signal<String>,
    current_step: Signal<i32>,
    api_keys_version: Signal<u32>,
    on_profile_change: Option<EventHandler<()>>,
) -> Element {
    if !*show_onboarding.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| *show_onboarding.write() = false,
            
            div {
                style: "background: #2d2d30; padding: 40px; border-radius: 8px; width: 600px;",
                onclick: move |e| e.stop_propagation(),
                
                h2 { style: "margin: 0 0 20px 0;", "Welcome to Hive Consensus" }
                p { "Set up your API keys to get started." }
                
                div {
                    style: "margin: 20px 0;",
                    label { style: "display: block; margin-bottom: 5px;", "OpenRouter API Key:" }
                    input {
                        style: "width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: white;",
                        value: "{openrouter_key.read()}",
                        oninput: move |e| *openrouter_key.write() = e.value(),
                        r#type: "password"
                    }
                }
                
                div {
                    style: "margin: 20px 0;",
                    label { style: "display: block; margin-bottom: 5px;", "Hive API Key (optional):" }
                    input {
                        style: "width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: white;",
                        value: "{hive_key.read()}",
                        oninput: move |e| *hive_key.write() = e.value(),
                        r#type: "password"
                    }
                }
                
                div {
                    style: "display: flex; gap: 10px; justify-content: flex-end;",
                    
                    button {
                        style: "padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            *show_onboarding.write() = false;
                            // Increment API keys version to trigger reload
                            *api_keys_version.write() += 1;
                            if let Some(handler) = &on_profile_change {
                                handler.call(());
                            }
                        },
                        "Continue"
                    }
                }
            }
        }
    }
}

// Define types that were in dialogs_backup
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileInfo {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WelcomeAction {
    OpenFolder,
    OpenRecent,
    NewFile,
}

#[component]
pub fn AboutDialog(show_about: Signal<bool>) -> Element {
    if !*show_about.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| *show_about.write() = false,
            
            div {
                style: "background: #2d2d30; padding: 30px; border-radius: 8px; max-width: 500px;",
                onclick: move |e| e.stop_propagation(),
                
                h2 { style: "margin: 0 0 20px 0;", "About Hive Consensus" }
                p { "Version: 2.0.0" }
                p { "AI-powered development assistant" }
                
                button {
                    style: "margin-top: 20px; padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: move |_| *show_about.write() = false,
                    "Close"
                }
            }
        }
    }
}

#[component]
pub fn CommandPalette(show_palette: Signal<bool>) -> Element {
    if !*show_palette.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 100px; left: 50%; transform: translateX(-50%); background: #2d2d30; border: 1px solid #3e3e42; border-radius: 8px; padding: 10px; width: 600px; z-index: 1000;",
            
            input {
                style: "width: 100%; padding: 10px; background: #1e1e1e; border: 1px solid #3e3e42; color: white; font-size: 14px;",
                placeholder: "Type a command...",
                onkeydown: move |e| {
                    if e.key() == Key::Escape {
                        *show_palette.write() = false;
                    }
                }
            }
        }
    }
}

#[component]
pub fn SettingsDialog(
    show_settings: Signal<bool>,
    openrouter_key: Signal<String>,
    hive_key: Signal<String>,
    on_profile_change: Option<EventHandler<()>>,
) -> Element {
    if !*show_settings.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| *show_settings.write() = false,
            
            div {
                style: "background: #2d2d30; padding: 30px; border-radius: 8px; max-width: 600px; width: 100%;",
                onclick: move |e| e.stop_propagation(),
                
                h2 { style: "margin: 0 0 20px 0;", "Settings" }
                
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 5px;", "OpenRouter API Key:" }
                    input {
                        style: "width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: white;",
                        value: "{openrouter_key.read()}",
                        oninput: move |e| *openrouter_key.write() = e.value(),
                        r#type: "password"
                    }
                }
                
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 5px;", "Hive API Key:" }
                    input {
                        style: "width: 100%; padding: 8px; background: #1e1e1e; border: 1px solid #3e3e42; color: white;",
                        value: "{hive_key.read()}",
                        oninput: move |e| *hive_key.write() = e.value(),
                        r#type: "password"
                    }
                }
                
                div {
                    style: "display: flex; gap: 10px; justify-content: flex-end;",
                    
                    button {
                        style: "padding: 8px 16px; background: #3e3e42; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| *show_settings.write() = false,
                        "Cancel"
                    }
                    
                    button {
                        style: "padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            *show_settings.write() = false;
                            if let Some(handler) = &on_profile_change {
                                handler.call(());
                            }
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
pub fn UpdateAvailableDialog(
    show: Signal<bool>,
    version: String,
    date: String,
    download_url: String,
) -> Element {
    if !*show.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| *show.write() = false,
            
            div {
                style: "background: #2d2d30; padding: 30px; border-radius: 8px; max-width: 500px;",
                onclick: move |e| e.stop_propagation(),
                
                h2 { style: "margin: 0 0 20px 0;", "Update Available" }
                p { "Version {version} is available (released {date})" }
                
                button {
                    style: "margin-top: 20px; padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: move |_| *show.write() = false,
                    "Download"
                }
            }
        }
    }
}

#[component]
pub fn NoUpdatesDialog(show: Signal<bool>) -> Element {
    if !*show.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| *show.write() = false,
            
            div {
                style: "background: #2d2d30; padding: 30px; border-radius: 8px; max-width: 500px;",
                onclick: move |e| e.stop_propagation(),
                
                h2 { style: "margin: 0 0 20px 0;", "No Updates Available" }
                p { "You're running the latest version." }
                
                button {
                    style: "margin-top: 20px; padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: move |_| *show.write() = false,
                    "OK"
                }
            }
        }
    }
}

#[component]
pub fn UpdateErrorDialog(show: Signal<bool>, error_message: String) -> Element {
    if !*show.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| *show.write() = false,
            
            div {
                style: "background: #2d2d30; padding: 30px; border-radius: 8px; max-width: 500px;",
                onclick: move |e| e.stop_propagation(),
                
                h2 { style: "margin: 0 0 20px 0; color: #f44336;", "Update Error" }
                p { "{error_message}" }
                
                button {
                    style: "margin-top: 20px; padding: 8px 16px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: move |_| *show.write() = false,
                    "Close"
                }
            }
        }
    }
}

#[component]
pub fn UpgradeDialog(show: Signal<bool>) -> Element {
    if !*show.read() {
        return rsx! {};
    }
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| *show.write() = false,
            
            div {
                style: "background: #2d2d30; padding: 30px; border-radius: 8px; max-width: 500px;",
                onclick: move |e| e.stop_propagation(),
                
                h2 { style: "margin: 0 0 20px 0;", "Upgrade to Pro" }
                p { "Unlock advanced features with Hive Pro!" }
                
                button {
                    style: "margin-top: 20px; padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: move |_| *show.write() = false,
                    "Learn More"
                }
            }
        }
    }
}

#[component]
pub fn WelcomeTab(
    on_action: EventHandler<WelcomeAction>,
) -> Element {
    rsx! {
        div {
            style: "padding: 40px; max-width: 800px; margin: 0 auto;",
            
            h1 { style: "font-size: 32px; margin-bottom: 20px;", "Welcome to Hive Consensus" }
            
            p {
                style: "font-size: 16px; color: #cccccc; margin-bottom: 40px;",
                "AI-powered development assistant at your fingertips."
            }
            
            div {
                style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px;",
                
                button {
                    style: "padding: 20px; background: #2d2d30; border: 1px solid #3e3e42; border-radius: 8px; cursor: pointer; text-align: center;",
                    onclick: move |_| on_action.call(WelcomeAction::OpenFolder),
                    
                    div { style: "font-size: 24px; margin-bottom: 10px;", "📁" }
                    div { style: "font-weight: bold;", "Open Folder" }
                    div { style: "font-size: 12px; color: #858585;", "Browse and open a project folder" }
                }
                
                button {
                    style: "padding: 20px; background: #2d2d30; border: 1px solid #3e3e42; border-radius: 8px; cursor: pointer; text-align: center;",
                    onclick: move |_| on_action.call(WelcomeAction::NewFile),
                    
                    div { style: "font-size: 24px; margin-bottom: 10px;", "📄" }
                    div { style: "font-weight: bold;", "New File" }
                    div { style: "font-size: 12px; color: #858585;", "Create a new file" }
                }
                
                button {
                    style: "padding: 20px; background: #2d2d30; border: 1px solid #3e3e42; border-radius: 8px; cursor: pointer; text-align: center;",
                    onclick: move |_| on_action.call(WelcomeAction::OpenRecent),
                    
                    div { style: "font-size: 24px; margin-bottom: 10px;", "🕐" }
                    div { style: "font-weight: bold;", "Recent Files" }
                    div { style: "font-size: 12px; color: #858585;", "Open a recently used file" }
                }
            }
        }
    }
}

// Function to load existing profiles from database
pub async fn load_existing_profiles() -> Result<Vec<ProfileInfo>, anyhow::Error> {
    use crate::core::database::get_database;
    
    let db = get_database().await?;
    let conn = db.get_connection()?;
    
    let profiles = tokio::task::spawn_blocking(move || -> Result<Vec<ProfileInfo>, anyhow::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, profile_name, is_default, created_at FROM consensus_profiles ORDER BY created_at DESC"
        )?;
        
        let profiles = stmt.query_map([], |row| {
            Ok(ProfileInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                is_default: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(profiles)
    }).await??;
    
    Ok(profiles)
}

// Dialog styles
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

.dialog {
    background: #2d2d30;
    border: 1px solid #3e3e42;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    max-width: 90vw;
    max-height: 90vh;
    overflow: auto;
}
"#;