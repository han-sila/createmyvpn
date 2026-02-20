use chrono::{DateTime, Utc};
use tauri::AppHandle;

use crate::persistence::store;
use crate::state::DeploymentStatus;

/// Spawns a background task that automatically destroys the VPN deployment
/// when `at` is reached. If the app restarts before firing, `lib.rs` re-spawns
/// this timer from the persisted `auto_destroy_at` field in state.
pub fn spawn_auto_destroy_timer(app: AppHandle, at: DateTime<Utc>) {
    tokio::spawn(async move {
        let now = Utc::now();
        let delay = if at > now {
            (at - now).to_std().unwrap_or(std::time::Duration::ZERO)
        } else {
            std::time::Duration::ZERO
        };

        tracing::info!("Auto-destroy timer set: fires in {:?}", delay);
        tokio::time::sleep(delay).await;

        match store::load_state() {
            Ok(state) if state.status == DeploymentStatus::Deployed => {
                tracing::info!("Auto-destroy timer fired — destroying deployment...");
                if let Err(e) = crate::commands::destroy::destroy_vpn_internal(&app).await {
                    tracing::error!("Auto-destroy failed: {}", e);
                }
            }
            _ => {
                tracing::info!(
                    "Auto-destroy timer fired but no active deployment found, skipping"
                );
            }
        }
    });
}
