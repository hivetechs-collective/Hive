use dioxus::prelude::*;

pub const DIALOG_STYLES: &str = r#"
    /* Dialog overlay styles */
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
        z-index: 2000;
    }
    
    .dialog {
        background: #252526;
        border: 1px solid #3e3e42;
        border-radius: 8px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
        min-width: 400px;
        max-width: 600px;
        animation: dialogFadeIn 0.2s ease-out;
    }
    
    @keyframes dialogFadeIn {
        from {
            opacity: 0;
            transform: scale(0.95);
        }
        to {
            opacity: 1;
            transform: scale(1);
        }
    }
    
    .dialog-header {
        padding: 16px 20px;
        border-bottom: 1px solid #3e3e42;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    
    .dialog-title {
        font-size: 16px;
        font-weight: 600;
        color: #cccccc;
        margin: 0;
    }
    
    .dialog-close {
        background: none;
        border: none;
        color: #858585;
        font-size: 20px;
        cursor: pointer;
        padding: 0;
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
        transition: background-color 0.1s;
    }
    
    .dialog-close:hover {
        background: #3e3e42;
        color: #cccccc;
    }
    
    .dialog-body {
        padding: 20px;
        color: #cccccc;
        line-height: 1.6;
    }
    
    .dialog-footer {
        padding: 16px 20px;
        border-top: 1px solid #3e3e42;
        display: flex;
        justify-content: flex-end;
        gap: 8px;
    }
    
    .dialog-button {
        padding: 6px 14px;
        border-radius: 4px;
        border: 1px solid transparent;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.1s;
    }
    
    .dialog-button.primary {
        background: #007acc;
        color: white;
        border-color: #007acc;
    }
    
    .dialog-button.primary:hover {
        background: #005a9e;
        border-color: #005a9e;
    }
    
    .dialog-button.secondary {
        background: #3e3e42;
        color: #cccccc;
        border-color: #3e3e42;
    }
    
    .dialog-button.secondary:hover {
        background: #4e4e52;
        border-color: #4e4e52;
    }
    
    /* About dialog specific styles */
    .about-content {
        text-align: center;
    }
    
    .about-logo {
        font-size: 48px;
        margin-bottom: 16px;
    }
    
    .about-title {
        font-size: 24px;
        font-weight: 600;
        color: #e1e1e1;
        margin: 0 0 8px 0;
    }
    
    .about-version {
        font-size: 14px;
        color: #858585;
        margin-bottom: 20px;
    }
    
    .about-description {
        font-size: 14px;
        margin-bottom: 20px;
        color: #cccccc;
    }
    
    .about-features {
        text-align: left;
        background: #1e1e1e;
        border-radius: 4px;
        padding: 16px;
        margin: 16px 0;
    }
    
    .about-features h4 {
        margin: 0 0 8px 0;
        color: #e1e1e1;
        font-size: 14px;
    }
    
    .about-features ul {
        margin: 0;
        padding-left: 20px;
        color: #cccccc;
        font-size: 13px;
    }
    
    .about-features li {
        margin: 4px 0;
    }
    
    .about-copyright {
        font-size: 12px;
        color: #858585;
        margin-top: 20px;
    }
"#;

#[component]
pub fn AboutDialog(on_close: EventHandler<()>) -> Element {
    rsx! {
        style { "{DIALOG_STYLES}" }
        
        div {
            class: "dialog-overlay",
            onclick: move |_| on_close.call(()),
            
            div {
                class: "dialog",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    h3 { class: "dialog-title", "About HiveTechs Consensus" }
                    button {
                        class: "dialog-close",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                
                div {
                    class: "dialog-body",
                    div {
                        class: "about-content",
                        div { class: "about-logo", "🐝" }
                        h2 { class: "about-title", "HiveTechs Consensus IDE" }
                        div { class: "about-version", "Version {env!(\"CARGO_PKG_VERSION\")}" }
                        
                        div {
                            class: "about-description",
                            "The most advanced AI-powered development assistant with "
                            "4-stage consensus pipeline and enterprise features."
                        }
                        
                        div {
                            class: "about-features",
                            h4 { "Key Features:" }
                            ul {
                                li { "🧠 4-Stage AI Consensus Pipeline" }
                                li { "⚡ 10-40x Performance vs TypeScript" }
                                li { "🔍 ML-Powered Repository Intelligence" }
                                li { "📊 Enterprise Analytics & Reporting" }
                                li { "🎯 Deterministic AI Control" }
                                li { "🌐 323+ AI Models via OpenRouter" }
                            }
                        }
                        
                        div { class: "about-copyright", "© 2024 HiveTechs. All rights reserved." }
                    }
                }
                
                div {
                    class: "dialog-footer",
                    button {
                        class: "dialog-button primary",
                        onclick: move |_| on_close.call(()),
                        "OK"
                    }
                }
            }
        }
    }
}

#[component]
pub fn WelcomeDialog(on_close: EventHandler<()>) -> Element {
    rsx! {
        style { "{DIALOG_STYLES}" }
        
        div {
            class: "dialog-overlay",
            onclick: move |_| on_close.call(()),
            
            div {
                class: "dialog",
                style: "max-width: 700px;",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    h3 { class: "dialog-title", "Welcome to HiveTechs Consensus" }
                    button {
                        class: "dialog-close",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                
                div {
                    class: "dialog-body",
                    h3 { style: "margin-top: 0;", "Get Started" }
                    
                    div { style: "margin-bottom: 16px;",
                        "👋 Welcome! Here's how to make the most of HiveTechs Consensus:"
                    }
                    
                    div { style: "margin-bottom: 12px;",
                        strong { "Open a Project:" }
                        ul { style: "margin: 4px 0;",
                            li { "Use File → Open Folder (Cmd+K Cmd+O) to open a project directory" }
                            li { "Or File → Open File (Cmd+O) to open individual files" }
                        }
                    }
                    
                    div { style: "margin-bottom: 12px;",
                        strong { "Use AI Assistance:" }
                        ul { style: "margin: 4px 0;",
                            li { "Type questions in the chat panel to get AI-powered help" }
                            li { "The 4-stage consensus ensures high-quality responses" }
                            li { "Use commands like 'analyze', 'plan', and 'ask' for specific tasks" }
                        }
                    }
                    
                    div { style: "margin-bottom: 12px;",
                        strong { "Keyboard Shortcuts:" }
                        ul { style: "margin: 4px 0;",
                            li { "Cmd+Shift+P - Open command palette" }
                            li { "Cmd+P - Quick file search" }
                            li { "Cmd+S - Save current file" }
                            li { "Cmd+W - Close current tab" }
                        }
                    }
                    
                    div { style: "margin-bottom: 12px;",
                        strong { "Explore Features:" }
                        ul { style: "margin: 4px 0;",
                            li { "File explorer on the left shows your project structure" }
                            li { "Code editor with syntax highlighting in the center" }
                            li { "AI chat panel on the right for assistance" }
                            li { "Status bar shows git info and file details" }
                        }
                    }
                }
                
                div {
                    class: "dialog-footer",
                    button {
                        class: "dialog-button primary",
                        onclick: move |_| on_close.call(()),
                        "Get Started"
                    }
                }
            }
        }
    }
}