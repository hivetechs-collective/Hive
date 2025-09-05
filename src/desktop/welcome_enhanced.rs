//! Enhanced VS Code-style Welcome/Getting Started experience
//! Based on VS Code's src/vs/workbench/contrib/welcomeViews/

use dioxus::prelude::*;

/// Welcome tab content
#[derive(Clone, Debug, PartialEq)]
pub enum WelcomeSection {
    Start,
    Recent,
    Walkthroughs,
    Help,
}

/// Welcome tab state
#[derive(Clone, Debug)]
pub struct WelcomeState {
    pub active_section: WelcomeSection,
    pub recent_projects: Vec<RecentProject>,
    pub walkthroughs_completed: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    pub last_opened: String,
}

impl Default for WelcomeState {
    fn default() -> Self {
        Self {
            active_section: WelcomeSection::Start,
            recent_projects: vec![
                RecentProject {
                    name: "hive-consensus".to_string(),
                    path: "~/Developer/Private/hive".to_string(),
                    last_opened: "2 hours ago".to_string(),
                },
                RecentProject {
                    name: "rust-analyzer".to_string(),
                    path: "~/Developer/rust-analyzer".to_string(),
                    last_opened: "Yesterday".to_string(),
                },
            ],
            walkthroughs_completed: vec![],
        }
    }
}

/// Enhanced Welcome/Getting Started component
#[component]
pub fn EnhancedWelcome(
    state: Signal<WelcomeState>,
    on_open_project: EventHandler<String>,
    on_new_file: EventHandler<()>,
    on_new_project: EventHandler<()>,
    on_open_folder: EventHandler<()>,
) -> Element {
    let welcome_state = state.read();
    
    rsx! {
        div {
            class: "welcome-container",
            
            // Welcome header with HiveTechs branding
            div {
                class: "welcome-header",
                
                div {
                    class: "welcome-branding",
                    
                    div {
                        class: "hivetechs-logo",
                        "🐝"
                    }
                    
                    div {
                        class: "welcome-title",
                        h1 { "HiveTechs Consensus" }
                        p { 
                            class: "welcome-subtitle",
                            "AI-Powered Development with 4-Stage Consensus Intelligence" 
                        }
                    }
                }
                
                div {
                    class: "welcome-version",
                    "v2.0.0"
                }
            }
            
            // Navigation tabs
            div {
                class: "welcome-nav",
                
                button {
                    class: if welcome_state.active_section == WelcomeSection::Start { "nav-button active" } else { "nav-button" },
                    onclick: move |_| {
                        state.write().active_section = WelcomeSection::Start;
                    },
                    "Start"
                }
                
                button {
                    class: if welcome_state.active_section == WelcomeSection::Recent { "nav-button active" } else { "nav-button" },
                    onclick: move |_| {
                        state.write().active_section = WelcomeSection::Recent;
                    },
                    "Recent"
                }
                
                button {
                    class: if welcome_state.active_section == WelcomeSection::Walkthroughs { "nav-button active" } else { "nav-button" },
                    onclick: move |_| {
                        state.write().active_section = WelcomeSection::Walkthroughs;
                    },
                    "Walkthroughs"
                }
                
                button {
                    class: if welcome_state.active_section == WelcomeSection::Help { "nav-button active" } else { "nav-button" },
                    onclick: move |_| {
                        state.write().active_section = WelcomeSection::Help;
                    },
                    "Help"
                }
            }
            
            // Content area
            div {
                class: "welcome-content",
                
                match welcome_state.active_section {
                    WelcomeSection::Start => rsx! {
                        StartSection {
                            on_new_file: on_new_file.clone(),
                            on_new_project: on_new_project.clone(),
                            on_open_folder: on_open_folder.clone(),
                        }
                    },
                    WelcomeSection::Recent => rsx! {
                        RecentSection {
                            projects: welcome_state.recent_projects.clone(),
                            on_open: on_open_project.clone(),
                        }
                    },
                    WelcomeSection::Walkthroughs => rsx! {
                        WalkthroughsSection {
                            completed: welcome_state.walkthroughs_completed.clone(),
                        }
                    },
                    WelcomeSection::Help => rsx! {
                        HelpSection {}
                    },
                }
            }
        }
    }
}

/// Start section with quick actions
#[component]
fn StartSection(
    on_new_file: EventHandler<()>,
    on_new_project: EventHandler<()>,
    on_open_folder: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "start-section",
            
            h2 { "Start" }
            
            div {
                class: "start-grid",
                
                button {
                    class: "start-card",
                    onclick: move |_| on_new_file.call(()),
                    
                    span { class: "codicon codicon-new-file" }
                    h3 { "New File" }
                    p { "Create a new file in the editor" }
                }
                
                button {
                    class: "start-card",
                    onclick: move |_| on_new_project.call(()),
                    
                    span { class: "codicon codicon-new-folder" }
                    h3 { "New Project" }
                    p { "Start a new Rust project with Hive" }
                }
                
                button {
                    class: "start-card",
                    onclick: move |_| on_open_folder.call(()),
                    
                    span { class: "codicon codicon-folder-opened" }
                    h3 { "Open Folder" }
                    p { "Open a project folder" }
                }
                
                button {
                    class: "start-card",
                    onclick: move |_| {},
                    
                    span { class: "codicon codicon-source-control" }
                    h3 { "Clone Repository" }
                    p { "Clone from GitHub, GitLab, etc." }
                }
            }
            
            div {
                class: "quick-setup",
                
                h3 { "Quick Setup" }
                
                div {
                    class: "setup-item",
                    span { class: "codicon codicon-key" }
                    span { "Configure API Keys for Consensus Engine" }
                    button { 
                        class: "setup-button",
                        "Configure" 
                    }
                }
                
                div {
                    class: "setup-item",
                    span { class: "codicon codicon-color-mode" }
                    span { "Choose your theme" }
                    button { 
                        class: "setup-button",
                        "Select" 
                    }
                }
                
                div {
                    class: "setup-item",
                    span { class: "codicon codicon-keyboard" }
                    span { "Configure keyboard shortcuts" }
                    button { 
                        class: "setup-button",
                        "Customize" 
                    }
                }
            }
        }
    }
}

/// Recent projects section
#[component]
fn RecentSection(
    projects: Vec<RecentProject>,
    on_open: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "recent-section",
            
            h2 { "Recent Projects" }
            
            if projects.is_empty() {
                div {
                    class: "no-recent",
                    "No recent projects"
                }
            } else {
                div {
                    class: "recent-list",
                    
                    for project in projects {
                        button {
                            class: "recent-item",
                            onclick: {
                                let path = project.path.clone();
                                move |_| on_open.call(path.clone())
                            },
                            
                            span { 
                                class: "codicon codicon-folder",
                                style: "font-size: 24px; color: var(--hivetechs-yellow);"
                            }
                            
                            div {
                                class: "recent-info",
                                
                                div { 
                                    class: "recent-name",
                                    "{project.name}" 
                                }
                                div { 
                                    class: "recent-path",
                                    "{project.path}" 
                                }
                                div { 
                                    class: "recent-time",
                                    "{project.last_opened}" 
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Walkthroughs section
#[component]
fn WalkthroughsSection(completed: Vec<String>) -> Element {
    rsx! {
        div {
            class: "walkthroughs-section",
            
            h2 { "Interactive Walkthroughs" }
            
            div {
                class: "walkthrough-list",
                
                WalkthroughCard {
                    id: "consensus-basics",
                    title: "Consensus Engine Basics",
                    description: "Learn how the 4-stage consensus engine works",
                    icon: "circuit-board",
                    steps: 5,
                    completed: completed.contains(&"consensus-basics".to_string()),
                }
                
                WalkthroughCard {
                    id: "first-query",
                    title: "Your First Query",
                    description: "Run your first AI-powered consensus query",
                    icon: "comment-discussion",
                    steps: 3,
                    completed: completed.contains(&"first-query".to_string()),
                }
                
                WalkthroughCard {
                    id: "profiles",
                    title: "Consensus Profiles",
                    description: "Customize AI behavior with profiles",
                    icon: "settings-gear",
                    steps: 4,
                    completed: completed.contains(&"profiles".to_string()),
                }
                
                WalkthroughCard {
                    id: "advanced",
                    title: "Advanced Features",
                    description: "Repository intelligence, planning, and more",
                    icon: "rocket",
                    steps: 6,
                    completed: completed.contains(&"advanced".to_string()),
                }
            }
        }
    }
}

/// Individual walkthrough card
#[component]
fn WalkthroughCard(
    id: String,
    title: String,
    description: String,
    icon: String,
    steps: u32,
    completed: bool,
) -> Element {
    rsx! {
        button {
            class: if completed { "walkthrough-card completed" } else { "walkthrough-card" },
            
            span { 
                class: "codicon codicon-{icon}",
                style: "font-size: 32px;"
            }
            
            div {
                class: "walkthrough-info",
                
                h3 { "{title}" }
                p { "{description}" }
                
                div {
                    class: "walkthrough-progress",
                    
                    if completed {
                        span { 
                            class: "codicon codicon-check",
                            style: "color: #4caf50;"
                        }
                        span { "Completed" }
                    } else {
                        span { "{steps} steps" }
                    }
                }
            }
        }
    }
}

/// Help section
#[component]
fn HelpSection() -> Element {
    rsx! {
        div {
            class: "help-section",
            
            h2 { "Help & Resources" }
            
            div {
                class: "help-grid",
                
                a {
                    class: "help-item",
                    href: "https://hivetechs.ai/docs",
                    target: "_blank",
                    
                    span { class: "codicon codicon-book" }
                    h3 { "Documentation" }
                    p { "Read the official docs" }
                }
                
                a {
                    class: "help-item",
                    href: "https://hivetechs.ai/support",
                    target: "_blank",
                    
                    span { class: "codicon codicon-comment-discussion" }
                    h3 { "Support" }
                    p { "Get help from the team" }
                }
                
                button {
                    class: "help-item",
                    
                    span { class: "codicon codicon-keyboard" }
                    h3 { "Keyboard Shortcuts" }
                    p { "View all shortcuts" }
                }
                
                button {
                    class: "help-item",
                    
                    span { class: "codicon codicon-info" }
                    h3 { "About" }
                    p { "Version and license info" }
                }
            }
            
            div {
                class: "help-tips",
                
                h3 { "Tips & Tricks" }
                
                ul {
                    li { "Press Ctrl+Shift+P to open the command palette" }
                    li { "Use Ctrl+` to toggle the integrated terminal" }
                    li { "Press F1 to access the consensus engine directly" }
                    li { "Ctrl+B toggles the sidebar visibility" }
                }
            }
        }
    }
}

/// Welcome CSS styles
pub const WELCOME_STYLES: &str = r#"
/* Welcome Container */
.welcome-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--vscode-editor-background);
    color: var(--vscode-editor-foreground);
    overflow-y: auto;
}

/* Header */
.welcome-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 24px 32px;
    border-bottom: 1px solid var(--vscode-widget-border);
}

.welcome-branding {
    display: flex;
    align-items: center;
    gap: 16px;
}

.hivetechs-logo {
    font-size: 48px;
    line-height: 1;
}

.welcome-title h1 {
    font-size: 24px;
    font-weight: 600;
    margin: 0;
    color: var(--hivetechs-yellow);
}

.welcome-subtitle {
    font-size: 14px;
    color: var(--vscode-descriptionForeground);
    margin: 4px 0 0 0;
}

.welcome-version {
    font-size: 12px;
    color: var(--vscode-descriptionForeground);
    opacity: 0.7;
}

/* Navigation */
.welcome-nav {
    display: flex;
    gap: 2px;
    padding: 0 32px;
    background-color: var(--vscode-tab-unfocusedActiveBackground);
    border-bottom: 1px solid var(--vscode-widget-border);
}

.nav-button {
    padding: 8px 16px;
    background: transparent;
    border: none;
    color: var(--vscode-tab-inactiveForeground);
    cursor: pointer;
    font-size: 13px;
    border-bottom: 2px solid transparent;
    transition: all 0.1s ease;
}

.nav-button:hover {
    color: var(--vscode-tab-activeForeground);
}

.nav-button.active {
    color: var(--vscode-tab-activeForeground);
    border-bottom-color: var(--hivetechs-yellow);
}

/* Content */
.welcome-content {
    flex: 1;
    padding: 32px;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
}

/* Start Section */
.start-section h2 {
    font-size: 20px;
    margin-bottom: 24px;
}

.start-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 32px;
}

.start-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 24px;
    background-color: var(--vscode-input-background);
    border: 1px solid var(--vscode-input-border);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
}

.start-card:hover {
    background-color: var(--vscode-list-hoverBackground);
    border-color: var(--hivetechs-yellow);
    transform: translateY(-2px);
}

.start-card .codicon {
    font-size: 32px;
    color: var(--hivetechs-yellow);
}

.start-card h3 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
}

.start-card p {
    font-size: 12px;
    color: var(--vscode-descriptionForeground);
    margin: 0;
}

/* Quick Setup */
.quick-setup {
    background-color: var(--vscode-sideBar-background);
    border-radius: 4px;
    padding: 20px;
}

.quick-setup h3 {
    font-size: 16px;
    margin-bottom: 16px;
}

.setup-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid var(--vscode-widget-border);
}

.setup-item:last-child {
    border-bottom: none;
}

.setup-item .codicon {
    font-size: 18px;
    color: var(--vscode-textLink-foreground);
}

.setup-button {
    margin-left: auto;
    padding: 4px 12px;
    background-color: var(--vscode-button-background);
    color: var(--vscode-button-foreground);
    border: none;
    border-radius: 3px;
    font-size: 12px;
    cursor: pointer;
}

.setup-button:hover {
    background-color: var(--vscode-button-hoverBackground);
}

/* Recent Section */
.recent-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.recent-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background-color: var(--vscode-list-inactiveSelectionBackground);
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.1s ease;
    text-align: left;
    width: 100%;
}

.recent-item:hover {
    background-color: var(--vscode-list-hoverBackground);
    border-color: var(--hivetechs-yellow);
}

.recent-info {
    flex: 1;
}

.recent-name {
    font-size: 14px;
    font-weight: 600;
}

.recent-path {
    font-size: 12px;
    color: var(--vscode-descriptionForeground);
    margin: 2px 0;
}

.recent-time {
    font-size: 11px;
    color: var(--vscode-descriptionForeground);
    opacity: 0.7;
}

/* Walkthroughs */
.walkthrough-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
}

.walkthrough-card {
    display: flex;
    gap: 16px;
    padding: 20px;
    background-color: var(--vscode-input-background);
    border: 1px solid var(--vscode-input-border);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
}

.walkthrough-card:hover {
    background-color: var(--vscode-list-hoverBackground);
    border-color: var(--hivetechs-yellow);
}

.walkthrough-card.completed {
    opacity: 0.7;
}

.walkthrough-info {
    flex: 1;
}

.walkthrough-info h3 {
    font-size: 14px;
    font-weight: 600;
    margin: 0 0 4px 0;
}

.walkthrough-info p {
    font-size: 12px;
    color: var(--vscode-descriptionForeground);
    margin: 0 0 8px 0;
}

.walkthrough-progress {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--vscode-descriptionForeground);
}

/* Help Section */
.help-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 32px;
}

.help-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 24px;
    background-color: var(--vscode-input-background);
    border: 1px solid var(--vscode-input-border);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
    text-decoration: none;
    color: inherit;
}

.help-item:hover {
    background-color: var(--vscode-list-hoverBackground);
    border-color: var(--hivetechs-yellow);
    transform: translateY(-2px);
}

.help-item .codicon {
    font-size: 32px;
    color: var(--vscode-textLink-foreground);
}

.help-item h3 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
}

.help-item p {
    font-size: 12px;
    color: var(--vscode-descriptionForeground);
    margin: 0;
}

.help-tips {
    background-color: var(--vscode-sideBar-background);
    border-radius: 4px;
    padding: 20px;
}

.help-tips h3 {
    font-size: 16px;
    margin-bottom: 12px;
}

.help-tips ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

.help-tips li {
    padding: 8px 0;
    font-size: 13px;
    border-bottom: 1px solid var(--vscode-widget-border);
}

.help-tips li:last-child {
    border-bottom: none;
}

.help-tips li::before {
    content: "💡";
    margin-right: 8px;
}

/* No content states */
.no-recent {
    text-align: center;
    padding: 40px;
    color: var(--vscode-descriptionForeground);
    font-style: italic;
}
"#;