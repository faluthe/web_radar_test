use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::StreamExt;
use log::{error, info, warn};

use crate::state::AppState;

/// Handles WebSocket connections and forwards state updates.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_upgrade(socket, state))
}

async fn websocket_upgrade(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    info!("Client connected!");

    // Send initial state
    let targets = state.targets.lock().unwrap().clone();
    if let Err(e) = socket
        .send(Message::Text(
            serde_json::to_string(&targets).unwrap().into(),
        ))
        .await
    {
        error!("Error sending initial targets to client: {}", e);
        return;
    }

    // Listen for and forward updates
    loop {
        tokio::select! {
            Ok(new_targets) = rx.recv() => {
                if socket.send(Message::Text(serde_json::to_string(&new_targets).unwrap().into())).await.is_ok() {
                    info!("Forwarded update to client");
                }
            },
            maybe_msg = socket.next() => {
                match maybe_msg {
                    Some(Ok(_)) => {
                        // We could handle incoming messages here if needed
                        // For now, if the client sends anything we assume its a close msg
                    }
                    Some(Err(e)) => {
                        warn!("Client disconnected (with error): {}", e);
                        break;
                    }
                    None => {
                        info!("Client disconnected");
                        break;
                    }
                }
            }
        }
    }
}
