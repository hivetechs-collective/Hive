//! VS Code Codicon-style Git icons as SVG strings
//! Icons are designed to work with both light and dark themes

use crate::desktop::styles::theme::Theme;

/// Git-related SVG icons from VS Code Codicon set
pub struct GitIcons;

impl GitIcons {
    /// Git branch icon
    pub fn branch(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M11.677 3.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zm0 1a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5zM5.5 7a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3zM2 8.5a3.5 3.5 0 1 1 7 0v5a.5.5 0 0 1-.5.5h-1a.5.5 0 0 1-.5-.5V12H5v1.5a.5.5 0 0 1-.5.5h-1a.5.5 0 0 1-.5.5v-5z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M11.677 3.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3zm0 1a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5zM5.5 7a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3zM2 8.5a3.5 3.5 0 1 1 7 0v5a.5.5 0 0 1-.5.5h-1a.5.5 0 0 1-.5.5H4a.5.5 0 0 1-.5.5v-5z"/></svg>"##
            }
        }
    }

    /// Sync icon (circular arrows)
    pub fn sync(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M2.5 7.775V9.85a.5.5 0 0 1-.5.5H.5a.5.5 0 0 1-.5-.5V4.4a.5.5 0 0 1 .5-.5h5.45a.5.5 0 0 1 .5.5v1.5a.5.5 0 0 1-.5.5h-2.65c.524.716 1.36 1.175 2.3 1.175a2.447 2.447 0 0 0 2.012-1.065l1.37.83A3.987 3.987 0 0 1 5.65 9.1a3.968 3.968 0 0 1-3.15-1.325zM13.5 7.775V5.7a.5.5 0 0 1 .5-.5h1.5a.5.5 0 0 1 .5.5v5.45a.5.5 0 0 1-.5.5h-5.45a.5.5 0 0 1-.5-.5v-1.5a.5.5 0 0 1 .5-.5h2.65A2.447 2.447 0 0 0 10.35 8a2.447 2.447 0 0 0-2.012 1.065l-1.37-.83A3.987 3.987 0 0 1 10.35 6.475c1.295 0 2.426.612 3.15 1.3z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M2.5 7.775V9.85a.5.5 0 0 1-.5.5H.5a.5.5 0 0 1-.5-.5V4.4a.5.5 0 0 1 .5-.5h5.45a.5.5 0 0 1 .5.5v1.5a.5.5 0 0 1-.5.5h-2.65c.524.716 1.36 1.175 2.3 1.175a2.447 2.447 0 0 0 2.012-1.065l1.37.83A3.987 3.987 0 0 1 5.65 9.1a3.968 3.968 0 0 1-3.15-1.325zM13.5 7.775V5.7a.5.5 0 0 1 .5-.5h1.5a.5.5 0 0 1 .5.5v5.45a.5.5 0 0 1-.5.5h-5.45a.5.5 0 0 1-.5-.5v-1.5a.5.5 0 0 1 .5-.5h2.65A2.447 2.447 0 0 0 10.35 8a2.447 2.447 0 0 0-2.012 1.065l-1.37-.83A3.987 3.987 0 0 1 10.35 6.475c1.295 0 2.426.612 3.15 1.3z"/></svg>"##
            }
        }
    }

    /// Sync spin icon (rotating sync)
    pub fn sync_spin(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M13.915 7.5a5.5 5.5 0 0 1-11 0 5.5 5.5 0 0 1 11 0zm-6.61 2.468A2.5 2.5 0 0 0 10.5 7.5h1a3.5 3.5 0 0 1-4.486 3.357l.291-.889zm1.516-4.936A2.5 2.5 0 0 0 5.5 7.5h-1a3.5 3.5 0 0 1 4.486-3.357l-.291.889z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M13.915 7.5a5.5 5.5 0 0 1-11 0 5.5 5.5 0 0 1 11 0zm-6.61 2.468A2.5 2.5 0 0 0 10.5 7.5h1a3.5 3.5 0 0 1-4.486 3.357l.291-.889zm1.516-4.936A2.5 2.5 0 0 0 5.5 7.5h-1a3.5 3.5 0 0 1 4.486-3.357l-.291.889z"/></svg>"##
            }
        }
    }

    /// Cloud upload icon
    pub fn cloud_upload(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M11.91 5a3.001 3.001 0 0 0-5.808-.238A4.5 4.5 0 0 0 2 9.5a4.49 4.49 0 0 0 4.49 4.49h6.01A3.5 3.5 0 0 0 16 10.5a3.5 3.5 0 0 0-4.09-5.5zM8.5 6.379V11.5a.5.5 0 0 1-1 0V6.379L5.854 8.025a.5.5 0 1 1-.708-.707l2.5-2.5a.5.5 0 0 1 .708 0l2.5 2.5a.5.5 0 0 1-.708.707L8.5 6.379z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M11.91 5a3.001 3.001 0 0 0-5.808-.238A4.5 4.5 0 0 0 2 9.5a4.49 4.49 0 0 0 4.49 4.49h6.01A3.5 3.5 0 0 0 16 10.5a3.5 3.5 0 0 0-4.09-5.5zM8.5 6.379V11.5a.5.5 0 0 1-1 0V6.379L5.854 8.025a.5.5 0 1 1-.708-.707l2.5-2.5a.5.5 0 0 1 .708 0l2.5 2.5a.5.5 0 0 1-.708.707L8.5 6.379z"/></svg>"##
            }
        }
    }

    /// Check icon
    pub fn check(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M14.431 3.323l-8.47 10-.79-.036-3.35-4.77.818-.574 2.978 4.24 8.051-9.506.764.646z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M14.431 3.323l-8.47 10-.79-.036-3.35-4.77.818-.574 2.978 4.24 8.051-9.506.764.646z"/></svg>"##
            }
        }
    }

    /// X (close) icon
    pub fn x(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M8 8.707l3.646 3.647.708-.707L8.707 8l3.647-3.646-.707-.708L8 7.293 4.354 3.646l-.707.708L7.293 8l-3.646 3.646.707.708L8 8.707z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M8 8.707l3.646 3.647.708-.707L8.707 8l3.647-3.646-.707-.708L8 7.293 4.354 3.646l-.707.708L7.293 8l-3.646 3.646.707.708L8 8.707z"/></svg>"##
            }
        }
    }

    /// Warning icon
    pub fn warning(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M7.56 1h.88l6.54 12.26-.44.74H1.44L1 13.26 7.56 1zM8 2.28L2.28 13H13.7L8 2.28zM8.625 12v-1h-1.25v1h1.25zm-1.25-2V6h1.25v4h-1.25z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M7.56 1h.88l6.54 12.26-.44.74H1.44L1 13.26 7.56 1zM8 2.28L2.28 13H13.7L8 2.28zM8.625 12v-1h-1.25v1h1.25zm-1.25-2V6h1.25v4h-1.25z"/></svg>"##
            }
        }
    }

    /// Refresh icon
    pub fn refresh(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M5.563 2.516A6.001 6.001 0 0 0 8 14a6 6 0 0 0 5.835-4.529l-.893-.37A5 5 0 1 1 6.016 2.516l.89-.375-.89.375c.477.154.911.394 1.29.694L5.354 5.163a.5.5 0 0 0 0 .707l.707.707a.5.5 0 0 0 .707 0l2.932-2.933c.026.034.053.067.081.1A5 5 0 0 1 13 8a.5.5 0 0 0 1 0 6 6 0 0 0-8.437-5.484z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M5.563 2.516A6.001 6.001 0 0 0 8 14a6 6 0 0 0 5.835-4.529l-.893-.37A5 5 0 1 1 6.016 2.516l.89-.375-.89.375c.477.154.911.394 1.29.694L5.354 5.163a.5.5 0 0 0 0 .707l.707.707a.5.5 0 0 0 .707 0l2.932-2.933c.026.034.053.067.081.1A5 5 0 0 1 13 8a.5.5 0 0 0 1 0 6 6 0 0 0-8.437-5.484z"/></svg>"##
            }
        }
    }

    /// Plus icon
    pub fn plus(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M14 7v1H8v6H7V8H1V7h6V1h1v6h6z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M14 7v1H8v6H7V8H1V7h6V1h1v6h6z"/></svg>"##
            }
        }
    }

    /// Minus icon
    pub fn minus(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M14 8v1H2V8h12z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M14 8v1H2V8h12z"/></svg>"##
            }
        }
    }

    /// History icon
    pub fn history(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M13.507 13.532a8 8 0 1 0 0-11.064l-.71.71a7 7 0 1 1 0 9.644l.71.71zM7.5 3v5.5l3.5 2.1.6-1L8.5 7.8V3h-1z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M13.507 13.532a8 8 0 1 0 0-11.064l-.71.71a7 7 0 1 1 0 9.644l.71.71zM7.5 3v5.5l3.5 2.1.6-1L8.5 7.8V3h-1z"/></svg>"##
            }
        }
    }

    /// Commit icon (dot)
    pub fn commit(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M11.748 8a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M11.748 8a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0z"/></svg>"##
            }
        }
    }

    /// Merge icon
    pub fn merge(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M5.5 3.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zM4 4.55a2.5 2.5 0 1 0-1 0v7.9a2.5 2.5 0 1 0 1 0V7.04c.322.094.658.142 1 .142a4.97 4.97 0 0 0 3.536-1.464l.707-.707A3.98 3.98 0 0 1 12.075 4c.358 0 .77.056 1.125.15v.4a2.5 2.5 0 1 0 1 0v-.4A4.95 4.95 0 0 0 12.075 3a4.98 4.98 0 0 0-3.535 1.465l-.707.707A3.978 3.978 0 0 1 5 6.142a2.978 2.978 0 0 1-1-.252V4.55zm-1 10.95a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3zm10.5-10a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M5.5 3.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zM4 4.55a2.5 2.5 0 1 0-1 0v7.9a2.5 2.5 0 1 0 1 0V7.04c.322.094.658.142 1 .142a4.97 4.97 0 0 0 3.536-1.464l.707-.707A3.98 3.98 0 0 1 12.075 4c.358 0 .77.056 1.125.15v.4a2.5 2.5 0 1 0 1 0v-.4A4.95 4.95 0 0 0 12.075 3a4.98 4.98 0 0 0-3.535 1.465l-.707.707A3.978 3.978 0 0 1 5 6.142a2.978 2.978 0 0 1-1-.252V4.55zm-1 10.95a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3zm10.5-10a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z"/></svg>"##
            }
        }
    }

    /// Pull request icon
    pub fn pull_request(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M5.5 3.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zM4 4.55a2.5 2.5 0 1 0-1 0v7.9a2.5 2.5 0 1 0 1 0v-7.9zM13.5 12.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zm-1.5-8.95a2.5 2.5 0 1 0 1 0v3.57l-2.06-2.06a.5.5 0 0 0-.708 0l-.353.354a.5.5 0 0 0 0 .707l3.48 3.48c.078.078.165.141.26.187a.5.5 0 0 0 .373 0 .502.502 0 0 0 .26-.186l3.48-3.48a.5.5 0 0 0 0-.708l-.353-.353a.5.5 0 0 0-.707 0L13 7.621V3.55zM3.5 15.5a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M5.5 3.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zM4 4.55a2.5 2.5 0 1 0-1 0v7.9a2.5 2.5 0 1 0 1 0v-7.9zM13.5 12.5a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0zm-1.5-8.95a2.5 2.5 0 1 0 1 0v3.57l-2.06-2.06a.5.5 0 0 0-.708 0l-.353.354a.5.5 0 0 0 0 .707l3.48 3.48c.078.078.165.141.26.187a.5.5 0 0 0 .373 0 .502.502 0 0 0 .26-.186l3.48-3.48a.5.5 0 0 0 0-.708l-.353-.353a.5.5 0 0 0-.707 0L13 7.621V3.55zM3.5 15.5a1.5 1.5 0 1 1 0-3 1.5 1.5 0 0 1 0 3z"/></svg>"##
            }
        }
    }

    /// Tag icon
    pub fn tag(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M13.224 2.475A3.118 3.118 0 0 0 10.623 1h-3.24L2.474 5.91a1.5 1.5 0 0 0 0 2.122l5.494 5.493a1.5 1.5 0 0 0 2.122 0l4.91-4.908v-3.24a3.118 3.118 0 0 0-1.475-2.6l-.3-.302zM10.5 5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M13.224 2.475A3.118 3.118 0 0 0 10.623 1h-3.24L2.474 5.91a1.5 1.5 0 0 0 0 2.122l5.494 5.493a1.5 1.5 0 0 0 2.122 0l4.91-4.908v-3.24a3.118 3.118 0 0 0-1.475-2.6l-.3-.302zM10.5 5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3z"/></svg>"##
            }
        }
    }

    /// Remote icon
    pub fn remote(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M12.904 9.57L8.928 5.596l3.976-3.976-.619-.62L8 5.286v.619l4.285 4.285.62-.619zM3 5.62l4.285 4.285.62-.619L3.928 5.31 7.904 1.334 7.285.715 3 5v.619z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M12.904 9.57L8.928 5.596l3.976-3.976-.619-.62L8 5.286v.619l4.285 4.285.62-.619zM3 5.62l4.285 4.285.62-.619L3.928 5.31 7.904 1.334 7.285.715 3 5v.619z"/></svg>"##
            }
        }
    }

    /// Stash icon
    pub fn stash(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path d="M10 2v2H6V2h4zm1 0h2.5a.5.5 0 0 1 .5.5v11a.5.5 0 0 1-.5.5h-11a.5.5 0 0 1-.5-.5v-11a.5.5 0 0 1 .5-.5H5v2a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V2zm0 4H5v1h6V6zm0 2H5v1h6V8zm-6 2h6v1H5v-1z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path d="M10 2v2H6V2h4zm1 0h2.5a.5.5 0 0 1 .5.5v11a.5.5 0 0 1-.5.5h-11a.5.5 0 0 1-.5-.5v-11a.5.5 0 0 1 .5-.5H5v2a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V2zm0 4H5v1h6V6zm0 2H5v1h6V8zm-6 2h6v1H5v-1z"/></svg>"##
            }
        }
    }

    /// Diff icon
    pub fn diff(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M2 3.5l.5-.5h5l.5.5v9l-.5.5h-5l-.5-.5v-9zM3 12h4V6H3v6zm0-7h4V4H3v1zm6.5-2h5l.5.5v9l-.5.5h-5l-.5-.5v-9l.5-.5zm.5 9h4v-2h-4v2zm0-3h4V4h-4v5z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M2 3.5l.5-.5h5l.5.5v9l-.5.5h-5l-.5-.5v-9zM3 12h4V6H3v6zm0-7h4V4H3v1zm6.5-2h5l.5.5v9l-.5.5h-5l-.5-.5v-9l.5-.5zm.5 9h4v-2h-4v2zm0-3h4V4h-4v5z"/></svg>"##
            }
        }
    }

    /// Info icon
    pub fn info(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#cccccc"><path fill-rule="evenodd" clip-rule="evenodd" d="M8.5 1a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13zm0 12a5.5 5.5 0 1 0 0-11 5.5 5.5 0 0 0 0 11zM8 4a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 8 4zm0 8a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"/></svg>"##
            }
            Theme::Light => {
                r##"<svg width="16" height="16" viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg" fill="#424242"><path fill-rule="evenodd" clip-rule="evenodd" d="M8.5 1a6.5 6.5 0 1 1 0 13 6.5 6.5 0 0 1 0-13zm0 12a5.5 5.5 0 1 0 0-11 5.5 5.5 0 0 0 0 11zM8 4a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 8 4zm0 8a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"/></svg>"##
            }
        }
    }
}
