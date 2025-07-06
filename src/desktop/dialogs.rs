//! Dialog components for the desktop application

use dioxus::prelude::*;
use anyhow;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WelcomeAction {
    OpenFolder,
    OpenRecent,
    NewFile,
}

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
pub fn WelcomeTab(show_welcome: Signal<bool>, on_action: EventHandler<WelcomeAction>) -> Element {
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
                        onclick: move |_| on_action.call(WelcomeAction::OpenFolder),
                        "Open Folder"
                    }
                    button { 
                        class: "welcome-button",
                        onclick: move |_| on_action.call(WelcomeAction::OpenRecent),
                        "Open Recent"
                    }
                    button { 
                        class: "welcome-button",
                        onclick: move |_| on_action.call(WelcomeAction::NewFile),
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

/// Settings Dialog Component
#[component]
pub fn SettingsDialog(show_settings: Signal<bool>, openrouter_key: Signal<String>, hive_key: Signal<String>) -> Element {
    let mut is_validating = use_signal(|| false);
    let mut validation_error = use_signal(|| None::<String>);
    
    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| *show_settings.write() = false,
            
            div {
                class: "dialog settings-dialog",
                style: "width: 600px; max-height: 80vh; overflow-y: auto;",
                onclick: move |evt| evt.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    h2 { "⚙️ Settings" }
                    button {
                        class: "dialog-close",
                        onclick: move |_| *show_settings.write() = false,
                        "×"
                    }
                }
                
                div {
                    class: "dialog-content settings-content",
                    
                    // API Keys Section
                    div {
                        class: "settings-section",
                        h3 { "🔑 API Keys" }
                        p { 
                            class: "settings-description",
                            "Configure your API keys to enable Hive Consensus functionality." 
                        }
                        
                        // OpenRouter API Key
                        div {
                            class: "settings-field",
                            label { 
                                class: "settings-label",
                                "OpenRouter API Key" 
                            }
                            input {
                                class: "settings-input",
                                r#type: "password",
                                value: "{openrouter_key.read()}",
                                placeholder: "sk-or-v1-...",
                                oninput: move |evt| *openrouter_key.write() = evt.value().clone(),
                            }
                            p { 
                                class: "settings-hint",
                                "Get your API key from ",
                                a {
                                    href: "#",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        spawn(async {
                                            let _ = webbrowser::open("https://openrouter.ai/keys");
                                        });
                                    },
                                    "openrouter.ai/keys"
                                }
                            }
                        }
                        
                        // Hive API Key
                        div {
                            class: "settings-field",
                            label { 
                                class: "settings-label",
                                "Hive API Key (Optional)" 
                            }
                            input {
                                class: "settings-input",
                                r#type: "password",
                                value: "{hive_key.read()}",
                                placeholder: "hive-...",
                                oninput: move |evt| *hive_key.write() = evt.value().clone(),
                            }
                            p { 
                                class: "settings-hint",
                                "Used for syncing conversations and advanced features" 
                            }
                        }
                        
                        // Show validation error if any
                        if let Some(error) = validation_error.read().as_ref() {
                            div {
                                style: "margin-top: 10px; padding: 10px; background: #5a1e1e; border: 1px solid #8b3a3a; border-radius: 4px; color: #ff6b6b;",
                                "❌ {error}"
                            }
                        }
                    }
                    
                    // Consensus Profile Section
                    div {
                        class: "settings-section",
                        h3 { "🧠 Consensus Profile" }
                        p { 
                            class: "settings-description",
                            "Choose your consensus processing profile based on your needs." 
                        }
                        
                        div {
                            class: "profile-grid",
                            ProfileOption { 
                                name: "Balanced",
                                description: "Best overall performance and quality",
                                models: "Claude 3.5 Sonnet, GPT-4 Turbo, Claude 3 Opus, GPT-4o",
                                is_selected: true,
                            }
                            ProfileOption { 
                                name: "Speed",
                                description: "Faster responses with good quality",
                                models: "Claude 3 Haiku, GPT-3.5 Turbo",
                                is_selected: false,
                            }
                            ProfileOption { 
                                name: "Quality",
                                description: "Highest quality responses",
                                models: "Claude 3 Opus, GPT-4o",
                                is_selected: false,
                            }
                            ProfileOption { 
                                name: "Cost",
                                description: "Most cost-effective option",
                                models: "Llama 3.2, Mistral 7B",
                                is_selected: false,
                            }
                        }
                    }
                }
                
                div {
                    class: "dialog-footer",
                    button {
                        class: "button button-secondary",
                        onclick: move |_| *show_settings.write() = false,
                        "Cancel"
                    }
                    button {
                        class: "button button-primary",
                        disabled: *is_validating.read(),
                        onclick: move |_| {
                            // Clear previous error
                            *validation_error.write() = None;
                            *is_validating.write() = true;
                            
                            // Save settings with validation
                            let openrouter = openrouter_key.read().clone();
                            let hive = hive_key.read().clone();
                            let mut is_validating = is_validating.clone();
                            let mut validation_error = validation_error.clone();
                            let mut show_settings = show_settings.clone();
                            
                            spawn(async move {
                                match save_api_keys(&openrouter, &hive).await {
                                    Ok(_) => {
                                        // Success - close dialog
                                        *show_settings.write() = false;
                                    }
                                    Err(e) => {
                                        // Show error
                                        *validation_error.write() = Some(e.to_string());
                                    }
                                }
                                *is_validating.write() = false;
                            });
                        },
                        if *is_validating.read() { "Validating..." } else { "Save Settings" }
                    }
                }
            }
        }
    }
}

#[component]
fn ProfileOption(name: &'static str, description: &'static str, models: &'static str, is_selected: bool) -> Element {
    rsx! {
        div {
            class: if is_selected { "profile-option selected" } else { "profile-option" },
            h4 { "{name}" }
            p { class: "profile-description", "{description}" }
            p { class: "profile-models", "{models}" }
        }
    }
}

/// Onboarding Dialog for first-time users
#[component]
pub fn OnboardingDialog(show_onboarding: Signal<bool>, openrouter_key: Signal<String>) -> Element {
    let mut current_step = use_signal(|| 1);
    
    rsx! {
        div {
            class: "dialog-overlay",
            
            div {
                class: "dialog onboarding-dialog",
                style: "width: 600px;",
                onclick: move |evt| evt.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    h2 { "🐝 Welcome to Hive Consensus" }
                }
                
                div {
                    class: "dialog-content onboarding-content",
                    
                    if *current_step.read() == 1 {
                        div {
                            class: "onboarding-step",
                            h3 { "Let's get you started!" }
                            p { 
                                "Hive Consensus uses advanced AI models to provide the best possible responses through a 4-stage consensus process." 
                            }
                            
                            div {
                                class: "features-list",
                                div { class: "feature-item", "✅ 4-stage consensus pipeline" }
                                div { class: "feature-item", "✅ 323+ AI models available" }
                                div { class: "feature-item", "✅ VS Code-like interface" }
                                div { class: "feature-item", "✅ Full file system access" }
                            }
                        }
                    } else if *current_step.read() == 2 {
                        div {
                            class: "onboarding-step",
                            h3 { "Configure Your API Key" }
                            p { 
                                "To use Hive Consensus, you'll need an OpenRouter API key. This gives you access to 323+ models from various providers." 
                            }
                            
                            div {
                                class: "settings-field",
                                label { 
                                    class: "settings-label",
                                    "OpenRouter API Key" 
                                }
                                input {
                                    class: "settings-input",
                                    r#type: "password",
                                    value: "{openrouter_key.read()}",
                                    placeholder: "sk-or-v1-...",
                                    oninput: move |evt| *openrouter_key.write() = evt.value().clone(),
                                }
                                
                                div {
                                    class: "api-key-help",
                                    p { 
                                        "Don't have an API key? ",
                                        a {
                                            href: "#",
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                spawn(async {
                                                    let _ = webbrowser::open("https://openrouter.ai/keys");
                                                });
                                            },
                                            "Get one from OpenRouter"
                                        }
                                    }
                                    p { 
                                        class: "settings-hint",
                                        "OpenRouter provides unified access to models from OpenAI, Anthropic, Google, Meta, and more." 
                                    }
                                }
                            }
                        }
                    } else {
                        div {
                            class: "onboarding-step",
                            h3 { "You're all set!" }
                            p { 
                                "Hive Consensus is ready to use. Here are some things you can try:" 
                            }
                            
                            div {
                                class: "suggestions-list",
                                div { 
                                    class: "suggestion-item",
                                    "💡 Ask about your code: \"What does this function do?\"" 
                                }
                                div { 
                                    class: "suggestion-item",
                                    "🔧 Request help: \"How can I optimize this algorithm?\"" 
                                }
                                div { 
                                    class: "suggestion-item",
                                    "📚 Learn concepts: \"Explain async/await in Rust\"" 
                                }
                                div { 
                                    class: "suggestion-item",
                                    "🐛 Debug issues: \"Why is this code not working?\"" 
                                }
                            }
                        }
                    }
                }
                
                div {
                    class: "dialog-footer onboarding-footer",
                    if *current_step.read() > 1 {
                        button {
                            class: "button button-secondary",
                            onclick: move |_| {
                                let step = *current_step.read();
                                if step > 1 {
                                    *current_step.write() = step - 1;
                                }
                            },
                            "Back"
                        }
                    }
                    
                    button {
                        class: "button button-primary",
                        disabled: if *current_step.read() == 2 && openrouter_key.read().is_empty() { true } else { false },
                        onclick: move |_| {
                            let step = *current_step.read();
                            if step < 3 {
                                // Don't allow moving past step 2 without a key
                                if step == 2 && openrouter_key.read().is_empty() {
                                    return;
                                }
                                
                                // If on step 2, validate the key
                                if step == 2 {
                                    let key = openrouter_key.read().clone();
                                    let mut current_step = current_step.clone();
                                    let mut show_onboarding = show_onboarding.clone();
                                    
                                    spawn(async move {
                                        match save_api_keys(&key, "").await {
                                            Ok(_) => {
                                                // Move to success step
                                                *current_step.write() = 3;
                                            }
                                            Err(e) => {
                                                // Show error in console for now
                                                eprintln!("Failed to validate API key: {}", e);
                                            }
                                        }
                                    });
                                } else {
                                    *current_step.write() = step + 1;
                                }
                            } else {
                                // Close on final step
                                *show_onboarding.write() = false;
                            }
                        },
                        if *current_step.read() < 3 { "Next" } else { "Get Started" }
                    }
                }
            }
        }
    }
}

/// Save API keys with validation and database storage
async fn save_api_keys(openrouter_key: &str, hive_key: &str) -> anyhow::Result<()> {
    use crate::core::api_keys::ApiKeyManager;
    
    // Validate OpenRouter key format first
    if !openrouter_key.is_empty() {
        ApiKeyManager::validate_format(openrouter_key)?;
        
        // Test with live API call
        match ApiKeyManager::test_openrouter_key(openrouter_key).await {
            Ok(true) => {
                // Key is valid, save to database
                ApiKeyManager::save_to_database(Some(openrouter_key), Some(hive_key)).await?;
            }
            Ok(false) => {
                return Err(anyhow::anyhow!("API key validation failed"));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to validate API key: {}", e));
            }
        }
    } else {
        return Err(anyhow::anyhow!("OpenRouter API key is required"));
    }
    
    Ok(())
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
    
    /* Settings dialog styles */
    .settings-dialog {
        width: 600px;
    }
    
    .settings-content {
        padding: 0;
        text-align: left;
    }
    
    .settings-section {
        padding: 20px 30px;
        border-bottom: 1px solid #3e3e42;
    }
    
    .settings-section:last-child {
        border-bottom: none;
    }
    
    .settings-section h3 {
        margin: 0 0 10px 0;
        color: #ffffff;
        font-size: 16px;
    }
    
    .settings-description {
        margin: 0 0 20px 0;
        color: #858585;
        font-size: 13px;
    }
    
    .settings-field {
        margin-bottom: 20px;
    }
    
    .settings-label {
        display: block;
        margin-bottom: 8px;
        color: #cccccc;
        font-size: 14px;
    }
    
    .settings-input {
        width: 100%;
        padding: 8px 12px;
        background: #3c3c3c;
        border: 1px solid #3e3e42;
        border-radius: 4px;
        color: #ffffff;
        font-size: 14px;
    }
    
    .settings-input:focus {
        outline: none;
        border-color: #007acc;
    }
    
    .settings-hint {
        margin: 5px 0 0 0;
        font-size: 12px;
        color: #858585;
    }
    
    .settings-hint a {
        color: #007acc;
        text-decoration: none;
    }
    
    .settings-hint a:hover {
        text-decoration: underline;
    }
    
    .profile-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 15px;
    }
    
    .profile-option {
        padding: 15px;
        background: #1e1e1e;
        border: 1px solid #3e3e42;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.2s;
    }
    
    .profile-option:hover {
        border-color: #007acc;
    }
    
    .profile-option.selected {
        border-color: #007acc;
        background: #094771;
    }
    
    .profile-option h4 {
        margin: 0 0 5px 0;
        color: #ffffff;
        font-size: 14px;
    }
    
    .profile-description {
        margin: 0 0 8px 0;
        color: #cccccc;
        font-size: 12px;
    }
    
    .profile-models {
        margin: 0;
        color: #858585;
        font-size: 11px;
    }
    
    /* Dialog footer */
    .dialog-footer {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        padding: 20px;
        border-top: 1px solid #3e3e42;
        background: #252526;
    }
    
    .button {
        padding: 8px 16px;
        border: none;
        border-radius: 4px;
        font-size: 14px;
        cursor: pointer;
        transition: background 0.2s;
    }
    
    .button-primary {
        background: #007acc;
        color: white;
    }
    
    .button-primary:hover {
        background: #005a9e;
    }
    
    .button-secondary {
        background: #3e3e42;
        color: #cccccc;
    }
    
    .button-secondary:hover {
        background: #4e4e52;
    }
    
    /* Onboarding dialog styles */
    .onboarding-dialog {
        width: 600px;
    }
    
    .onboarding-content {
        padding: 30px;
        text-align: left;
    }
    
    .onboarding-step h3 {
        margin: 0 0 15px 0;
        color: #ffffff;
        font-size: 20px;
    }
    
    .onboarding-step p {
        margin: 0 0 20px 0;
        color: #cccccc;
        line-height: 1.5;
    }
    
    .features-list {
        margin: 20px 0;
    }
    
    .feature-item {
        padding: 8px 0;
        color: #cccccc;
        font-size: 14px;
    }
    
    .api-key-help {
        margin-top: 15px;
        padding: 15px;
        background: #1e1e1e;
        border-radius: 6px;
        border: 1px solid #3e3e42;
    }
    
    .api-key-help p {
        margin: 0 0 10px 0;
    }
    
    .api-key-help p:last-child {
        margin: 0;
    }
    
    .suggestions-list {
        margin-top: 20px;
    }
    
    .suggestion-item {
        padding: 10px 15px;
        margin: 8px 0;
        background: #1e1e1e;
        border: 1px solid #3e3e42;
        border-radius: 6px;
        font-size: 14px;
        color: #cccccc;
    }
    
    .onboarding-footer {
        justify-content: space-between;
    }
    
    .dialog {
        background: #2d2d30;
        border: 1px solid #3e3e42;
        border-radius: 8px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        overflow: hidden;
    }
"#;