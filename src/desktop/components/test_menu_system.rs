//! Test file for the menu system components

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use crate::desktop::components::{
        menu::{Menu, MenuProps, MenuItem, GenericMenuItem, MenuItemBuilder},
        tooltip::{Tooltip, TooltipProps, TooltipPosition, TooltipTrigger},
        progress_indicator::{
            ProgressIndicator, ProgressIndicatorProps, ProgressType, 
            ProgressSize, ProgressColor, Spinner, LoadingDots
        },
    };
    use dioxus::prelude::*;

    #[test]
    fn test_menu_item_builder() {
        let item = MenuItemBuilder::new("File")
            .id("file-menu")
            .shortcut("Ctrl+F")
            .icon("📁")
            .build();

        assert_eq!(item.label(), "File");
        assert_eq!(item.shortcut(), Some("Ctrl+F"));
        assert_eq!(item.icon(), Some("📁"));
        assert!(item.enabled());
        assert!(!item.is_separator());
    }

    #[test]
    fn test_menu_separator() {
        let separator = MenuItemBuilder::separator().build();
        assert!(separator.is_separator());
        assert!(!separator.enabled());
    }

    #[test]
    fn test_progress_types() {
        assert_eq!(ProgressType::default(), ProgressType::Linear);
        assert_eq!(ProgressSize::default(), ProgressSize::Medium);
        assert_eq!(ProgressColor::default(), ProgressColor::Primary);
    }

    #[test]
    fn test_tooltip_defaults() {
        assert_eq!(TooltipPosition::default(), TooltipPosition::Auto);
        assert_eq!(TooltipTrigger::default(), TooltipTrigger::HoverOrFocus);
    }

    #[test]
    fn test_progress_size_pixels() {
        assert_eq!(ProgressSize::Small.to_pixels(), 16.0);
        assert_eq!(ProgressSize::Medium.to_pixels(), 24.0);
        assert_eq!(ProgressSize::Large.to_pixels(), 32.0);
        assert_eq!(ProgressSize::Custom(48.0).to_pixels(), 48.0);
    }

    #[test]
    fn test_progress_color_css() {
        assert_eq!(ProgressColor::Primary.to_css_var(), "var(--color-primary)");
        assert_eq!(ProgressColor::Success.to_css_var(), "var(--color-success)");
        assert_eq!(ProgressColor::Custom("#FF0000".to_string()).to_css_var(), "#FF0000");
    }
}