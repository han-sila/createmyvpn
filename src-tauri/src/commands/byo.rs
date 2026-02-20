use tauri::{AppHandle, Emitter};

use crate::commands::timer;
use crate::error::AppError;
use crate::persistence::store;
use crate::ssh;
use crate::state::{DeploymentState, DeploymentStatus, ProgressEvent};
use crate::wireguard::{client_config, keys, server_config};

fn emit_progress(app: &AppHandle, step: u32, total: u32, message: &str, status: &str) {
    let event = ProgressEvent {
        step,
        total_steps: total,
        message: message.to_string(),
        status: status.to_string(),
    };
    let _ = app.emit("deploy-progress", event);
}

/// Deploy WireGuard on any Ubuntu 22.04/24.04 server via SSH.
/// Reuses the same SSH configure module as the AWS deploy path.
#[tauri::command]
pub async fn deploy_byo_vps(
    app: AppHandle,
    server_ip: String,
    ssh_private_key: String,
    ssh_user: String,
    ssh_port: u16,
    auto_destroy_hours: Option<u32>,
) -> Result<DeploymentState, AppError> {
    let total_steps = 4u32;
    tracing::info!("=== Starting BYO VPS deployment to {} ===", server_ip);

    let mut state = DeploymentState {
        status: DeploymentStatus::Deploying,
        deployment_mode: Some("byo".to_string()),
        elastic_ip: Some(server_ip.clone()),
        ssh_private_key: Some(ssh_private_key.clone()),
        ssh_user: Some(ssh_user.clone()),
        ..Default::default()
    };
    store::save_state(&state)?;

    let settings = store::load_settings()?;

    // Step 1: Generate WireGuard keys
    emit_progress(&app, 1, total_steps, "Generating WireGuard keys...", "running");
    tracing::info!("[BYO 1/{}] Generating WireGuard key pairs", total_steps);

    let server_keys = keys::generate_keypair();
    let client_keys = keys::generate_keypair();

    let wg_server_conf = server_config::render_server_config(
        &server_keys.private_key,
        &client_keys.public_key,
        settings.wireguard_port,
    );
    let client_conf = client_config::render_client_config(
        &client_keys.private_key,
        &server_keys.public_key,
        &server_ip,
        settings.wireguard_port,
    );

    // Step 2: SSH connect
    emit_progress(&app, 2, total_steps, "Connecting via SSH...", "running");
    tracing::info!(
        "[BYO 2/{}] Connecting to {}:{} as {}",
        total_steps,
        server_ip,
        ssh_port,
        ssh_user
    );
    let ssh_session =
        ssh::client::SshSession::connect(&server_ip, ssh_port, &ssh_user, &ssh_private_key, 60)
            .await?;
    tracing::info!("[BYO 2/{}] SSH connected", total_steps);

    // Step 3: Install WireGuard
    emit_progress(
        &app,
        3,
        total_steps,
        "Installing WireGuard (this may take a minute)...",
        "running",
    );
    tracing::info!("[BYO 3/{}] Configuring WireGuard via SSH", total_steps);
    ssh::configure::configure_wireguard(&ssh_session, &wg_server_conf, &server_keys.public_key)
        .await?;
    tracing::info!("[BYO 3/{}] WireGuard configured on server", total_steps);

    // Step 4: Save state and client config
    emit_progress(&app, 4, total_steps, "Saving client configuration...", "running");
    tracing::info!("[BYO 4/{}] Saving state and client config", total_steps);

    state.server_public_key = Some(server_keys.public_key);
    state.client_private_key = Some(client_keys.private_key);
    state.client_public_key = Some(client_keys.public_key);
    state.client_config = Some(client_conf.clone());
    state.status = DeploymentStatus::Deployed;
    state.deployed_at = Some(chrono::Utc::now());

    if let Some(hours) = auto_destroy_hours {
        let destroy_at = chrono::Utc::now() + chrono::Duration::hours(hours as i64);
        state.auto_destroy_at = Some(destroy_at);
        tracing::info!("[BYO] Auto-destroy scheduled for {}", destroy_at);
    }

    store::save_state(&state)?;
    store::save_client_config(&client_conf)?;

    if let Some(at) = state.auto_destroy_at {
        timer::spawn_auto_destroy_timer(app.clone(), at);
    }

    emit_progress(&app, total_steps, total_steps, "Your server is ready!", "done");
    tracing::info!("=== BYO VPS deployment complete! Server: {} ===", server_ip);

    Ok(state)
}
