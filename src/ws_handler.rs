use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::StreamExt;
use log::{error, info, warn};

use crate::state::{AppState, Target};

/// Handles WebSocket connections and forwards state updates.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_upgrade(socket, state))
}

async fn websocket_upgrade(mut socket: WebSocket, state: AppState) {
    info!("Client connected!");

    // Subscribe to the updater's broadcast channel
    let mut rx = state.tx.subscribe();

    // Acquire lock on shared state
    let targets = match state.targets.lock() {
        Ok(gaurd) => gaurd.clone(),
        Err(e) => {
            error!("Failed to lock targets: {:?}", e);
            return;
        }
    };

    // Send initial targets to client
    let initial_json = match serde_json::to_string(&targets) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize initial targets: {}", e);
            return;
        }
    };

    if let Err(e) = socket.send(Message::Text(initial_json.into())).await {
        error!("Error sending initial targets to client: {}", e);
        return;
    }

    // Listen for and forward updates
    loop {
        tokio::select! {
            Ok(new_targets) = rx.recv() => {
                if let Err(e) = send_serialized_targets(&mut socket, &new_targets).await {
                    warn!("Failed to send updated targets to client: {}", e);
                } else {
                    info!("Forwarded updated targets to client");
                }
            },
            // Stream returns None when the client disconnects
            None = socket.next() => { break; }
        }
    }

    info!("Client disconnected");
}

async fn send_serialized_targets(
    socket: &mut WebSocket,
    targets: &Vec<Target>,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(targets)?;
    socket.send(Message::Text(json.into())).await?;
    Ok(())
}
