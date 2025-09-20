// Temporarily re-export from crates.io Dioxus until we switch to local copy
pub use dioxus::prelude;
pub use dioxus::*;
pub use dioxus_desktop as desktop;
pub use dioxus_router as router;
pub use dioxus_signals as signals;

// These don't exist in 0.6.3 crates.io version, so we'll need to handle differently
// pub use dioxus_document as document;
// pub use dioxus_html as html;
// pub use dioxus_hooks as hooks;

// Re-export what we need for compatibility
pub mod events {
    pub use dioxus::events::*;
}

// For now, document functions come from dioxus::prelude
pub mod document {
    pub use dioxus::prelude::*;
}
