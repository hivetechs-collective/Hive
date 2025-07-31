//! Git Decoration Styles
//!
//! CSS styles for git decorations integrated into the component system

use dioxus::prelude::*;

/// Get the CSS styles for git decorations
pub fn get_git_decoration_styles() -> &'static str {
    include_str!("decoration_styles.css")
}

/// Component that injects git decoration styles
#[component]
pub fn GitDecorationStyles() -> Element {
    rsx! {
        style {
            dangerous_inner_html: "{get_git_decoration_styles()}"
        }
    }
}

/// CSS variables for git decoration colors that can be customized
pub fn get_git_decoration_css_variables(config: &crate::desktop::git::decorations::GitDecorationConfig) -> String {
    let styles = &config.styles;
    
    format!(
        r#"
        :root {{
            --git-modified-color: {modified};
            --git-added-color: {added};
            --git-deleted-color: {deleted};
            --git-untracked-color: {untracked};
            --git-ignored-color: {ignored};
            --git-conflict-color: {conflict};
            --git-renamed-color: {renamed};
            --git-copied-color: {copied};
            --git-decoration-opacity: {opacity};
            --git-status-font-weight: {font_weight};
        }}
        "#,
        modified = styles.modified_color,
        added = styles.added_color,
        deleted = styles.deleted_color,
        untracked = styles.untracked_color,
        ignored = styles.ignored_color,
        conflict = styles.conflict_color,
        renamed = styles.renamed_color,
        copied = styles.copied_color,
        opacity = styles.opacity,
        font_weight = styles.status_font_weight,
    )
}

/// Component that injects dynamic CSS variables based on configuration
#[component]
pub fn GitDecorationDynamicStyles(
    config: crate::desktop::git::decorations::GitDecorationConfig
) -> Element {
    let css_variables = get_git_decoration_css_variables(&config);
    
    rsx! {
        style {
            dangerous_inner_html: "{css_variables}"
        }
    }
}

/// Utility function to get inline styles for a specific git status
pub fn get_inline_git_status_style(
    status: &crate::desktop::state::GitFileStatus,
    config: &crate::desktop::git::decorations::GitDecorationConfig,
) -> String {
    if !config.enabled || !config.show_colors {
        return String::new();
    }

    let color = match status {
        crate::desktop::state::GitFileStatus::Modified => &config.styles.modified_color,
        crate::desktop::state::GitFileStatus::Added => &config.styles.added_color,
        crate::desktop::state::GitFileStatus::Deleted => &config.styles.deleted_color,
        crate::desktop::state::GitFileStatus::Untracked => &config.styles.untracked_color,
        crate::desktop::state::GitFileStatus::Renamed => &config.styles.renamed_color,
        crate::desktop::state::GitFileStatus::Copied => &config.styles.copied_color,
        crate::desktop::state::GitFileStatus::Ignored => &config.styles.ignored_color,
    };

    format!(
        "color: {}; opacity: {}; font-weight: {};",
        color,
        config.styles.opacity,
        config.styles.status_font_weight
    )
}

/// Get CSS class name for a git status
pub fn get_git_status_class(status: &crate::desktop::state::GitFileStatus) -> &'static str {
    match status {
        crate::desktop::state::GitFileStatus::Modified => "git-modified",
        crate::desktop::state::GitFileStatus::Added => "git-added",
        crate::desktop::state::GitFileStatus::Deleted => "git-deleted",
        crate::desktop::state::GitFileStatus::Untracked => "git-untracked",
        crate::desktop::state::GitFileStatus::Renamed => "git-renamed",
        crate::desktop::state::GitFileStatus::Copied => "git-copied",
        crate::desktop::state::GitFileStatus::Ignored => "git-ignored",
    }
}

/// Get conflict-specific CSS class
pub fn get_conflict_class(is_conflicted: bool) -> &'static str {
    if is_conflicted {
        "git-conflict"
    } else {
        ""
    }
}

/// Generate complete CSS class string for a file with git status
pub fn get_file_git_classes(
    status: Option<&crate::desktop::state::GitFileStatus>,
    is_conflicted: bool,
) -> String {
    let mut classes = Vec::new();

    if let Some(status) = status {
        classes.push(get_git_status_class(status));
    }

    if is_conflicted {
        classes.push(get_conflict_class(true));
    }

    classes.join(" ")
}

/// VS Code theme integration
pub fn get_vscode_git_theme_styles() -> &'static str {
    r#"
    /* VS Code theme integration for git decorations */
    
    /* Light theme */
    [data-vscode-theme-kind="vscode-light"] {
        --git-modified-color: #e2c08d;
        --git-added-color: #73c991;
        --git-deleted-color: #f48771;
        --git-untracked-color: #73c991;
        --git-ignored-color: #8c8c8c;
        --git-conflict-color: #f44747;
        --git-renamed-color: #75beff;
        --git-copied-color: #73c991;
    }
    
    /* Dark theme */
    [data-vscode-theme-kind="vscode-dark"] {
        --git-modified-color: #e2c08d;
        --git-added-color: #73c991;
        --git-deleted-color: #f48771;
        --git-untracked-color: #73c991;
        --git-ignored-color: #6b6b6b;
        --git-conflict-color: #f44747;
        --git-renamed-color: #75beff;
        --git-copied-color: #73c991;
    }
    
    /* High contrast theme */
    [data-vscode-theme-kind="vscode-high-contrast"] {
        --git-modified-color: #ffff00;
        --git-added-color: #00ff00;
        --git-deleted-color: #ff0000;
        --git-untracked-color: #00ff00;
        --git-ignored-color: #808080;
        --git-conflict-color: #ff0000;
        --git-renamed-color: #0080ff;
        --git-copied-color: #00ff00;
    }
    "#
}

/// Component for VS Code theme integration
#[component]
pub fn VSCodeGitThemeStyles() -> Element {
    rsx! {
        style {
            dangerous_inner_html: "{get_vscode_git_theme_styles()}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::{git::decorations::GitDecorationConfig, state::GitFileStatus};

    #[test]
    fn test_git_status_class() {
        assert_eq!(get_git_status_class(&GitFileStatus::Modified), "git-modified");
        assert_eq!(get_git_status_class(&GitFileStatus::Added), "git-added");
        assert_eq!(get_git_status_class(&GitFileStatus::Deleted), "git-deleted");
    }

    #[test]
    fn test_conflict_class() {
        assert_eq!(get_conflict_class(true), "git-conflict");
        assert_eq!(get_conflict_class(false), "");
    }

    #[test]
    fn test_file_git_classes() {
        let classes = get_file_git_classes(Some(&GitFileStatus::Modified), true);
        assert!(classes.contains("git-modified"));
        assert!(classes.contains("git-conflict"));
    }

    #[test]
    fn test_inline_style_generation() {
        let config = GitDecorationConfig::default();
        let style = get_inline_git_status_style(&GitFileStatus::Modified, &config);
        assert!(style.contains("color:"));
        assert!(style.contains("opacity:"));
        assert!(style.contains("font-weight:"));
    }

    #[test]
    fn test_css_variables_generation() {
        let config = GitDecorationConfig::default();
        let css = get_git_decoration_css_variables(&config);
        assert!(css.contains("--git-modified-color:"));
        assert!(css.contains("--git-added-color:"));
        assert!(css.contains("--git-decoration-opacity:"));
    }
}