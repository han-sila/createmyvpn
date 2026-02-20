use tauri::{AppHandle, Emitter};

use crate::aws::{client, teardown};
use crate::do_cloud;
use crate::error::AppError;
use crate::persistence::store;
use crate::ssh;
use crate::state::{DeploymentStatus, ProgressEvent};

fn emit_progress(app: &AppHandle, step: u32, total: u32, message: &str, status: &str) {
    let event = ProgressEvent {
        step,
        total_steps: total,
        message: message.to_string(),
        status: status.to_string(),
    };
    let _ = app.emit("destroy-progress", event);
}

/// Internal destroy logic — called by both the Tauri command and the auto-destroy timer.
pub async fn destroy_vpn_internal(app: &AppHandle) -> Result<(), AppError> {
    let mut state = store::load_state()?;

    if state.status == DeploymentStatus::NotDeployed {
        return Err(AppError::State("No deployment to destroy".into()));
    }

    // ── DigitalOcean: API teardown + clear local state ────────────────────────
    if state.deployment_mode.as_deref() == Some("do") {
        tracing::info!("Destroying DigitalOcean deployment");
        state.status = DeploymentStatus::Destroying;
        store::save_state(&state)?;

        match store::load_do_credentials() {
            Ok(Some(creds)) => {
                let do_client = do_cloud::client::DoClient::new(&creds.api_token);

                emit_progress(app, 1, 3, "Deleting firewall...", "running");
                if let Some(ref firewall_id) = state.do_firewall_id {
                    let result = do_cloud::firewall::delete_firewall(&do_client, firewall_id).await;
                    if let Err(e) = result {
                        tracing::warn!("Failed to delete DO firewall: {}", e);
                    }
                }

                emit_progress(app, 2, 3, "Deleting server...", "running");
                if let Some(droplet_id) = state.droplet_id {
                    let result = do_cloud::droplet::delete_droplet(&do_client, droplet_id).await;
                    if let Err(e) = result {
                        tracing::warn!("Failed to delete DO droplet: {}", e);
                    }
                }

                emit_progress(app, 3, 3, "Cleaning up...", "running");
                if let Some(key_id) = state.do_ssh_key_id {
                    let result = do_cloud::key::delete_ssh_key(&do_client, key_id).await;
                    if let Err(e) = result {
                        tracing::warn!("Failed to delete DO SSH key: {}", e);
                    }
                }
            }
            _ => {
                tracing::warn!(
                    "No DO credentials found during destroy — clearing local state only"
                );
            }
        }

        store::clear_state()?;
        emit_progress(app, 3, 3, "All resources destroyed", "done");
        return Ok(());
    }

    // ── BYO VPS: SSH cleanup + clear local state (no AWS calls) ──────────────
    if state.deployment_mode.as_deref() == Some("byo") {
        tracing::info!("Destroying BYO VPS deployment");
        emit_progress(app, 1, 2, "Stopping WireGuard on server...", "running");

        if let (Some(ip), Some(key)) = (&state.elastic_ip, &state.ssh_private_key) {
            let ssh_user = state.ssh_user.as_deref().unwrap_or("ubuntu");
            match ssh::client::SshSession::connect(ip, 22, ssh_user, key, 15).await {
                Ok(ssh) => {
                    let _ = ssh.execute("sudo systemctl stop wg-quick@wg0").await;
                    let _ = ssh.execute("sudo systemctl disable wg-quick@wg0").await;
                    tracing::info!("WireGuard stopped on BYO server {}", ip);
                }
                Err(e) => {
                    tracing::warn!(
                        "Could not SSH to BYO server for cleanup: {} — clearing local state anyway",
                        e
                    );
                }
            }
        }

        emit_progress(app, 2, 2, "Cleaning up local config...", "running");
        store::clear_state()?;
        emit_progress(app, 2, 2, "Server disconnected", "done");
        return Ok(());
    }

    // ── AWS teardown ──────────────────────────────────────────────────────────
    let region = state
        .region
        .clone()
        .ok_or_else(|| AppError::State("No region in state".into()))?;
    tracing::info!("Destroying AWS deployment in region: {}", region);

    let creds = store::load_credentials()?
        .ok_or_else(|| AppError::Credential("No credentials saved".into()))?;

    state.status = DeploymentStatus::Destroying;
    store::save_state(&state)?;
    tracing::info!("State updated to Destroying");

    emit_progress(app, 1, 3, "Connecting to AWS...", "running");
    tracing::info!("[Destroy 1/3] Connecting to AWS...");
    let config = client::build_config(&creds, &region).await?;
    let ec2_client = aws_sdk_ec2::Client::new(&config);
    tracing::info!("[Destroy 1/3] AWS connection established");

    emit_progress(app, 2, 3, "Destroying infrastructure...", "running");
    tracing::info!("[Destroy 2/3] Tearing down all AWS resources...");
    teardown::teardown_all(&ec2_client, &state).await?;
    tracing::info!("[Destroy 2/3] All AWS resources destroyed");

    emit_progress(app, 3, 3, "Cleaning up...", "running");
    tracing::info!("[Destroy 3/3] Cleaning up local state...");
    store::clear_state()?;
    tracing::info!("[Destroy 3/3] Local state cleared");

    emit_progress(app, 3, 3, "All resources destroyed", "done");
    Ok(())
}

#[tauri::command]
pub async fn destroy_vpn(app: AppHandle) -> Result<(), AppError> {
    tracing::info!("=== Starting VPN server destruction ===");
    destroy_vpn_internal(&app).await?;
    tracing::info!("=== VPN server destruction complete ===");
    Ok(())
}
