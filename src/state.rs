use serde::Serialize;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize)]
pub struct Target {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone)]
pub struct AppState {
    pub targets: Arc<Mutex<Vec<Target>>>,
    pub tx: broadcast::Sender<Vec<Target>>,
}
