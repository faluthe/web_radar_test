use crate::{
    state::{AppState, Target},
    updater::spawn_updater,
    ws_handler::websocket_handler,
};
use axum::{
    routing::{get, get_service},
    Router,
};
use log::info;
use std::{
    env,
    sync::{Arc, Mutex},
};
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::services::ServeDir;

mod state;
mod updater;
mod ws_handler;

const ADDRESS: &str = "0.0.0.0";
const PORT: u16 = 3000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    // Create a broadcast channel and shared state
    let (tx, _rx) = broadcast::channel(16);
    let app_state = AppState {
        targets: Arc::new(Mutex::new(vec![
            Target { x: 0, y: 0 },
            Target { x: 1, y: 1 },
        ])),
        tx,
    };

    // Spawn the background updater
    info!("Starting update loop");
    spawn_updater(app_state.clone());

    // Build router
    let router = Router::new()
        .route("/ws", get(websocket_handler))
        .fallback_service(get_service(ServeDir::new("./static")))
        .with_state(app_state);

    // Start the server
    info!("Starting server on http://{}:{}", ADDRESS, PORT);
    axum::serve(
        TcpListener::bind(format!("{}:{}", ADDRESS, PORT)).await?,
        router.into_make_service(),
    )
    .await?;

    Ok(())
}
