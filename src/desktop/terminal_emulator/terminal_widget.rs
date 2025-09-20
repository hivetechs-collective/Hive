//! Terminal emulator widget for Dioxus

use alacritty_terminal::{event::Event as TermEvent, sync::FairMutex, Term};
use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{
    backend::{EventProxy, TerminalBackend},
    config::TerminalConfig,
    grid_renderer::GridRenderer,
    input::{handle_paste, keyboard_to_bytes, mouse_to_bytes},
    pty_manager::PtyManager,
};

/// Terminal emulator component
#[component]
pub fn TerminalEmulator(terminal_id: String, initial_directory: Option<String>) -> Element {
    // Terminal state
    let mut terminal_ref = use_signal(|| None::<Arc<FairMutex<Term<EventProxy>>>>);
    let mut backend_ref = use_signal(|| None::<Arc<Mutex<TerminalBackend>>>);
    let mut pty_manager = use_signal(|| Arc::new(Mutex::new(PtyManager::new(terminal_id.clone()))));
    let mut grid_html = use_signal(|| String::new());
    let mut terminal_size = use_signal(|| (80u16, 24u16));
    let mut scroll_offset = use_signal(|| 0i32);

    // Initialize terminal
    let terminal_id_for_init = terminal_id.clone();
    use_effect(move || {
        if terminal_ref.read().is_none() {
            let mut config = TerminalConfig::default();
            config.working_directory = initial_directory.clone();

            // Add Claude-specific environment variables
            config
                .env
                .push(("CLAUDE_TERMINAL".to_string(), "1".to_string()));

            match TerminalBackend::new(config) {
                Ok((backend, mut event_rx)) => {
                    let terminal = backend.terminal();
                    terminal_ref.set(Some(Arc::clone(&terminal)));

                    // Register with alacritty registry for Send to Consensus
                    use super::alacritty_registry::register_alacritty_terminal;
                    register_alacritty_terminal(
                        terminal_id_for_init.clone(),
                        Arc::clone(&terminal),
                    );

                    let backend = Arc::new(Mutex::new(backend));
                    backend_ref.set(Some(Arc::clone(&backend)));

                    // Update grid on terminal events
                    let terminal_for_events = Arc::clone(&terminal);
                    let mut grid_html = grid_html.clone();

                    spawn(async move {
                        while let Some(event) = event_rx.recv().await {
                            match event {
                                TermEvent::PtyWrite(data) => {
                                    // Terminal wants to write to PTY (shouldn't happen)
                                }
                                TermEvent::Title(title) => {
                                    tracing::info!("Terminal title: {}", title);
                                }
                                TermEvent::ResetTitle => {
                                    tracing::info!("Terminal title reset");
                                }
                                TermEvent::ClipboardStore(_, _) => {
                                    // Handle clipboard
                                }
                                TermEvent::ClipboardLoad(_, _) => {
                                    // Handle clipboard
                                }
                                TermEvent::ColorRequest(_, _) => {
                                    // Handle color request
                                }
                                TermEvent::MouseCursorDirty => {
                                    // Update mouse cursor
                                }
                                TermEvent::Exit => {
                                    tracing::info!("Terminal exited");
                                    break;
                                }
                                TermEvent::CursorBlinkingChange => {
                                    // Handle cursor blinking change
                                }
                                _ => {}
                            }

                            // Update grid display with scroll offset
                            let terminal = terminal_for_events.lock();
                            let html = GridRenderer::render_to_html_with_scroll(
                                &*terminal,
                                *scroll_offset.read(),
                            );
                            drop(terminal);
                            grid_html.set(html);
                        }
                    });

                    // Initial render
                    let terminal = terminal.lock();
                    let html = GridRenderer::render_to_html_with_scroll(&*terminal, 0);
                    drop(terminal);

                    grid_html.set(html);

                    tracing::info!("Terminal emulator initialized for {}", terminal_id_for_init);
                }
                Err(e) => {
                    tracing::error!("Failed to create terminal backend: {}", e);
                }
            }
        }
    });

    // Handle keyboard input
    let handle_keydown = {
        let backend_ref = backend_ref.clone();
        let terminal_ref = terminal_ref.clone();
        move |evt: Event<KeyboardData>| {
            evt.prevent_default();
            evt.stop_propagation();

            if let Some(backend) = backend_ref.read().as_ref() {
                if let Some(terminal) = terminal_ref.read().as_ref() {
                    let terminal = terminal.lock();
                    let alt_screen = terminal
                        .mode()
                        .contains(alacritty_terminal::term::TermMode::ALT_SCREEN);
                    drop(terminal);

                    if let Some(bytes) = keyboard_to_bytes(&evt, alt_screen) {
                        spawn(async move {
                            if let Ok(mut backend) = backend.lock().await {
                                if let Err(e) = backend.write(&bytes) {
                                    tracing::error!("Failed to write to terminal: {}", e);
                                }
                            }
                        });
                    }
                }
            }
        }
    };

    // Handle paste
    let handle_paste = {
        let backend_ref = backend_ref.clone();
        move |evt: Event<ClipboardData>| {
            evt.prevent_default();

            if let Some(backend) = backend_ref.read().as_ref() {
                spawn(async move {
                    if let Ok(text) = evt.files() {
                        let bytes = handle_paste(&text);
                        if let Ok(mut backend) = backend.lock().await {
                            if let Err(e) = backend.write(&bytes) {
                                tracing::error!("Failed to paste: {}", e);
                            }
                        }
                    }
                });
            }
        }
    };

    // Terminal container style
    let container_style = "
        width: 100%;
        height: 100%;
        background: #000000;
        color: #cccccc;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: 13px;
        line-height: 16px;
        overflow: auto;
        cursor: text;
        padding: 4px;
        box-sizing: border-box;
    ";

    rsx! {
        div {
            class: "terminal-emulator",
            style: "{container_style}",
            tabindex: "0",
            onkeydown: handle_keydown,
            onpaste: handle_paste,
            onwheel: move |evt| {
                // Handle mouse wheel for scrolling
                if let Some(terminal) = terminal_ref.read().as_ref() {
                    use alacritty_terminal::grid::Scroll;

                    let delta = evt.delta().y;
                    let lines_to_scroll = (delta / 16.0).round() as i32; // 16px per line

                    // Scroll the terminal
                    let mut terminal = terminal.lock();
                    if lines_to_scroll > 0 {
                        // Scrolling down (towards newer content)
                        terminal.scroll_display(Scroll::Delta(-lines_to_scroll));
                    } else if lines_to_scroll < 0 {
                        // Scrolling up (towards older content)
                        terminal.scroll_display(Scroll::Delta(-lines_to_scroll));
                    }

                    // Re-render after scroll
                    let html = GridRenderer::render_to_html(&*terminal);
                    drop(terminal);
                    grid_html.set(html);
                }
            },

            // Render terminal grid
            div {
                dangerous_inner_html: "{grid_html}"
            }
        }
    }
}
