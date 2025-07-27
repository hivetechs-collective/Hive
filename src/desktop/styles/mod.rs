//! VS Code Theme Styles for Dioxus Desktop
//!
//! Provides complete VS Code dark theme styling with professional appearance

pub mod components;
pub mod example;
pub mod hivetechs_theme;
pub mod theme;

/// Get the complete global CSS styles for VS Code theming
pub fn get_global_styles() -> String {
    format!(
        r#"{base_styles}

/* Platform-specific font adjustments */
{platform_fonts}

/* Platform-specific UI adjustments */
{platform_adjustments}

/* Dialog styles */
{dialog_styles}

/* Activity Bar styles */
{activity_bar_styles}

/* Enhanced Explorer styles */
{explorer_styles}

/* Enhanced Status Bar styles */
{status_bar_styles}

/* VS Code Context Menu styles */
{vscode_menu_styles}

/* Welcome/Getting Started styles */
{welcome_styles}

/* Enhanced Layout styles */
{layout_styles}

/* HiveTechs Brand Theme */
{hivetechs_theme}

/* Terminal and Layout styles */
{terminal_styles}"#,
        base_styles = get_base_styles(),
        platform_fonts = get_platform_fonts(),
        platform_adjustments = theme::platform_adjustments(),
        dialog_styles = crate::desktop::dialogs::DIALOG_STYLES,
        activity_bar_styles = crate::desktop::activity_bar::ACTIVITY_BAR_STYLES,
        explorer_styles = crate::desktop::explorer_enhanced::EXPLORER_STYLES,
        status_bar_styles = crate::desktop::status_bar_enhanced::STATUS_BAR_STYLES,
        vscode_menu_styles = get_vscode_menu_styles(),
        welcome_styles = crate::desktop::welcome_enhanced::WELCOME_STYLES,
        layout_styles = crate::desktop::layout_enhanced::LAYOUT_STYLES,
        hivetechs_theme = hivetechs_theme::get_hivetechs_theme(),
        terminal_styles = get_terminal_styles()
    )
}

/// Get the base CSS styles
fn get_base_styles() -> &'static str {
    r#"
/* ===== CSS Reset & Base Styles ===== */
*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

html, body {
    height: 100%;
    width: 100%;
    overflow: hidden;
}

body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
    font-size: 13px;
    line-height: 1.5;
    color: #cccccc;
    background-color: #1e1e1e;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}

/* ===== VS Code Color Variables ===== */
:root {
    /* Editor Colors */
    --vscode-editor-background: #1e1e1e;
    --vscode-editor-foreground: #cccccc;
    --vscode-editor-selectionBackground: #264f78;
    --vscode-editor-lineHighlightBackground: #2a2d2e;

    /* Activity Bar */
    --vscode-activityBar-background: #333333;
    --vscode-activityBar-foreground: #ffffff;
    --vscode-activityBar-activeBorder: #007acc;
    --vscode-activityBar-inactiveForeground: #868686;

    /* Side Bar */
    --vscode-sideBar-background: #252526;
    --vscode-sideBar-foreground: #cccccc;
    --vscode-sideBar-border: #1e1e1e;

    /* Title Bar */
    --vscode-titleBar-activeBackground: #3c3c3c;
    --vscode-titleBar-activeForeground: #cccccc;
    --vscode-titleBar-inactiveBackground: #3c3c3c;

    /* Tabs */
    --vscode-tab-activeBackground: #1e1e1e;
    --vscode-tab-activeForeground: #ffffff;
    --vscode-tab-inactiveBackground: #2d2d30;
    --vscode-tab-inactiveForeground: #969696;
    --vscode-tab-border: #252526;

    /* Input */
    --vscode-input-background: #3c3c3c;
    --vscode-input-foreground: #cccccc;
    --vscode-input-border: #3c3c3c;
    --vscode-inputOption-activeBorder: #007acc;

    /* Button */
    --vscode-button-background: #0e639c;
    --vscode-button-foreground: #ffffff;
    --vscode-button-hoverBackground: #1177bb;
    --vscode-button-secondaryBackground: #3a3d41;
    --vscode-button-secondaryForeground: #cccccc;

    /* Dropdown */
    --vscode-dropdown-background: #3c3c3c;
    --vscode-dropdown-foreground: #cccccc;
    --vscode-dropdown-border: #3c3c3c;

    /* Lists */
    --vscode-list-activeSelectionBackground: #094771;
    --vscode-list-activeSelectionForeground: #ffffff;
    --vscode-list-hoverBackground: #2a2d2e;
    --vscode-list-inactiveSelectionBackground: #37373d;

    /* Scrollbar */
    --vscode-scrollbar-shadow: #000000;
    --vscode-scrollbarSlider-background: #79797966;
    --vscode-scrollbarSlider-hoverBackground: #646464b3;
    --vscode-scrollbarSlider-activeBackground: #bfbfbf66;

    /* Badge */
    --vscode-badge-background: #4d4d4d;
    --vscode-badge-foreground: #ffffff;

    /* Status Bar */
    --vscode-statusBar-background: #007acc;
    --vscode-statusBar-foreground: #ffffff;
    --vscode-statusBar-noFolderBackground: #68217a;

    /* Terminal */
    --vscode-terminal-background: #1e1e1e;
    --vscode-terminal-foreground: #cccccc;
    --vscode-terminal-ansiBlack: #000000;
    --vscode-terminal-ansiRed: #cd3131;
    --vscode-terminal-ansiGreen: #0dbc79;
    --vscode-terminal-ansiYellow: #e5e510;
    --vscode-terminal-ansiBlue: #2472c8;
    --vscode-terminal-ansiMagenta: #bc3fbc;
    --vscode-terminal-ansiCyan: #11a8cd;
    --vscode-terminal-ansiWhite: #e5e5e5;

    /* Syntax Highlighting */
    --vscode-syntax-keyword: #569cd6;
    --vscode-syntax-string: #ce9178;
    --vscode-syntax-number: #b5cea8;
    --vscode-syntax-comment: #6a9955;
    --vscode-syntax-function: #dcdcaa;
    --vscode-syntax-type: #4ec9b0;
    --vscode-syntax-variable: #9cdcfe;
}

/* ===== Layout Container ===== */
#app {
    background: var(--vscode-editor-background);
}

/* ===== Typography ===== */
h1, h2, h3, h4, h5, h6 {
    font-weight: 600;
    line-height: 1.2;
    color: var(--vscode-editor-foreground);
}

h1 { font-size: 2em; margin: 0.67em 0; }
h2 { font-size: 1.5em; margin: 0.75em 0; }
h3 { font-size: 1.17em; margin: 0.83em 0; }
h4 { font-size: 1em; margin: 1.12em 0; }
h5 { font-size: 0.83em; margin: 1.5em 0; }
h6 { font-size: 0.75em; margin: 1.67em 0; }

code, pre {
    font-family: 'Cascadia Code', 'Monaco', 'Consolas', 'Courier New', monospace;
    font-size: 13px;
    background: var(--vscode-editor-lineHighlightBackground);
    border-radius: 3px;
}

code {
    padding: 2px 4px;
}

pre {
    padding: 12px;
    overflow-x: auto;
}

/* ===== Links ===== */
a {
    color: #3794ff;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

/* ===== Selection ===== */
::selection {
    background: var(--vscode-editor-selectionBackground);
    color: #ffffff;
}

/* ===== Scrollbars ===== */
::-webkit-scrollbar {
    width: 14px;
    height: 14px;
}

::-webkit-scrollbar-track {
    background: transparent;
}

::-webkit-scrollbar-thumb {
    background: var(--vscode-scrollbarSlider-background);
    border: 3px solid transparent;
    border-radius: 7px;
    background-clip: padding-box;
}

::-webkit-scrollbar-thumb:hover {
    background: var(--vscode-scrollbarSlider-hoverBackground);
    background-clip: padding-box;
}

::-webkit-scrollbar-thumb:active {
    background: var(--vscode-scrollbarSlider-activeBackground);
    background-clip: padding-box;
}

::-webkit-scrollbar-corner {
    background: transparent;
}

/* ===== Buttons ===== */
.btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 14px;
    font-size: 13px;
    font-weight: 400;
    line-height: 22px;
    border: 1px solid transparent;
    border-radius: 2px;
    cursor: pointer;
    outline: none;
    transition: all 0.1s ease;
    font-family: inherit;
}

.btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
}

.btn-primary {
    background: var(--vscode-button-background);
    color: var(--vscode-button-foreground);
}

.btn-primary:hover:not(:disabled) {
    background: var(--vscode-button-hoverBackground);
}

.btn-secondary {
    background: var(--vscode-button-secondaryBackground);
    color: var(--vscode-button-secondaryForeground);
}

.btn-secondary:hover:not(:disabled) {
    background: #45494e;
}

.btn-success {
    background: #388a34;
    color: #ffffff;
}

.btn-success:hover:not(:disabled) {
    background: #44a340;
}

.btn-warning {
    background: #b89500;
    color: #ffffff;
}

.btn-warning:hover:not(:disabled) {
    background: #d4a800;
}

.btn-danger {
    background: #a1260d;
    color: #ffffff;
}

.btn-danger:hover:not(:disabled) {
    background: #be3319;
}

/* ===== Inputs ===== */
input[type="text"],
input[type="password"],
input[type="email"],
input[type="number"],
input[type="search"],
textarea,
select {
    background: var(--vscode-input-background);
    color: var(--vscode-input-foreground);
    border: 1px solid var(--vscode-input-border);
    border-radius: 2px;
    padding: 4px 8px;
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.1s ease;
}

input:focus,
textarea:focus,
select:focus {
    border-color: var(--vscode-inputOption-activeBorder);
}

textarea {
    resize: vertical;
    min-height: 60px;
}

/* ===== Checkboxes & Radios ===== */
input[type="checkbox"],
input[type="radio"] {
    width: 14px;
    height: 14px;
    margin-right: 6px;
    vertical-align: middle;
}

/* ===== Lists ===== */
.list-item {
    padding: 4px 8px;
    cursor: pointer;
    transition: background-color 0.1s ease;
}

.list-item:hover {
    background: var(--vscode-list-hoverBackground);
}

.list-item.active {
    background: var(--vscode-list-activeSelectionBackground);
    color: var(--vscode-list-activeSelectionForeground);
}

/* ===== Panels ===== */
.panel {
    background: var(--vscode-sideBar-background);
    border: 1px solid var(--vscode-sideBar-border);
    overflow: hidden;
}

.panel-header {
    background: var(--vscode-sideBar-background);
    border-bottom: 1px solid var(--vscode-sideBar-border);
    padding: 8px 12px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.panel-body {
    padding: 12px;
    overflow-y: auto;
}

/* ===== Tabs ===== */
.tabs {
    display: flex;
    background: var(--vscode-tab-inactiveBackground);
    border-bottom: 1px solid var(--vscode-tab-border);
    overflow-x: auto;
}

.tab {
    padding: 8px 16px;
    cursor: pointer;
    border-right: 1px solid var(--vscode-tab-border);
    color: var(--vscode-tab-inactiveForeground);
    transition: all 0.1s ease;
    white-space: nowrap;
}

.tab:hover {
    background: rgba(255, 255, 255, 0.05);
}

.tab.active {
    background: var(--vscode-tab-activeBackground);
    color: var(--vscode-tab-activeForeground);
}

/* ===== Badges ===== */
.badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 6px;
    font-size: 11px;
    font-weight: 600;
    background: var(--vscode-badge-background);
    color: var(--vscode-badge-foreground);
    border-radius: 11px;
    line-height: 1;
}

/* ===== Loading Spinner ===== */
.spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #ffffff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* ===== Icons ===== */
.icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    font-size: 14px;
}

/* ===== File Explorer Specific ===== */
.file-explorer {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #0E1414 0%, #181E21 100%);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--hive-yellow-dark);
}

.explorer-header {
    padding: 12px 16px;
    background: rgba(255, 193, 7, 0.05);
    border-bottom: 1px solid var(--hive-yellow-dark);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.explorer-header h3 {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--hive-yellow);
    margin: 0;
}

.file-tree {
    font-family: 'Inter', sans-serif;
    font-size: 13px;
    user-select: none;
    overflow-y: auto;
    flex: 1;
    padding: 8px 0;
}

.file-item {
    display: flex;
    align-items: center;
    height: 28px;
    cursor: pointer;
    transition: all 0.1s ease;
    position: relative;
}

.file-item:hover {
    background: linear-gradient(90deg, rgba(255, 193, 7, 0.1) 0%, rgba(255, 193, 7, 0.05) 100%);
}

.file-item.selected {
    background: linear-gradient(90deg, rgba(255, 193, 7, 0.2) 0%, rgba(255, 193, 7, 0.1) 100%);
    border-left: 3px solid var(--hive-yellow);
}

.file-item .expand-icon {
    color: var(--hive-yellow-dark);
    transition: transform 0.2s ease;
}

.file-item:hover .expand-icon {
    color: var(--hive-yellow);
}

.file-icon {
    margin-right: 6px;
    flex-shrink: 0;
    color: var(--hive-yellow-light);
}

.file-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #cccccc;
}

.file-item:hover .file-name {
    color: #ffffff;
}

.file-item.selected .file-name {
    color: var(--hive-yellow);
    font-weight: 500;
}

.git-status {
    margin-left: 4px;
    font-size: 11px;
    font-weight: 600;
    opacity: 0.8;
}

.file-children {
    position: relative;
}

.file-children::before {
    content: '';
    position: absolute;
    left: 10px;
    top: 0;
    bottom: 0;
    width: 1px;
    background: rgba(255, 193, 7, 0.2);
}

.empty-state {
    text-align: center;
    padding: 20px;
    color: #666;
    font-style: italic;
}

/* Git status colors with HiveTechs accent */
.file-item[data-git-status="modified"] .git-status { color: #e2c08d; }
.file-item[data-git-status="added"] .git-status { color: #73c991; }
.file-item[data-git-status="deleted"] .git-status { color: #f48771; }
.file-item[data-git-status="untracked"] .git-status { color: #75beff; }
.file-item[data-git-status="renamed"] .git-status { color: var(--hive-yellow); }

/* Tree item compatibility */
.tree-item {
    display: flex;
    align-items: center;
    padding: 2px 8px;
    cursor: pointer;
    transition: background-color 0.1s ease;
}

.tree-item:hover {
    background: var(--vscode-list-hoverBackground);
}

.tree-item.selected {
    background: var(--vscode-list-activeSelectionBackground);
    color: var(--vscode-list-activeSelectionForeground);
}

.tree-item-icon {
    margin-right: 6px;
    flex-shrink: 0;
}

.tree-item-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* ===== Terminal Specific ===== */
.terminal {
    background: var(--vscode-terminal-background);
    color: var(--vscode-terminal-foreground);
    font-family: 'Cascadia Code', 'Monaco', 'Consolas', monospace;
    font-size: 13px;
    line-height: 1.4;
    padding: 8px;
    overflow-y: auto;
}

.terminal-line {
    white-space: pre-wrap;
    word-break: break-all;
}

/* ===== Chat/Consensus Panel ===== */
.chat-container {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    height: 100%;
}

.chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    min-height: 0;
}

.message {
    padding: 12px;
    margin-bottom: 8px;
    border-radius: 4px;
    animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(5px); }
    to { opacity: 1; transform: translateY(0); }
}

.message-user {
    background: rgba(0, 122, 204, 0.1);
    border-left: 3px solid var(--vscode-inputOption-activeBorder);
}

.message-ai {
    background: rgba(255, 255, 255, 0.05);
    border-left: 3px solid #6a9955;
}

.message-header {
    font-weight: 600;
    margin-bottom: 4px;
    font-size: 12px;
    opacity: 0.8;
}

.message-content {
    line-height: 1.5;
}

/* Chat Input Container */
.chat-input-container {
    flex-shrink: 0;
    border-top: 1px solid var(--vscode-sideBar-border);
    background: var(--vscode-input-background);
    padding: 12px;
}

/* ===== Code Blocks in Messages ===== */
.code-block {
    background: var(--vscode-editor-background);
    border: 1px solid var(--vscode-tab-border);
    border-radius: 4px;
    margin: 8px 0;
    overflow: hidden;
}

.code-block-header {
    background: var(--vscode-tab-inactiveBackground);
    padding: 4px 12px;
    font-size: 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.code-block-content {
    padding: 12px;
    overflow-x: auto;
}

/* ===== Syntax Highlighting ===== */
.keyword { color: var(--vscode-syntax-keyword); }
.string { color: var(--vscode-syntax-string); }
.number { color: var(--vscode-syntax-number); }
.comment { color: var(--vscode-syntax-comment); font-style: italic; }
.function { color: var(--vscode-syntax-function); }
.type { color: var(--vscode-syntax-type); }
.variable { color: var(--vscode-syntax-variable); }

/* ===== Status Bar ===== */
.status-bar {
    background: var(--vscode-statusBar-background);
    color: var(--vscode-statusBar-foreground);
    height: 22px;
    display: flex;
    align-items: center;
    padding: 0 8px;
    font-size: 12px;
    user-select: none;
}

.status-bar-item {
    padding: 0 8px;
    display: flex;
    align-items: center;
    height: 100%;
}

.status-bar-item:hover {
    background: rgba(255, 255, 255, 0.1);
}

/* ===== Tooltips ===== */
.tooltip {
    position: absolute;
    background: #1e1e1e;
    border: 1px solid #454545;
    color: #cccccc;
    padding: 4px 8px;
    font-size: 12px;
    border-radius: 3px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    z-index: 1000;
    pointer-events: none;
}

/* ===== Context Menu ===== */
.context-menu {
    position: absolute;
    background: var(--vscode-dropdown-background);
    border: 1px solid var(--vscode-dropdown-border);
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    min-width: 150px;
    z-index: 1000;
    padding: 4px 0;
}

.context-menu-item {
    padding: 4px 16px;
    cursor: pointer;
    font-size: 13px;
    display: flex;
    align-items: center;
}

.context-menu-item:hover {
    background: var(--vscode-list-hoverBackground);
}

.context-menu-separator {
    height: 1px;
    background: var(--vscode-dropdown-border);
    margin: 4px 0;
}

/* ===== Accessibility ===== */
.sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
}

/* ===== Focus Styles ===== */
:focus {
    outline: 1px solid var(--vscode-inputOption-activeBorder);
    outline-offset: -1px;
}

button:focus,
.btn:focus {
    outline: 1px solid var(--vscode-inputOption-activeBorder);
    outline-offset: 2px;
}

/* ===== Responsive Design ===== */
@media (max-width: 768px) {
    body {
        font-size: 14px;
    }

    .panel-body {
        padding: 8px;
    }

    .tab {
        padding: 6px 12px;
    }
}

/* ===== Print Styles ===== */
@media print {
    body {
        background: white;
        color: black;
    }

    .no-print {
        display: none !important;
    }
}

/* ===== Utility Classes ===== */
.text-muted { color: #969696; }
.text-danger { color: #f48771; }
.text-success { color: #89d185; }
.text-warning { color: #e9c46a; }
.text-info { color: #75beff; }

.bg-danger { background: rgba(244, 135, 113, 0.1); }
.bg-success { background: rgba(137, 209, 133, 0.1); }
.bg-warning { background: rgba(233, 196, 106, 0.1); }
.bg-info { background: rgba(117, 190, 255, 0.1); }

.flex { display: flex; }
.flex-col { flex-direction: column; }
.flex-1 { flex: 1; }
.items-center { align-items: center; }
.justify-between { justify-content: space-between; }
.justify-center { justify-content: center; }

.p-1 { padding: 4px; }
.p-2 { padding: 8px; }
.p-3 { padding: 12px; }
.p-4 { padding: 16px; }

.m-1 { margin: 4px; }
.m-2 { margin: 8px; }
.m-3 { margin: 12px; }
.m-4 { margin: 16px; }

.w-full { width: 100%; }
.h-full { height: 100%; }

.overflow-auto { overflow: auto; }
.overflow-hidden { overflow: hidden; }
.overflow-x-auto { overflow-x: auto; }
.overflow-y-auto { overflow-y: auto; }

.cursor-pointer { cursor: pointer; }
.select-none { user-select: none; }

.opacity-50 { opacity: 0.5; }
.opacity-75 { opacity: 0.75; }

.transition { transition: all 0.2s ease; }
.transition-fast { transition: all 0.1s ease; }

/* ===== Custom Animations ===== */
@keyframes slideIn {
    from { transform: translateX(-100%); }
    to { transform: translateX(0); }
}

@keyframes slideOut {
    from { transform: translateX(0); }
    to { transform: translateX(-100%); }
}

@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes fadeOut {
    from { opacity: 1; }
    to { opacity: 0; }
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

.animate-slide-in { animation: slideIn 0.3s ease; }
.animate-slide-out { animation: slideOut 0.3s ease; }
.animate-fade-in { animation: fadeIn 0.3s ease; }
.animate-fade-out { animation: fadeOut 0.3s ease; }
.animate-pulse { animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite; }

/* ===== Additional Component Styles ===== */

/* Icon Button (no text, just icon) */
.btn-icon {
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    color: var(--vscode-editor-foreground);
    border: none;
}

.btn-icon:hover {
    background: var(--vscode-list-hoverBackground);
}

/* Ghost Button (minimal styling) */
.btn-ghost {
    background: transparent;
    color: var(--vscode-editor-foreground);
    border: 1px solid transparent;
}

.btn-ghost:hover {
    background: var(--vscode-list-hoverBackground);
    border-color: var(--vscode-input-border);
}

/* Icon Sizes */
.icon-sm { font-size: 12px; width: 12px; height: 12px; }
.icon-md { font-size: 16px; width: 16px; height: 16px; }
.icon-lg { font-size: 20px; width: 20px; height: 20px; }

/* Panel Variants */
.panel-sidebar {
    background: var(--vscode-sideBar-background);
    color: var(--vscode-sideBar-foreground);
}

.panel-editor {
    background: var(--vscode-editor-background);
    color: var(--vscode-editor-foreground);
}

.panel-terminal {
    background: var(--vscode-terminal-background);
    color: var(--vscode-terminal-foreground);
}

.panel-output {
    background: var(--vscode-editor-background);
    color: var(--vscode-editor-foreground);
    font-family: 'Cascadia Code', 'Monaco', 'Consolas', monospace;
}

/* Theme Switcher */
.theme-switcher {
    display: inline-flex;
    align-items: center;
}

/* Consensus Stage Indicators */
.consensus-stage {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    margin: 4px 0;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.05);
    transition: all 0.3s ease;
}

.consensus-stage.active {
    background: rgba(0, 122, 204, 0.2);
    border-left: 3px solid var(--vscode-inputOption-activeBorder);
}

.consensus-stage.complete {
    background: rgba(137, 209, 133, 0.1);
    border-left: 3px solid #89d185;
}

.consensus-stage-icon {
    margin-right: 8px;
    font-size: 16px;
}

.consensus-stage-name {
    font-weight: 600;
    margin-right: auto;
}

.consensus-stage-status {
    font-size: 12px;
    color: var(--vscode-tab-inactiveForeground);
}

/* File Explorer Icons (Git status) */
.tree-item.modified .tree-item-label { color: #e2c08d; }
.tree-item.added .tree-item-label { color: #73c991; }
.tree-item.deleted .tree-item-label { color: #f48771; }
.tree-item.renamed .tree-item-label { color: #75beff; }
.tree-item.conflicted .tree-item-label { color: #e51400; }

/* Split View */
.split-view {
    display: flex;
    height: 100%;
    width: 100%;
}

.split-view-horizontal {
    flex-direction: row;
}

.split-view-vertical {
    flex-direction: column;
}

.split-view-pane {
    flex: 1;
    overflow: hidden;
    position: relative;
}

.split-view-divider {
    background: var(--vscode-sideBar-border);
    cursor: col-resize;
    user-select: none;
}

.split-view-horizontal .split-view-divider {
    width: 1px;
    cursor: col-resize;
}

.split-view-vertical .split-view-divider {
    height: 1px;
    cursor: row-resize;
}

.split-view-divider:hover {
    background: var(--vscode-inputOption-activeBorder);
}

/* Search Input */
.search-input {
    display: flex;
    align-items: center;
    background: var(--vscode-input-background);
    border: 1px solid var(--vscode-input-border);
    border-radius: 2px;
    padding: 4px 8px;
}

.search-input:focus-within {
    border-color: var(--vscode-inputOption-activeBorder);
}

.search-input input {
    background: transparent;
    border: none;
    flex: 1;
    padding: 0;
}

.search-input input:focus {
    outline: none;
    border: none;
}

.search-input-icon {
    margin-right: 6px;
    color: var(--vscode-input-foreground);
    opacity: 0.6;
}

/* Notification Toast */
.notification {
    position: fixed;
    top: 20px;
    right: 20px;
    min-width: 300px;
    background: var(--vscode-dropdown-background);
    border: 1px solid var(--vscode-dropdown-border);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    padding: 12px 16px;
    display: flex;
    align-items: flex-start;
    animation: slideInRight 0.3s ease;
    z-index: 1000;
}

@keyframes slideInRight {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
}

.notification-icon {
    margin-right: 12px;
    font-size: 18px;
}

.notification-content {
    flex: 1;
}

.notification-title {
    font-weight: 600;
    margin-bottom: 4px;
}

.notification-message {
    font-size: 12px;
    color: var(--vscode-tab-inactiveForeground);
}

.notification-close {
    margin-left: 12px;
    cursor: pointer;
    opacity: 0.6;
}

.notification-close:hover {
    opacity: 1;
}

.notification.error {
    border-color: #f48771;
}

.notification.error .notification-icon {
    color: #f48771;
}

.notification.success {
    border-color: #89d185;
}

.notification.success .notification-icon {
    color: #89d185;
}

.notification.warning {
    border-color: #e9c46a;
}

.notification.warning .notification-icon {
    color: #e9c46a;
}

.notification.info {
    border-color: #75beff;
}

.notification.info .notification-icon {
    color: #75beff;
}

/* Markdown Content Styling */
.markdown-content {
    line-height: 1.6;
    color: var(--vscode-editor-foreground);
}

.markdown-content h1,
.markdown-content h2,
.markdown-content h3,
.markdown-content h4,
.markdown-content h5,
.markdown-content h6 {
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    line-height: 1.25;
}

.markdown-content h1 { font-size: 2em; border-bottom: 1px solid var(--vscode-tab-border); padding-bottom: 0.3em; }
.markdown-content h2 { font-size: 1.5em; }
.markdown-content h3 { font-size: 1.25em; }

.markdown-content p {
    margin-bottom: 16px;
}

.markdown-content ul,
.markdown-content ol {
    margin-bottom: 16px;
    padding-left: 2em;
}

.markdown-content li {
    margin-bottom: 4px;
}

.markdown-content blockquote {
    border-left: 4px solid var(--vscode-tab-border);
    padding-left: 16px;
    margin: 16px 0;
    color: var(--vscode-tab-inactiveForeground);
}

.markdown-content table {
    border-collapse: collapse;
    margin: 16px 0;
    width: 100%;
}

.markdown-content table th,
.markdown-content table td {
    border: 1px solid var(--vscode-tab-border);
    padding: 8px 12px;
}

.markdown-content table th {
    background: var(--vscode-editor-lineHighlightBackground);
    font-weight: 600;
}

.markdown-content img {
    max-width: 100%;
    height: auto;
}

.markdown-content hr {
    border: none;
    border-top: 1px solid var(--vscode-tab-border);
    margin: 24px 0;
}

/* Progress Bar */
.progress-bar {
    height: 2px;
    background: var(--vscode-input-background);
    border-radius: 1px;
    overflow: hidden;
    position: relative;
}

.progress-bar-fill {
    height: 100%;
    background: var(--vscode-inputOption-activeBorder);
    transition: width 0.3s ease;
}

.progress-bar.indeterminate .progress-bar-fill {
    width: 30%;
    position: absolute;
    animation: indeterminate 1.5s infinite;
}

@keyframes indeterminate {
    0% { left: -30%; }
    100% { left: 100%; }
}

/* Breadcrumb Navigation */
.breadcrumb {
    display: flex;
    align-items: center;
    font-size: 13px;
    color: var(--vscode-tab-inactiveForeground);
    padding: 4px 0;
}

.breadcrumb-item {
    display: flex;
    align-items: center;
}

.breadcrumb-item:not(:last-child)::after {
    content: '›';
    margin: 0 8px;
    color: var(--vscode-tab-inactiveForeground);
}

.breadcrumb-item a {
    color: inherit;
    text-decoration: none;
}

.breadcrumb-item a:hover {
    color: var(--vscode-editor-foreground);
}

.breadcrumb-item:last-child {
    color: var(--vscode-editor-foreground);
}

/* Consensus Progress Panel - Always Visible */
.consensus-progress-overlay {
    position: fixed;
    top: 30px;
    right: 20px;
    width: 350px;
    z-index: 1000;
    opacity: 1;
    transform: translateX(0);
    pointer-events: auto;
}

.consensus-progress-overlay.active {
    opacity: 1;
    transform: translateX(0);
    pointer-events: auto;
}

.consensus-progress-overlay.always-visible {
    opacity: 1 !important;
    transform: translateX(0) !important;
    pointer-events: auto !important;
    display: block !important;
    visibility: visible !important;
}

.consensus-progress {
    background: var(--vscode-sideBar-background);
    border: 1px solid var(--vscode-sideBar-border);
    border-radius: 6px;
    padding: 16px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(10px);
}

.consensus-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--vscode-sideBar-border);
}

.consensus-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--vscode-editor-foreground);
    display: flex;
    align-items: center;
    gap: 8px;
}

.consensus-profile-name {
    font-size: 12px;
    color: var(--vscode-tab-inactiveForeground);
    font-weight: 500;
}

.consensus-stages {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 16px;
}

.stage {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    background: var(--vscode-input-background);
    border: 1px solid var(--vscode-input-border);
    border-radius: 4px;
    transition: all 0.2s ease;
}

.stage.active {
    border-color: var(--vscode-inputOption-activeBorder);
    background: var(--vscode-editor-lineHighlightBackground);
}

.stage.completed {
    border-color: #388a34;
    background: rgba(56, 138, 52, 0.1);
}

.stage.error {
    border-color: #f48771;
    background: rgba(244, 135, 113, 0.1);
}

.stage-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.stage-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--vscode-editor-foreground);
}

.stage-status-icon {
    font-size: 14px;
}

.stage-model {
    font-size: 11px;
    color: var(--vscode-tab-inactiveForeground);
    font-family: 'SF Mono', 'Monaco', 'Cascadia Code', 'Consolas', monospace;
}

.stage-progress-container {
    margin-top: 4px;
}

.overall-progress {
    margin-bottom: 16px;
}

.consensus-stats {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding-top: 8px;
    border-top: 1px solid var(--vscode-sideBar-border);
}

.stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
}

.stat-label {
    font-size: 11px;
    color: var(--vscode-tab-inactiveForeground);
    font-weight: 500;
}

.stat-value {
    font-size: 12px;
    color: var(--vscode-editor-foreground);
    font-weight: 600;
    font-family: 'SF Mono', 'Monaco', 'Cascadia Code', 'Consolas', monospace;
}

/* ===== Context Menu ===== */
.context-menu {
    position: fixed;
    background: var(--vscode-menu-background);
    border: 1px solid var(--vscode-menu-border);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    padding: 4px 0;
    min-width: 180px;
    z-index: 1000;
    font-size: 13px;
}

.context-menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    cursor: pointer;
    color: var(--vscode-menu-foreground);
    transition: background-color 0.1s ease;
}

.context-menu-item:hover {
    background: var(--vscode-menu-selectionBackground);
    color: var(--vscode-menu-selectionForeground);
}

.context-menu-item.disabled {
    opacity: 0.5;
    cursor: default;
}

.context-menu-item.disabled:hover {
    background: transparent;
    color: var(--vscode-menu-foreground);
}

.context-menu-separator {
    height: 1px;
    background: var(--vscode-menu-separatorBackground);
    margin: 4px 0;
}

/* ===== Explorer Toolbar ===== */
.explorer-toolbar-btn {
    background: transparent;
    border: none;
    color: #cccccc;
    padding: 4px 8px;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    height: 22px;
    transition: background-color 0.1s ease;
}

.explorer-toolbar-btn:hover {
    background: #2a2d2e !important;
}

.explorer-toolbar-btn:active {
    background: #094771 !important;
}

/* ===== File Tree Items ===== */
.file-tree-item {
    display: flex;
    align-items: center;
    height: 22px;
    line-height: 22px;
    cursor: pointer;
    user-select: none;
    font-size: 13px;
    color: #cccccc;
    background-color: transparent;
    position: relative;
}

.file-tree-item:hover {
    background-color: #2a2d2e;
}

.file-tree-item.selected {
    background-color: #094771;
    color: #ffffff;
}

.file-tree-item.selected:hover {
    background-color: #094771;
}

/* ===== VS Code Monaco Menu Styles ===== */
.vscode-menu-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 999;
}

.monaco-menu {
    position: absolute;
    background-color: var(--vscode-menu-background, #3c3c3c);
    color: var(--vscode-menu-foreground, #cccccc);
    border: 1px solid var(--vscode-menu-border, #555555);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    border-radius: 5px;
    font-size: 13px;
    line-height: 22px;
    z-index: 1000;
    animation: menuFadeIn 0.1s ease-out;
}

@keyframes menuFadeIn {
    from { opacity: 0; transform: translateY(-5px); }
    to { opacity: 1; transform: translateY(0); }
}

.monaco-action-bar.vertical {
    padding: 4px 0;
}

.monaco-action-bar .actions-container {
    list-style: none;
    margin: 0;
    padding: 0;
}

.monaco-action-bar .action-item {
    display: block;
    position: relative;
}

.monaco-action-bar .action-menu-item {
    display: flex;
    align-items: center;
    padding: 0 16px;
    height: 24px;
    color: inherit;
    text-decoration: none;
    cursor: pointer;
    position: relative;
    white-space: nowrap;
    transition: background-color 0.1s ease;
}

.monaco-action-bar .action-menu-item.focused,
.monaco-action-bar .action-menu-item:hover {
    background-color: var(--vscode-menu-selectionBackground, #094771);
    color: var(--vscode-menu-selectionForeground, #ffffff);
}

.monaco-action-bar .action-menu-item.disabled {
    opacity: 0.4;
    cursor: default;
}

.monaco-action-bar .action-menu-item.disabled:hover {
    background-color: transparent;
    color: inherit;
}

.monaco-action-bar .action-label {
    flex: 1;
    margin-right: 8px;
}

.monaco-action-bar .keybinding {
    margin-left: auto;
    opacity: 0.7;
    font-size: 11px;
    padding-left: 16px;
}

.monaco-action-bar .submenu-indicator {
    position: absolute;
    right: 8px;
    font-size: 10px;
}

.monaco-action-bar-separator {
    height: 1px;
    background-color: var(--vscode-menu-separatorBackground, #606060);
    margin: 4px 0;
}

/* Monaco Submenu */
.monaco-submenu {
    position: absolute;
    background-color: var(--vscode-menu-background, #3c3c3c);
    color: var(--vscode-menu-foreground, #cccccc);
    border: 1px solid var(--vscode-menu-border, #555555);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    border-radius: 5px;
    z-index: 1001;
    animation: submenuFadeIn 0.1s ease-out;
}

@keyframes submenuFadeIn {
    from { opacity: 0; transform: translateX(-5px); }
    to { opacity: 1; transform: translateX(0); }
}

/* Codicon support for menu icons */
.codicon {
    font-family: codicon;
    display: inline-block;
    font-style: normal;
    font-weight: normal;
    font-size: 14px;
    line-height: 1;
    text-decoration: none;
    text-rendering: auto;
    text-align: center;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    user-select: none;
    -webkit-user-select: none;
    -ms-user-select: none;
}

/* Common codicon definitions */
.codicon-new-file:before { content: '\ea7f' }
.codicon-new-folder:before { content: '\ea80' }
.codicon-edit:before { content: '\ea73' }
.codicon-trash:before { content: '\ea81' }
.codicon-copy:before { content: '\eae1' }
.codicon-cut:before { content: '\eae2' }
.codicon-paste:before { content: '\eae3' }
.codicon-search:before { content: '\ea6d' }
.codicon-diff:before { content: '\ea93' }
.codicon-split-horizontal:before { content: '\eb57' }
.codicon-terminal:before { content: '\ea85' }
.codicon-folder-opened:before { content: '\eab4' }
.codicon-chevron-right:before { content: '\eb60' }
.codicon-check:before { content: '\eab2' }
"#
}

/// Get platform-specific font styles
pub fn get_platform_fonts() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        r#"
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Inter', 'Segoe UI', sans-serif;
        }
        code, pre, .terminal {
            font-family: 'SF Mono', 'Monaco', 'Cascadia Code', 'Consolas', monospace;
        }
        "#
    }

    #[cfg(target_os = "windows")]
    {
        r#"
        body {
            font-family: 'Segoe UI', 'Inter', -apple-system, system-ui, sans-serif;
        }
        code, pre, .terminal {
            font-family: 'Cascadia Code', 'Consolas', 'Courier New', monospace;
        }
        "#
    }

    #[cfg(target_os = "linux")]
    {
        r#"
        body {
            font-family: 'Inter', 'Ubuntu', 'Cantarell', 'DejaVu Sans', system-ui, sans-serif;
        }
        code, pre, .terminal {
            font-family: 'Cascadia Code', 'Ubuntu Mono', 'DejaVu Sans Mono', 'Consolas', monospace;
        }
        "#
    }
}

/// Get the appropriate Font Awesome icon class for a folder
pub fn get_folder_icon(is_expanded: bool) -> &'static str {
    if is_expanded {
        "fa-solid fa-folder-open"
    } else {
        "fa-solid fa-folder"
    }
}

/// Get the appropriate Font Awesome icon class for a file based on its extension
pub fn get_file_icon(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        // Code files
        "rs" => "fa-brands fa-rust",
        "js" | "jsx" => "fa-brands fa-js",
        "ts" | "tsx" => "fa-brands fa-js", // TypeScript uses JS icon
        "py" => "fa-brands fa-python",
        "java" => "fa-brands fa-java",
        "go" => "fa-brands fa-golang",
        "cpp" | "cc" | "cxx" | "c++" => "fa-solid fa-code",
        "c" => "fa-solid fa-code",
        "cs" => "fa-solid fa-code",
        "php" => "fa-brands fa-php",
        "rb" => "fa-solid fa-gem",
        "swift" => "fa-brands fa-swift",
        "kt" => "fa-solid fa-code",
        "dart" => "fa-solid fa-code",
        "r" => "fa-brands fa-r-project",

        // Web files
        "html" | "htm" => "fa-brands fa-html5",
        "css" | "scss" | "sass" | "less" => "fa-brands fa-css3-alt",
        "vue" => "fa-brands fa-vuejs",
        "react" => "fa-brands fa-react",

        // Config files
        "json" => "fa-solid fa-brackets-curly",
        "toml" => "fa-solid fa-gear",
        "yaml" | "yml" => "fa-solid fa-gear",
        "xml" => "fa-solid fa-code",
        "ini" | "cfg" | "conf" => "fa-solid fa-gear",

        // Documentation
        "md" | "markdown" => "fa-brands fa-markdown",
        "txt" => "fa-solid fa-file-lines",
        "pdf" => "fa-solid fa-file-pdf",
        "doc" | "docx" => "fa-solid fa-file-word",

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "svg" | "webp" => "fa-solid fa-image",

        // Archives
        "zip" | "tar" | "gz" | "rar" | "7z" => "fa-solid fa-file-zipper",

        // Scripts
        "sh" | "bash" | "zsh" | "fish" => "fa-solid fa-terminal",
        "ps1" | "psm1" | "psd1" => "fa-solid fa-terminal",
        "bat" | "cmd" => "fa-solid fa-terminal",

        // Data files
        "sql" => "fa-solid fa-database",
        "db" | "sqlite" => "fa-solid fa-database",
        "csv" => "fa-solid fa-table",

        // Git files
        "gitignore" | "gitattributes" => "fa-brands fa-git-alt",

        // Lock files
        "lock" => "fa-solid fa-lock",

        // Default
        _ => "fa-solid fa-file",
    }
}

/// Get VS Code menu styles
fn get_vscode_menu_styles() -> &'static str {
    r#"
/* VS Code Menu Backdrop */
.vscode-menu-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 999;
}

/* Monaco Menu Styles */
.monaco-menu {
    position: absolute;
    background-color: var(--vscode-menu-background, #3c3c3c);
    color: var(--vscode-menu-foreground, #cccccc);
    border: 1px solid var(--vscode-menu-border, #555555);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    border-radius: 5px;
    font-size: 13px;
    line-height: 22px;
    z-index: 1000;
    animation: menuFadeIn 0.1s ease-out;
}

@keyframes menuFadeIn {
    from {
        opacity: 0;
        transform: translateY(-5px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.monaco-action-bar.vertical {
    padding: 4px 0;
}

.monaco-action-bar .actions-container {
    list-style: none;
    margin: 0;
    padding: 0;
}

.monaco-action-bar .action-item {
    position: relative;
}

.monaco-action-bar .action-menu-item {
    display: flex;
    align-items: center;
    padding: 0 26px;
    height: 22px;
    color: inherit;
    text-decoration: none;
    cursor: pointer;
    white-space: nowrap;
    position: relative;
}

.monaco-action-bar .action-menu-item:hover:not(.disabled) {
    background-color: var(--vscode-menu-selectionBackground, #094771);
    color: var(--vscode-menu-selectionForeground, #ffffff);
}

.monaco-action-bar .action-menu-item.focused {
    background-color: var(--vscode-menu-selectionBackground, #094771);
    color: var(--vscode-menu-selectionForeground, #ffffff);
    outline: 1px solid var(--hivetechs-yellow, #FFC107);
    outline-offset: -1px;
}

.monaco-action-bar .action-menu-item.disabled {
    opacity: 0.4;
    cursor: default;
}

.monaco-action-bar .action-label {
    flex: 1;
    margin-right: 20px;
}

.monaco-action-bar .keybinding {
    margin-left: auto;
    opacity: 0.7;
    font-size: 11px;
}

.monaco-action-bar-separator {
    height: 1px;
    background-color: var(--vscode-menu-separatorBackground, #606060);
    margin: 4px 0;
}

/* Submenu support */
.monaco-submenu {
    position: absolute;
    background-color: var(--vscode-menu-background, #3c3c3c);
    border: 1px solid var(--vscode-menu-border, #555555);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    border-radius: 5px;
    animation: submenuFadeIn 0.1s ease-out;
}

@keyframes submenuFadeIn {
    from {
        opacity: 0;
        transform: translateX(-5px);
    }
    to {
        opacity: 1;
        transform: translateX(0);
    }
}

.submenu-indicator {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
}

/* Checkbox menu items */
.monaco-action-bar .action-menu-item[role="menuitemcheckbox"] .codicon-check {
    visibility: hidden;
}

.monaco-action-bar .action-menu-item[role="menuitemcheckbox"][aria-checked="true"] .codicon-check {
    visibility: visible;
}

/* Icons in menu items */
.monaco-action-bar .codicon {
    font-size: 16px;
    line-height: 22px;
}

.monaco-action-bar .action-menu-item .codicon:first-child {
    margin-right: 8px;
}
"#
}

/// Additional theme variants
pub mod themes {
    pub const LIGHT_THEME_OVERRIDES: &str = r#"
    :root {
        --vscode-editor-background: #ffffff;
        --vscode-editor-foreground: #000000;
        --vscode-editor-selectionBackground: #add6ff;
        --vscode-editor-lineHighlightBackground: #f3f3f3;

        --vscode-activityBar-background: #2c2c2c;
        --vscode-activityBar-foreground: #ffffff;
        --vscode-activityBar-activeBorder: #0078d4;

        --vscode-sideBar-background: #f3f3f3;
        --vscode-sideBar-foreground: #616161;
        --vscode-sideBar-border: #e7e7e7;

        --vscode-statusBar-background: #0078d4;
        --vscode-statusBar-foreground: #ffffff;
    }
    "#;

    pub const HIGH_CONTRAST_THEME_OVERRIDES: &str = r#"
    :root {
        --vscode-editor-background: #000000;
        --vscode-editor-foreground: #ffffff;
        --vscode-editor-selectionBackground: #ffffff;
        --vscode-editor-lineHighlightBackground: #ffffff1a;

        --vscode-activityBar-background: #000000;
        --vscode-activityBar-foreground: #ffffff;
        --vscode-activityBar-activeBorder: #f38518;

        --vscode-sideBar-background: #000000;
        --vscode-sideBar-foreground: #ffffff;
        --vscode-sideBar-border: #6fc3df;
    }
    "#;
}

/// Get terminal and layout styles
fn get_terminal_styles() -> &'static str {
    r#"
/* Main layout with terminal */
.main-layout-with-terminal {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
}

/* Workspace container - main area below menu bar */
.workspace-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    height: 100%;
}

/* Editor area - contains file explorer and chat */
.editor-area {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
}

/* Center content area */
.center-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

/* Panel container - terminal panel at bottom */
.panel-container {
    display: flex;
    flex-direction: column;
    height: 35%;
    min-height: 150px;
    max-height: 80%;
    background: #1e1e1e;
    border-top: 1px solid #474747;
}

/* Panel header bar */
.panel-header {
    display: flex;
    align-items: center;
    height: 35px;
    background: #252526;
    padding: 0 10px;
    border-bottom: 1px solid #1e1e1e;
    user-select: none;
}

/* Panel content area */
.panel-content {
    flex: 1;
    overflow: hidden;
}

/* Terminal tabs styles */
.terminal-tabs {
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    height: 100%;
}

.terminal-tab-bar {
    display: flex;
    align-items: center;
    height: 35px;
    background: #252526;
    border-bottom: 1px solid #1e1e1e;
    padding: 0 10px;
    gap: 2px;
}

.terminal-tab {
    display: flex;
    align-items: center;
    padding: 0 12px;
    height: 35px;
    cursor: pointer;
    user-select: none;
    font-size: 13px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    border-right: 1px solid #1e1e1e;
    gap: 6px;
    transition: background 0.1s;
}

.terminal-tab.active {
    background: #1e1e1e;
    color: #cccccc;
}

.terminal-tab:not(.active) {
    background: transparent;
    color: #969696;
}

.terminal-tab:hover {
    background: #2a2a2a;
}

.terminal-content {
    flex: 1;
    background: #000000;
    color: #cccccc;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 13px;
    overflow-y: auto;
    padding: 10px;
}

/* Terminal output styles */
.terminal-line {
    margin: 2px 0;
    display: flex;
    align-items: baseline;
}

.terminal-line.command {
    color: #569cd6;
    font-weight: 600;
}

.terminal-line.output {
    color: #cccccc;
}

.terminal-line.error {
    color: #f44747;
}

.terminal-line.success {
    color: #4ec9b0;
}

.terminal-line.prompt {
    color: #dcdcaa;
    font-weight: 600;
}

/* Terminal input */
.terminal-input-container {
    display: flex;
    align-items: center;
    padding: 8px 10px;
    background: #1a1a1a;
    border-top: 1px solid #333333;
}

.terminal-input {
    flex: 1;
    background: transparent;
    border: none;
    color: #cccccc;
    font-family: inherit;
    font-size: inherit;
    outline: none;
    padding: 4px;
}

.terminal-prompt {
    color: #4ec9b0;
    margin-right: 8px;
    font-weight: 600;
}

/* Resizable terminal */
.terminal-resize-handle {
    height: 4px;
    background: transparent;
    cursor: ns-resize;
    position: relative;
}

.terminal-resize-handle:hover {
    background: #007acc;
}

/* Main layout adjustments */
.main-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.center-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
}

.file-explorer {
    width: 250px;
    min-width: 200px;
    max-width: 500px;
    background: #252526;
    border-right: 1px solid #3e3e42;
    overflow-y: auto;
}

.chat-container {
    display: flex;
    flex-direction: column;
    background: #1e1e1e;
    overflow: hidden;
}

/* CSS Grid Layout for main app */
.app-container-grid {
    display: grid;
    grid-template-rows: 30px 1fr 22px;
    grid-template-columns: 1fr;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
}

.main-layout-grid {
    display: grid;
    grid-template-columns: 250px 1fr 350px;
    grid-template-rows: 1fr;
    height: 100%;
    overflow: hidden;
}

.center-panel-grid {
    display: grid;
    grid-template-rows: 1fr 250px;
    grid-template-columns: 1fr;
    overflow: hidden;
}

.terminal-panel-grid {
    background: #ff0000;
    border-top: 3px solid #00ff00;
    overflow: hidden;
}
"#
}
