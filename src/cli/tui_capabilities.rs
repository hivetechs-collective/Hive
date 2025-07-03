//! Terminal Capability Detection for Progressive Enhancement
//!
//! This module detects terminal capabilities and provides progressive enhancement
//! for the TUI, falling back gracefully for limited terminals.

use anyhow::Result;
use crossterm::terminal;
use std::env;

/// Terminal capabilities for progressive enhancement
#[derive(Debug, Clone)]
pub struct TuiCapabilities {
    pub supports_color: bool,
    pub supports_unicode: bool,
    pub supports_mouse: bool,
    pub supports_alternate_screen: bool,
    pub terminal_size: (u16, u16),
    pub color_depth: ColorDepth,
    pub terminal_type: TerminalType,
    pub supports_true_color: bool,
    pub supports_256_color: bool,
    pub supports_focus_events: bool,
}

/// Color depth support
#[derive(Debug, Clone, PartialEq)]
pub enum ColorDepth {
    Monochrome,     // No color support
    Basic8,         // 8 basic colors
    Extended256,    // 256 colors
    TrueColor,      // 24-bit true color
}

/// Terminal type detection
#[derive(Debug, Clone, PartialEq)]
pub enum TerminalType {
    Unknown,
    VT100,
    XTerm,
    WindowsTerminal,
    ITerm2,
    VSCodeTerminal,
    Kitty,
    Alacritty,
    GNOME,
    KDE,
}

/// TUI mode based on capabilities
#[derive(Debug, Clone, PartialEq)]
pub enum TuiMode {
    Disabled,       // TUI not supported
    Simple,         // Basic TUI without advanced features
    Enhanced,       // Full TUI with advanced features
}

impl TuiCapabilities {
    /// Detect terminal capabilities
    pub fn detect() -> Result<Self> {
        let terminal_size = terminal::size().unwrap_or((80, 24));
        let terminal_type = Self::detect_terminal_type();
        let supports_color = Self::detect_color_support();
        let color_depth = Self::detect_color_depth();
        let supports_unicode = Self::detect_unicode_support();
        let supports_mouse = Self::detect_mouse_support();
        let supports_alternate_screen = Self::detect_alternate_screen_support();
        let supports_true_color = color_depth == ColorDepth::TrueColor;
        let supports_256_color = matches!(color_depth, ColorDepth::Extended256 | ColorDepth::TrueColor);
        let supports_focus_events = Self::detect_focus_events_support();

        Ok(Self {
            supports_color,
            supports_unicode,
            supports_mouse,
            supports_alternate_screen,
            terminal_size,
            color_depth,
            terminal_type,
            supports_true_color,
            supports_256_color,
            supports_focus_events,
        })
    }

    /// Determine the best TUI mode based on capabilities
    pub fn determine_tui_mode(&self) -> TuiMode {
        // Check if TUI is explicitly disabled
        if env::var("HIVE_SIMPLE_CLI").is_ok() || env::var("NO_TUI").is_ok() {
            return TuiMode::Disabled;
        }

        // Check minimum requirements for any TUI
        if self.terminal_size.0 < 80 || self.terminal_size.1 < 24 {
            return TuiMode::Disabled;
        }

        // Check if we're in a pipe or redirect
        if !Self::is_interactive_terminal() {
            return TuiMode::Disabled;
        }

        // Determine enhanced vs simple mode
        if self.supports_enhanced_tui() {
            TuiMode::Enhanced
        } else if self.supports_simple_tui() {
            TuiMode::Simple
        } else {
            TuiMode::Disabled
        }
    }

    /// Check if terminal supports enhanced TUI features
    pub fn supports_enhanced_tui(&self) -> bool {
        self.terminal_size.0 >= 120 &&
        self.terminal_size.1 >= 30 &&
        self.supports_color &&
        self.supports_alternate_screen &&
        self.supports_unicode
    }

    /// Check if terminal supports simple TUI features
    pub fn supports_simple_tui(&self) -> bool {
        self.terminal_size.0 >= 80 &&
        self.terminal_size.1 >= 24 &&
        self.supports_alternate_screen
    }

    /// Get recommended layout constraints based on capabilities
    pub fn get_layout_constraints(&self) -> LayoutConstraints {
        match self.determine_tui_mode() {
            TuiMode::Enhanced => LayoutConstraints {
                min_width: 120,
                min_height: 30,
                explorer_width: 25,
                consensus_width: 30,
                terminal_height: 25,
                supports_panels: true,
                supports_tabs: true,
                supports_syntax_highlighting: self.supports_256_color,
            },
            TuiMode::Simple => LayoutConstraints {
                min_width: 80,
                min_height: 24,
                explorer_width: 0,  // No explorer in simple mode
                consensus_width: 0, // No separate consensus panel
                terminal_height: 0, // No separate terminal
                supports_panels: false,
                supports_tabs: false,
                supports_syntax_highlighting: false,
            },
            TuiMode::Disabled => LayoutConstraints::minimal(),
        }
    }

    // Detection methods
    fn detect_terminal_type() -> TerminalType {
        // Check environment variables for terminal identification
        if let Ok(term_program) = env::var("TERM_PROGRAM") {
            match term_program.as_str() {
                "vscode" => return TerminalType::VSCodeTerminal,
                "iTerm.app" => return TerminalType::ITerm2,
                "WezTerm" => return TerminalType::ITerm2, // Similar capabilities
                _ => {}
            }
        }

        if let Ok(term) = env::var("TERM") {
            match term.as_str() {
                t if t.starts_with("xterm") => TerminalType::XTerm,
                "screen" | "tmux" => TerminalType::XTerm, // Usually xterm-compatible
                "kitty" => TerminalType::Kitty,
                "alacritty" => TerminalType::Alacritty,
                "vt100" => TerminalType::VT100,
                _ => TerminalType::Unknown,
            }
        } else if env::var("WT_SESSION").is_ok() {
            TerminalType::WindowsTerminal
        } else {
            TerminalType::Unknown
        }
    }

    fn detect_color_support() -> bool {
        // Check for explicit color disable
        if env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Check COLORTERM for true color support
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return true;
            }
        }

        // Check TERM for color capabilities
        if let Ok(term) = env::var("TERM") {
            return term.contains("color") || 
                   term.contains("256") ||
                   term.starts_with("xterm") ||
                   term == "screen" ||
                   term == "tmux";
        }

        // Default to false for safety
        false
    }

    fn detect_color_depth() -> ColorDepth {
        // Check for explicit true color support
        if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return ColorDepth::TrueColor;
            }
        }

        // Check TERM for color depth indicators
        if let Ok(term) = env::var("TERM") {
            if term.contains("256") {
                return ColorDepth::Extended256;
            } else if term.contains("color") {
                return ColorDepth::Basic8;
            }
        }

        // Check for common terminals with known capabilities
        match Self::detect_terminal_type() {
            TerminalType::ITerm2 |
            TerminalType::Kitty |
            TerminalType::Alacritty |
            TerminalType::VSCodeTerminal |
            TerminalType::WindowsTerminal => ColorDepth::TrueColor,
            TerminalType::XTerm => ColorDepth::Extended256,
            _ => {
                if Self::detect_color_support() {
                    ColorDepth::Basic8
                } else {
                    ColorDepth::Monochrome
                }
            }
        }
    }

    fn detect_unicode_support() -> bool {
        // Check locale for UTF-8
        if let Ok(lang) = env::var("LANG") {
            return lang.to_uppercase().contains("UTF-8") || lang.to_uppercase().contains("UTF8");
        }

        if let Ok(lc_all) = env::var("LC_ALL") {
            return lc_all.to_uppercase().contains("UTF-8");
        }

        // Modern terminals generally support Unicode
        !matches!(Self::detect_terminal_type(), TerminalType::VT100 | TerminalType::Unknown)
    }

    fn detect_mouse_support() -> bool {
        // Most modern terminals support mouse
        !matches!(Self::detect_terminal_type(), TerminalType::VT100)
    }

    fn detect_alternate_screen_support() -> bool {
        // Most terminals support alternate screen
        !matches!(Self::detect_terminal_type(), TerminalType::VT100)
    }

    fn detect_focus_events_support() -> bool {
        // Focus events are supported by most modern terminals
        matches!(
            Self::detect_terminal_type(),
            TerminalType::XTerm |
            TerminalType::ITerm2 |
            TerminalType::Kitty |
            TerminalType::Alacritty |
            TerminalType::VSCodeTerminal |
            TerminalType::WindowsTerminal
        )
    }

    fn is_interactive_terminal() -> bool {
        // Use atty crate for cross-platform terminal detection
        atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout)
    }
}

/// Layout constraints based on terminal capabilities
#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub min_width: u16,
    pub min_height: u16,
    pub explorer_width: u16,     // Percentage
    pub consensus_width: u16,    // Percentage
    pub terminal_height: u16,    // Percentage
    pub supports_panels: bool,
    pub supports_tabs: bool,
    pub supports_syntax_highlighting: bool,
}

impl LayoutConstraints {
    pub fn minimal() -> Self {
        Self {
            min_width: 0,
            min_height: 0,
            explorer_width: 0,
            consensus_width: 0,
            terminal_height: 0,
            supports_panels: false,
            supports_tabs: false,
            supports_syntax_highlighting: false,
        }
    }
}

/// TUI launcher with capability detection
pub struct TuiLauncher;

impl TuiLauncher {
    /// Check if TUI should be launched
    pub async fn should_launch_tui() -> Result<TuiMode> {
        let capabilities = TuiCapabilities::detect()?;
        Ok(capabilities.determine_tui_mode())
    }

    /// Get capabilities for TUI configuration
    pub async fn get_capabilities() -> Result<TuiCapabilities> {
        TuiCapabilities::detect()
    }

    /// Check if current environment supports enhanced TUI
    pub async fn supports_enhanced_tui() -> Result<bool> {
        let capabilities = TuiCapabilities::detect()?;
        Ok(capabilities.supports_enhanced_tui())
    }

    /// Check if current environment supports any TUI
    pub async fn supports_any_tui() -> Result<bool> {
        let mode = Self::should_launch_tui().await?;
        Ok(mode != TuiMode::Disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_detection() {
        let capabilities = TuiCapabilities::detect().unwrap();
        
        // Basic checks that should always work
        assert!(capabilities.terminal_size.0 > 0);
        assert!(capabilities.terminal_size.1 > 0);
        
        // The mode should be deterministic
        let mode1 = capabilities.determine_tui_mode();
        let mode2 = capabilities.determine_tui_mode();
        assert_eq!(mode1, mode2);
    }

    #[test]
    fn test_layout_constraints() {
        let capabilities = TuiCapabilities::detect().unwrap();
        let constraints = capabilities.get_layout_constraints();
        
        // Constraints should be reasonable
        assert!(constraints.min_width <= 120);
        assert!(constraints.min_height <= 50);
    }
}