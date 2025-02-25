use log::{error, info};
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
            let mut targets = match app_state.targets.lock(){
                Ok(gaurd) => gaurd,
                Err(e) => {
                    error!("Failed to lock targets: {:?}", e);
                    continue;
                }
            };
            for target in targets.iter_mut() {
                target.x = rng.random_range(-2000..2001);
                target.y = rng.random_range(-2000..2001);
            }

            // Broadcast the updated targets to all active WebSocket clients
            if app_state.tx.send(targets.clone()).is_ok() {
                info!("Updated targets: {:?}", *targets);
            } else {
                error!("Failed to broadcast updated targets");
            }
        }
    });
}
