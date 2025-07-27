//! Terminal emulator widget for Dioxus

use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use alacritty_terminal::{
    event::Event as TermEvent,
    sync::FairMutex,
    Term,
};

use super::{
    backend::{TerminalBackend, EventProxy},
    config::TerminalConfig,
    grid_renderer::GridRenderer,
    input::{keyboard_to_bytes, mouse_to_bytes, handle_paste},
    pty_manager::PtyManager,
};

/// Terminal emulator component
#[component]
pub fn TerminalEmulator(
    terminal_id: String,
    initial_directory: Option<String>,
) -> Element {
    // Terminal state
    let mut terminal_ref = use_signal(|| None::<Arc<FairMutex<Term<EventProxy>>>>);
    let mut backend_ref = use_signal(|| None::<Arc<Mutex<TerminalBackend>>>);
    let mut pty_manager = use_signal(|| Arc::new(Mutex::new(PtyManager::new(terminal_id.clone()))));
    let mut grid_html = use_signal(|| String::new());
    let mut terminal_size = use_signal(|| (80u16, 24u16));
    
    // Initialize terminal
    let terminal_id_for_init = terminal_id.clone();
    use_effect(move || {
        if terminal_ref.read().is_none() {
            let mut config = TerminalConfig::default();
            config.working_directory = initial_directory.clone();
            
            // Add Claude-specific environment variables
            config.env.push(("CLAUDE_TERMINAL".to_string(), "1".to_string()));
            
            match TerminalBackend::new(config) {
                Ok((backend, mut event_rx)) => {
                    let terminal = backend.terminal();
                    terminal_ref.set(Some(Arc::clone(&terminal)));
                    
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
                            
                            // Update grid display
                            let terminal = terminal_for_events.lock();
                            let html = GridRenderer::render_to_html(&*terminal);
                            drop(terminal);
                            grid_html.set(html);
                        }
                    });
                    
                    // Initial render
                    let terminal = terminal.lock();
                    let html = GridRenderer::render_to_html(&*terminal);
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
                    let alt_screen = terminal.mode().contains(alacritty_terminal::term::TermMode::ALT_SCREEN);
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
            
            // Render terminal grid
            div {
                dangerous_inner_html: "{grid_html}"
            }
        }
    }
}