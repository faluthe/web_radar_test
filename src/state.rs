use serde::Serialize;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize)]
pub struct Target {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone)]
pub struct AppState {
    pub targets: Arc<Mutex<Vec<Target>>>,
    pub tx: broadcast::Sender<Vec<Target>>,
}
