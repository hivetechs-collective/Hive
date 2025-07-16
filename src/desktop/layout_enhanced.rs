//! Enhanced VS Code-style Layout Management
//! Based on VS Code's src/vs/workbench/browser/layout.ts

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Panel visibility state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PanelVisibility {
    pub activity_bar: bool,
    pub side_bar: bool,
    pub panel: bool,
    pub status_bar: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            activity_bar: true,
            side_bar: true,
            panel: false,
            status_bar: true,
        }
    }
}

/// Layout dimensions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutDimensions {
    pub activity_bar_width: f32,
    pub side_bar_width: f32,
    pub side_bar_position: SideBarPosition,
    pub panel_height: f32,
    pub panel_position: PanelPosition,
}

impl Default for LayoutDimensions {
    fn default() -> Self {
        Self {
            activity_bar_width: 48.0,
            side_bar_width: 300.0,
            side_bar_position: SideBarPosition::Left,
            panel_height: 200.0,
            panel_position: PanelPosition::Bottom,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SideBarPosition {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PanelPosition {
    Bottom,
    Right,
}

/// Layout state management
#[derive(Clone, Debug)]
pub struct LayoutState {
    pub visibility: PanelVisibility,
    pub dimensions: LayoutDimensions,
    pub is_zen_mode: bool,
    pub is_fullscreen: bool,
    pub sidebar_content: SidebarContent,
    pub panel_content: PanelContent,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SidebarContent {
    Explorer,
    Search,
    SourceControl,
    Debug,
    Extensions,
    Consensus,
    Models,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PanelContent {
    Terminal,
    Problems,
    Output,
    DebugConsole,
    ConsensusProgress,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            visibility: PanelVisibility::default(),
            dimensions: LayoutDimensions::default(),
            is_zen_mode: false,
            is_fullscreen: false,
            sidebar_content: SidebarContent::Explorer,
            panel_content: PanelContent::Terminal,
        }
    }
}

impl LayoutState {
    /// Toggle zen mode (hides all UI except editor)
    pub fn toggle_zen_mode(&mut self) {
        self.is_zen_mode = !self.is_zen_mode;
        if self.is_zen_mode {
            self.visibility.activity_bar = false;
            self.visibility.side_bar = false;
            self.visibility.panel = false;
            self.visibility.status_bar = false;
        } else {
            self.visibility = PanelVisibility::default();
        }
    }
    
    /// Toggle sidebar visibility (Ctrl+B)
    pub fn toggle_sidebar(&mut self) {
        self.visibility.side_bar = !self.visibility.side_bar;
    }
    
    /// Toggle panel visibility (Ctrl+J)
    pub fn toggle_panel(&mut self) {
        self.visibility.panel = !self.visibility.panel;
    }
    
    /// Save layout to localStorage
    pub fn save_to_storage(&self) {
        if let Ok(json) = serde_json::to_string(&(
            &self.visibility,
            &self.dimensions,
        )) {
            // In a real implementation, save to localStorage
            tracing::debug!("Saving layout: {}", json);
        }
    }
    
    /// Load layout from localStorage
    pub fn load_from_storage(&mut self) {
        // In a real implementation, load from localStorage
        tracing::debug!("Loading layout from storage");
    }
}

/// Resizable splitter component
#[component]
pub fn Splitter(
    direction: SplitterDirection,
    on_resize: EventHandler<f32>,
    min_size: f32,
    max_size: f32,
) -> Element {
    let is_dragging = use_signal(|| false);
    let start_pos = use_signal(|| 0.0);
    let start_size = use_signal(|| 0.0);
    
    let splitter_class = match direction {
        SplitterDirection::Vertical => "splitter vertical",
        SplitterDirection::Horizontal => "splitter horizontal",
    };
    
    rsx! {
        div {
            class: "{splitter_class}",
            class: if *is_dragging.read() { "dragging" } else { "" },
            onmousedown: move |e| {
                is_dragging.set(true);
                match direction {
                    SplitterDirection::Vertical => {
                        start_pos.set(e.client_coordinates().x as f32);
                    },
                    SplitterDirection::Horizontal => {
                        start_pos.set(e.client_coordinates().y as f32);
                    }
                }
                start_size.set(0.0); // Would get current size from parent
            },
            onmousemove: move |e| {
                if *is_dragging.read() {
                    let delta = match direction {
                        SplitterDirection::Vertical => {
                            e.client_coordinates().x as f32 - *start_pos.read()
                        },
                        SplitterDirection::Horizontal => {
                            e.client_coordinates().y as f32 - *start_pos.read()
                        }
                    };
                    
                    let new_size = (*start_size.read() + delta).clamp(min_size, max_size);
                    on_resize.call(new_size);
                }
            },
            onmouseup: move |_| {
                is_dragging.set(false);
            },
            onmouseleave: move |_| {
                is_dragging.set(false);
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SplitterDirection {
    Vertical,
    Horizontal,
}

/// Main layout wrapper component
#[component]
pub fn VSCodeLayout(
    state: Signal<LayoutState>,
    activity_bar: Element,
    sidebar: Element,
    editor: Element,
    panel: Element,
    status_bar: Element,
) -> Element {
    let layout = state.read();
    
    // Calculate layout styles
    let container_class = if layout.is_zen_mode {
        "vscode-layout zen-mode"
    } else {
        "vscode-layout"
    };
    
    let sidebar_style = format!(
        "width: {}px; {}",
        layout.dimensions.side_bar_width,
        if layout.dimensions.side_bar_position == SideBarPosition::Right {
            "order: 3;"
        } else {
            "order: 1;"
        }
    );
    
    let panel_style = format!(
        "height: {}px;",
        layout.dimensions.panel_height
    );
    
    rsx! {
        div {
            class: "{container_class}",
            
            // Main horizontal layout
            div {
                class: "layout-main",
                
                // Activity Bar
                if layout.visibility.activity_bar {
                    div {
                        class: "layout-activity-bar",
                        {activity_bar}
                    }
                }
                
                // Sidebar with splitter
                if layout.visibility.side_bar {
                    div {
                        class: "layout-sidebar-container",
                        style: "{sidebar_style}",
                        
                        div {
                            class: "layout-sidebar",
                            {sidebar}
                        }
                        
                        Splitter {
                            direction: SplitterDirection::Vertical,
                            on_resize: move |size| {
                                state.write().dimensions.side_bar_width = size;
                                state.write().save_to_storage();
                            },
                            min_size: 170.0,
                            max_size: 600.0,
                        }
                    }
                }
                
                // Editor area (includes panel if position is right)
                div {
                    class: "layout-editor-container",
                    style: "order: 2; flex: 1;",
                    
                    // Editor
                    div {
                        class: "layout-editor",
                        {editor}
                    }
                    
                    // Panel (if bottom position)
                    if layout.visibility.panel && layout.dimensions.panel_position == PanelPosition::Bottom {
                        Splitter {
                            direction: SplitterDirection::Horizontal,
                            on_resize: move |size| {
                                state.write().dimensions.panel_height = size;
                                state.write().save_to_storage();
                            },
                            min_size: 100.0,
                            max_size: 400.0,
                        }
                        
                        div {
                            class: "layout-panel",
                            style: "{panel_style}",
                            {panel}
                        }
                    }
                }
            }
            
            // Status Bar
            if layout.visibility.status_bar {
                div {
                    class: "layout-status-bar",
                    {status_bar}
                }
            }
        }
    }
}

/// Layout CSS styles
pub const LAYOUT_STYLES: &str = r#"
/* Main layout container */
.vscode-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background-color: var(--vscode-editor-background);
}

.vscode-layout.zen-mode .layout-activity-bar,
.vscode-layout.zen-mode .layout-sidebar-container,
.vscode-layout.zen-mode .layout-panel,
.vscode-layout.zen-mode .layout-status-bar {
    display: none !important;
}

/* Main horizontal layout */
.layout-main {
    display: flex;
    flex: 1;
    overflow: hidden;
}

/* Activity Bar */
.layout-activity-bar {
    flex-shrink: 0;
    width: 48px;
}

/* Sidebar container */
.layout-sidebar-container {
    display: flex;
    flex-shrink: 0;
    position: relative;
}

.layout-sidebar {
    flex: 1;
    overflow: hidden;
}

/* Editor container */
.layout-editor-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    overflow: hidden;
}

.layout-editor {
    flex: 1;
    overflow: hidden;
}

/* Panel */
.layout-panel {
    flex-shrink: 0;
    overflow: hidden;
    border-top: 1px solid var(--vscode-panel-border);
}

/* Status Bar */
.layout-status-bar {
    flex-shrink: 0;
    height: 22px;
}

/* Splitters */
.splitter {
    position: relative;
    background-color: var(--vscode-sash-hoverBorder);
    opacity: 0;
    transition: opacity 0.1s ease;
    z-index: 10;
}

.splitter:hover,
.splitter.dragging {
    opacity: 1;
}

.splitter.vertical {
    width: 4px;
    height: 100%;
    cursor: ew-resize;
}

.splitter.horizontal {
    width: 100%;
    height: 4px;
    cursor: ns-resize;
}

.splitter.dragging {
    background-color: var(--hivetechs-yellow);
}

/* Responsive breakpoints */
@media (max-width: 768px) {
    .layout-sidebar-container {
        position: absolute;
        left: 0;
        top: 0;
        bottom: 0;
        z-index: 100;
        box-shadow: 2px 0 8px rgba(0, 0, 0, 0.3);
    }
    
    .layout-sidebar-container:not(.mobile-visible) {
        transform: translateX(-100%);
    }
}

/* Animations */
.layout-sidebar-container,
.layout-panel {
    transition: width 0.2s ease, height 0.2s ease;
}

/* Focus trap for accessibility */
.layout-focus-trap {
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

/* Layout overlay for drag operations */
.layout-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
    cursor: inherit;
    display: none;
}

.splitter.dragging ~ .layout-overlay {
    display: block;
}
"#;