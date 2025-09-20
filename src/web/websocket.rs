use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tracing::warn;

/// Minimal placeholder websocket handler. The modern Electron stack handles
/// realtime streaming, so this endpoint simply accepts the socket and echoes
/// nothing back.
pub async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Close(_)) => break,
            Ok(_) => {
                // Ignore all incoming messages; the legacy backend is retired.
            }
            Err(err) => {
                warn!("WebSocket error: {}", err);
                break;
            }
        }
    }

    // Best-effort close when the loop exits.
    let _ = socket.send(Message::Close(None)).await;
}
