//! Reusable menu component with keyboard navigation support
//!
//! This module provides a flexible menu system that can be used for dropdown menus,
//! context menus, and command palettes with full keyboard navigation.

use dioxus::prelude::*;
use std::fmt::Debug;

/// Menu item trait for defining menu entries
pub trait MenuItem: Clone + Debug + PartialEq + 'static {
    /// Get the display label for the menu item
    fn label(&self) -> &str;

    /// Check if the menu item is enabled
    fn enabled(&self) -> bool {
        true
    }

    /// Get the keyboard shortcut for the menu item
    fn shortcut(&self) -> Option<&str> {
        None
    }

    /// Check if this is a separator item
    fn is_separator(&self) -> bool {
        false
    }

    /// Get the icon for the menu item
    fn icon(&self) -> Option<&str> {
        None
    }

    /// Check if the menu item has a submenu
    fn has_submenu(&self) -> bool {
        false
    }
}

/// Generic menu item implementation
#[derive(Clone, Debug, PartialEq)]
pub struct GenericMenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub shortcut: Option<String>,
    pub icon: Option<String>,
    pub is_separator: bool,
    pub has_submenu: bool,
}

impl MenuItem for GenericMenuItem {
    fn label(&self) -> &str {
        &self.label
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn shortcut(&self) -> Option<&str> {
        self.shortcut.as_deref()
    }

    fn is_separator(&self) -> bool {
        self.is_separator
    }

    fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    fn has_submenu(&self) -> bool {
        self.has_submenu
    }
}

/// Menu position for dropdown/context menus
#[derive(Clone, Debug, PartialEq)]
pub struct MenuPosition {
    pub x: f64,
    pub y: f64,
}

/// Menu props for the reusable menu component
#[derive(Props, Clone, PartialEq)]
pub struct MenuProps<T: MenuItem> {
    /// List of menu items
    pub items: Vec<T>,

    /// Callback when an item is selected
    pub on_select: EventHandler<T>,

    /// Callback when the menu should close
    #[props(default = EventHandler::new(|_| {}))]
    pub on_close: EventHandler<()>,

    /// Position for dropdown/context menus
    #[props(optional)]
    pub position: Option<MenuPosition>,

    /// Whether the menu is visible
    #[props(default = true)]
    pub visible: bool,

    /// Custom CSS class for styling
    #[props(default = "menu".to_string())]
    pub class: String,

    /// Whether to show keyboard shortcuts
    #[props(default = true)]
    pub show_shortcuts: bool,

    /// Whether to show icons
    #[props(default = true)]
    pub show_icons: bool,

    /// Maximum height before scrolling
    #[props(optional)]
    pub max_height: Option<f64>,

    /// Width of the menu
    #[props(default = 200.0)]
    pub width: f64,
}

/// Reusable menu component with keyboard navigation
#[component]
pub fn Menu<T: MenuItem>(props: MenuProps<T>) -> Element {
    let mut selected_index = use_signal(|| None::<usize>);
    let menu_ref = use_node_ref();

    // Find the next selectable item
    let find_next_item = |current: Option<usize>, items: &[T], forward: bool| -> Option<usize> {
        let len = items.len();
        if len == 0 {
            return None;
        }

        let start = current
            .map(|i| if forward { i + 1 } else { i.wrapping_sub(1) })
            .unwrap_or(0);

        for i in 0..len {
            let idx = if forward {
                (start + i) % len
            } else {
                (start + len - i) % len
            };

            if !items[idx].is_separator() && items[idx].enabled() {
                return Some(idx);
            }
        }

        None
    };

    // Handle keyboard navigation
    let keyboard_handler = move |evt: KeyboardEvent| match evt.key().as_str() {
        "ArrowDown" => {
            evt.prevent_default();
            let next = find_next_item(selected_index(), &props.items, true);
            selected_index.set(next);
        }
        "ArrowUp" => {
            evt.prevent_default();
            let next = find_next_item(selected_index(), &props.items, false);
            selected_index.set(next);
        }
        "Enter" | " " => {
            evt.prevent_default();
            if let Some(idx) = selected_index() {
                if let Some(item) = props.items.get(idx) {
                    if item.enabled() && !item.is_separator() {
                        props.on_select.call(item.clone());
                    }
                }
            }
        }
        "Escape" => {
            evt.prevent_default();
            props.on_close.call(());
        }
        "Home" => {
            evt.prevent_default();
            let next = find_next_item(None, &props.items, true);
            selected_index.set(next);
        }
        "End" => {
            evt.prevent_default();
            let next = find_next_item(Some(props.items.len()), &props.items, false);
            selected_index.set(next);
        }
        _ => {}
    };

    // Focus the menu when it becomes visible
    use_effect(move || {
        if props.visible {
            if let Some(element) = menu_ref.get() {
                element.focus();
            }
        }
    });

    if !props.visible {
        return rsx! { div {} };
    }

    let style = if let Some(pos) = &props.position {
        format!(
            "position: fixed; left: {}px; top: {}px; width: {}px; {}",
            pos.x,
            pos.y,
            props.width,
            props
                .max_height
                .map(|h| format!("max-height: {}px; overflow-y: auto;", h))
                .unwrap_or_default()
        )
    } else {
        format!(
            "width: {}px; {}",
            props.width,
            props
                .max_height
                .map(|h| format!("max-height: {}px; overflow-y: auto;", h))
                .unwrap_or_default()
        )
    };

    rsx! {
        div {
            class: "{props.class}",
            style: "{style}",
            tabindex: "0",
            node_ref: menu_ref,
            onkeydown: keyboard_handler,
            onmouseleave: move |_| selected_index.set(None),

            for (idx, item) in props.items.iter().enumerate() {
                if item.is_separator() {
                    div { class: "menu-separator" }
                } else {
                    div {
                        class: format!(
                            "menu-item {} {}",
                            if !item.enabled() { "disabled" } else { "" },
                            if selected_index() == Some(idx) { "selected" } else { "" }
                        ),
                        onmouseenter: move |_| {
                            if item.enabled() {
                                selected_index.set(Some(idx));
                            }
                        },
                        onclick: move |evt| {
                            evt.prevent_default();
                            if item.enabled() {
                                props.on_select.call(item.clone());
                            }
                        },

                        // Icon
                        if props.show_icons {
                            div {
                                class: "menu-item-icon",
                                if let Some(icon) = item.icon() {
                                    "{icon}"
                                } else {
                                    div { class: "menu-item-icon-placeholder" }
                                }
                            }
                        }

                        // Label
                        div {
                            class: "menu-item-label",
                            "{item.label()}"
                        }

                        // Submenu indicator or shortcut
                        div {
                            class: "menu-item-right",
                            if item.has_submenu() {
                                "▶"
                            } else if props.show_shortcuts {
                                if let Some(shortcut) = item.shortcut() {
                                    span { class: "menu-item-shortcut", "{shortcut}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Builder for creating menu items
pub struct MenuItemBuilder {
    id: String,
    label: String,
    enabled: bool,
    shortcut: Option<String>,
    icon: Option<String>,
    is_separator: bool,
    has_submenu: bool,
}

impl MenuItemBuilder {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            id: label.clone(),
            label,
            enabled: true,
            shortcut: None,
            icon: None,
            is_separator: false,
            has_submenu: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: "separator".to_string(),
            label: String::new(),
            enabled: false,
            shortcut: None,
            icon: None,
            is_separator: true,
            has_submenu: false,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn submenu(mut self) -> Self {
        self.has_submenu = true;
        self
    }

    pub fn build(self) -> GenericMenuItem {
        GenericMenuItem {
            id: self.id,
            label: self.label,
            enabled: self.enabled,
            shortcut: self.shortcut,
            icon: self.icon,
            is_separator: self.is_separator,
            has_submenu: self.has_submenu,
        }
    }
}
