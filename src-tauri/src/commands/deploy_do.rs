use tauri::{AppHandle, Emitter};

use crate::commands::timer;
use crate::do_cloud::{client::DoClient, droplet, firewall, key as do_key};
use crate::error::AppError;
use crate::persistence::store;
use crate::ssh;
use crate::state::{DeploymentState, DeploymentStatus, ProgressEvent};
use crate::wireguard::{client_config, keys, server_config};

use rand::rngs::OsRng;
use ssh_key::{Algorithm, LineEnding, PrivateKey};

fn emit_progress(app: &AppHandle, step: u32, total: u32, message: &str, status: &str) {
    let event = ProgressEvent {
        step,
        total_steps: total,
        message: message.to_string(),
        status: status.to_string(),
    };
    let _ = app.emit("deploy-progress", event);
}

/// Deploy a WireGuard VPN on a DigitalOcean Droplet (7 steps).
#[tauri::command]
pub async fn deploy_do(
    app: AppHandle,
    region: String,
    size: String,
    auto_destroy_hours: Option<u32>,
) -> Result<DeploymentState, AppError> {
    let total_steps = 7u32;
    tracing::info!(
        "=== Starting DigitalOcean deployment to region: {} ===",
        region
    );

    let mut state = DeploymentState {
        status: DeploymentStatus::Deploying,
        deployment_mode: Some("do".to_string()),
        region: Some(region.clone()),
        ssh_user: Some("root".to_string()),
        ..Default::default()
    };
    store::save_state(&state)?;

    let creds = store::load_do_credentials()?
        .ok_or_else(|| AppError::Credential("No DigitalOcean credentials saved".into()))?;
    let settings = store::load_settings()?;

    // Step 1: Validate token / init client
    emit_progress(&app, 1, total_steps, "Connecting to DigitalOcean...", "running");
    tracing::info!("[DO 1/{}] Validating DigitalOcean API token", total_steps);
    DoClient::validate(&creds.api_token).await?;
    let client = DoClient::new(&creds.api_token);
    tracing::info!("[DO 1/{}] Token valid", total_steps);

    // Step 2: Generate SSH key pair + upload to DO
    emit_progress(&app, 2, total_steps, "Generating SSH keys...", "running");
    tracing::info!("[DO 2/{}] Generating Ed25519 SSH key pair", total_steps);
    let ssh_private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
        .map_err(|e| AppError::General(format!("SSH key generation failed: {}", e)))?;
    let ssh_private_pem = ssh_private_key
        .to_openssh(LineEnding::LF)
        .map_err(|e| AppError::General(format!("SSH key serialization failed: {}", e)))?;
    let ssh_public_openssh = ssh_private_key
        .public_key()
        .to_openssh()
        .map_err(|e| AppError::General(format!("SSH public key serialization failed: {}", e)))?;

    // Store private key (as string copy — we need it for SSH later and for state persistence)
    let private_pem_str: String = ssh_private_pem.to_string();
    state.ssh_private_key = Some(private_pem_str.clone());

    let key_id = do_key::upload_ssh_key(&client, "createmyvpn-key", &ssh_public_openssh).await?;
    tracing::info!(
        "[DO 2/{}] SSH key uploaded to DO, key_id={}",
        total_steps,
        key_id
    );
    state.do_ssh_key_id = Some(key_id);
    store::save_state(&state)?;

    // Step 3: Create Droplet
    emit_progress(&app, 3, total_steps, "Creating Droplet...", "running");
    tracing::info!(
        "[DO 3/{}] Creating Droplet (region={}, size={})",
        total_steps,
        region,
        size
    );
    let droplet_id =
        droplet::create_droplet(&client, "createmyvpn-server", &region, &size, key_id).await?;
    tracing::info!(
        "[DO 3/{}] Droplet created: droplet_id={}",
        total_steps,
        droplet_id
    );
    state.droplet_id = Some(droplet_id);
    store::save_state(&state)?;

    // Step 4: Create Firewall and attach it to the Droplet
    emit_progress(&app, 4, total_steps, "Creating firewall rules...", "running");
    tracing::info!(
        "[DO 4/{}] Creating firewall (WireGuard port: {})",
        total_steps,
        settings.wireguard_port
    );
    let firewall_id =
        firewall::create_firewall(&client, droplet_id, settings.wireguard_port).await?;
    tracing::info!(
        "[DO 4/{}] Firewall created: {}",
        total_steps,
        firewall_id
    );
    state.do_firewall_id = Some(firewall_id);
    store::save_state(&state)?;

    // Step 5: Wait for Droplet to become active + extract public IPv4
    emit_progress(&app, 5, total_steps, "Waiting for server to start...", "running");
    tracing::info!(
        "[DO 5/{}] Waiting for Droplet {} to become active...",
        total_steps,
        droplet_id
    );
    let server_ip = droplet::wait_for_active(&client, droplet_id).await?;
    tracing::info!(
        "[DO 5/{}] Droplet active, IP: {}",
        total_steps,
        server_ip
    );
    state.elastic_ip = Some(server_ip.clone());
    store::save_state(&state)?;

    // Step 6: Configure WireGuard via SSH
    emit_progress(
        &app,
        6,
        total_steps,
        "Configuring WireGuard (this may take a minute)...",
        "running",
    );
    tracing::info!("[DO 6/{}] Generating WireGuard key pairs", total_steps);
    let server_keys = keys::generate_keypair();
    let client_keys = keys::generate_keypair();

    let wg_server_conf = server_config::render_server_config(
        &server_keys.private_key,
        &client_keys.public_key,
        settings.wireguard_port,
    );

    // Give the droplet time for SSH to become reachable
    tracing::info!(
        "[DO 6/{}] Waiting 30s for SSH to become available on {}:22...",
        total_steps,
        server_ip
    );
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    tracing::info!(
        "[DO 6/{}] Connecting via SSH to {} as root (timeout=120s)",
        total_steps,
        server_ip
    );
    let ssh_session =
        ssh::client::SshSession::connect(&server_ip, 22, "root", &private_pem_str, 120).await?;
    tracing::info!(
        "[DO 6/{}] SSH connected, configuring WireGuard...",
        total_steps
    );

    ssh::configure::configure_wireguard(&ssh_session, &wg_server_conf, &server_keys.public_key)
        .await?;
    tracing::info!(
        "[DO 6/{}] WireGuard configured on server",
        total_steps
    );

    state.server_public_key = Some(server_keys.public_key.clone());
    state.client_private_key = Some(client_keys.private_key.clone());
    state.client_public_key = Some(client_keys.public_key.clone());

    // Step 7: Generate + save client config
    emit_progress(&app, 7, total_steps, "Generating client config...", "running");
    tracing::info!("[DO 7/{}] Rendering WireGuard client config", total_steps);
    let client_conf = client_config::render_client_config(
        &client_keys.private_key,
        &server_keys.public_key,
        &server_ip,
        settings.wireguard_port,
    );
    state.client_config = Some(client_conf.clone());
    store::save_client_config(&client_conf)?;

    state.status = DeploymentStatus::Deployed;
    state.deployed_at = Some(chrono::Utc::now());

    if let Some(hours) = auto_destroy_hours {
        let destroy_at = chrono::Utc::now() + chrono::Duration::hours(hours as i64);
        state.auto_destroy_at = Some(destroy_at);
        tracing::info!("[DO] Auto-destroy scheduled for {}", destroy_at);
    }

    store::save_state(&state)?;

    if let Some(at) = state.auto_destroy_at {
        timer::spawn_auto_destroy_timer(app.clone(), at);
    }

    tracing::info!(
        "=== DigitalOcean deployment complete! Server IP: {} ===",
        server_ip
    );
    emit_progress(
        &app,
        total_steps,
        total_steps,
        "VPN deployed successfully!",
        "done",
    );

    Ok(state)
}
