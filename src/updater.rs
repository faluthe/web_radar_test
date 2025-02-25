use log::info;
use std::time::Duration;
use tokio::time;

use crate::state::AppState;
use rand::Rng;

/// Spawns a background task that periodically updates the targets
pub fn spawn_updater(app_state: AppState) {
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(2));
        loop {
            ticker.tick().await;
            let mut rng = rand::rng();

            // Lock and update targets
            let mut targets = app_state.targets.lock().unwrap();
            for target in targets.iter_mut() {
                target.x = rng.random_range(-2000..2001);
                target.y = rng.random_range(-2000..2001);
            }

            // Broadcast the updated targets to all active WebSocket clients
            if let Err(e) = app_state.tx.send(targets.clone()) {
                // If no receivers, send() fails, but we can just log or ignore
                info!("No active subscribers for broadcast: {:?}", e);
            }

            info!("Updated targets: {:?}", *targets);
        }
    });
}
