use tauri::{AppHandle, Emitter};

use crate::aws::{ami, client, ec2, security_group, vpc};
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

#[tauri::command]
pub async fn deploy_vpn(
    app: AppHandle,
    region: String,
    auto_destroy_hours: Option<u32>,
) -> Result<DeploymentState, AppError> {
    let total_steps = 10;
    tracing::info!("=== Starting VPN deployment to region: {} ===", region);

    let mut state = DeploymentState {
        status: DeploymentStatus::Deploying,
        region: Some(region.clone()),
        ..Default::default()
    };
    store::save_state(&state)?;

    let creds = store::load_credentials()?
        .ok_or_else(|| AppError::Credential("No credentials saved".into()))?;
    let settings = store::load_settings()?;
    tracing::info!(
        "Loaded settings: instance_type={}, wireguard_port={}",
        settings.instance_type,
        settings.wireguard_port
    );

    // Step 1: Build AWS config
    emit_progress(&app, 1, total_steps, "Connecting to AWS...", "running");
    tracing::info!("[Step 1/{}] Building AWS config for region {}", total_steps, region);
    let config = client::build_config(&creds, &region).await?;
    let ec2_client = aws_sdk_ec2::Client::new(&config);
    tracing::info!("[Step 1/{}] AWS config ready", total_steps);

    // Step 2: Lookup AMI
    emit_progress(&app, 2, total_steps, "Finding Ubuntu AMI...", "running");
    tracing::info!("[Step 2/{}] Looking up latest Ubuntu 22.04 AMI in {}", total_steps, region);
    let ami_id = ami::lookup_ubuntu_ami(&config).await?;
    tracing::info!("[Step 2/{}] Using AMI: {}", total_steps, ami_id);

    // Step 3: Create VPC
    emit_progress(&app, 3, total_steps, "Creating VPC...", "running");
    tracing::info!("[Step 3/{}] Creating VPC", total_steps);
    let vpc_id = vpc::create_vpc(&ec2_client).await?;
    tracing::info!("[Step 3/{}] VPC created: {}", total_steps, vpc_id);
    state.vpc_id = Some(vpc_id.clone());
    store::save_state(&state)?;

    // Step 4: Create IGW + Subnet + Route Table
    emit_progress(&app, 4, total_steps, "Setting up networking...", "running");
    tracing::info!("[Step 4/{}] Creating Internet Gateway", total_steps);
    let igw_id = vpc::create_internet_gateway(&ec2_client, &vpc_id).await?;
    tracing::info!("[Step 4/{}] IGW created: {}", total_steps, igw_id);
    state.igw_id = Some(igw_id.clone());
    store::save_state(&state)?;

    tracing::info!("[Step 4/{}] Creating subnet in {}a", total_steps, region);
    let subnet_id = vpc::create_subnet(&ec2_client, &vpc_id, &region).await?;
    tracing::info!("[Step 4/{}] Subnet created: {}", total_steps, subnet_id);
    state.subnet_id = Some(subnet_id.clone());
    store::save_state(&state)?;

    tracing::info!("[Step 4/{}] Creating route table", total_steps);
    let rt_id = vpc::create_route_table(&ec2_client, &vpc_id, &igw_id, &subnet_id).await?;
    tracing::info!("[Step 4/{}] Route table created: {}", total_steps, rt_id);
    state.route_table_id = Some(rt_id);
    store::save_state(&state)?;

    // Step 5: Create Security Group
    emit_progress(&app, 5, total_steps, "Creating firewall rules...", "running");
    tracing::info!(
        "[Step 5/{}] Creating security group (WireGuard port: {})",
        total_steps,
        settings.wireguard_port
    );
    let sg_id =
        security_group::create_security_group(&ec2_client, &vpc_id, settings.wireguard_port)
            .await?;
    tracing::info!("[Step 5/{}] Security group created: {}", total_steps, sg_id);
    state.security_group_id = Some(sg_id.clone());
    store::save_state(&state)?;

    // Step 6: Create Key Pair
    emit_progress(&app, 6, total_steps, "Generating SSH keys...", "running");
    tracing::info!("[Step 6/{}] Creating EC2 key pair", total_steps);
    let (key_name, private_key) = ec2::create_key_pair(&ec2_client).await?;
    tracing::info!("[Step 6/{}] Key pair created: {}", total_steps, key_name);
    state.key_pair_name = Some(key_name.clone());
    state.ssh_private_key = Some(private_key.clone());
    store::save_state(&state)?;

    // Step 7: Launch Instance
    emit_progress(&app, 7, total_steps, "Launching server...", "running");
    tracing::info!(
        "[Step 7/{}] Launching EC2 instance (ami={}, type={}, subnet={}, sg={})",
        total_steps,
        ami_id,
        settings.instance_type,
        subnet_id,
        sg_id
    );
    let instance_id = ec2::launch_instance(
        &ec2_client,
        &ami_id,
        &settings.instance_type,
        &subnet_id,
        &sg_id,
        &key_name,
    )
    .await?;
    tracing::info!("[Step 7/{}] Instance launched: {}", total_steps, instance_id);
    state.instance_id = Some(instance_id.clone());
    store::save_state(&state)?;

    tracing::info!("[Step 7/{}] Waiting for instance {} to reach running state...", total_steps, instance_id);
    ec2::wait_for_instance_running(&ec2_client, &instance_id).await?;
    tracing::info!("[Step 7/{}] Instance {} is running", total_steps, instance_id);

    // Step 8: Allocate EIP
    emit_progress(&app, 8, total_steps, "Allocating static IP...", "running");
    tracing::info!("[Step 8/{}] Allocating Elastic IP", total_steps);
    let (alloc_id, assoc_id, elastic_ip) =
        ec2::allocate_and_associate_eip(&ec2_client, &instance_id).await?;
    tracing::info!(
        "[Step 8/{}] EIP allocated: {} (alloc={}, assoc={})",
        total_steps,
        elastic_ip,
        alloc_id,
        assoc_id
    );
    state.allocation_id = Some(alloc_id);
    state.association_id = Some(assoc_id);
    state.elastic_ip = Some(elastic_ip.clone());
    store::save_state(&state)?;

    // Step 9: Generate WireGuard keys and configure via SSH
    emit_progress(
        &app,
        9,
        total_steps,
        "Configuring WireGuard (this may take a minute)...",
        "running",
    );
    tracing::info!("[Step 9/{}] Generating WireGuard key pairs", total_steps);

    let server_keys = keys::generate_keypair();
    let client_keys = keys::generate_keypair();

    let wg_server_conf = server_config::render_server_config(
        &server_keys.private_key,
        &client_keys.public_key,
        settings.wireguard_port,
    );

    // Wait a bit for SSH to become available after instance starts
    tracing::info!(
        "[Step 9/{}] Waiting 30s for SSH to become available on {}:22...",
        total_steps,
        elastic_ip
    );
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    tracing::info!(
        "[Step 9/{}] Connecting via SSH to {} as ubuntu (timeout=120s)",
        total_steps,
        elastic_ip
    );
    let ssh_session =
        ssh::client::SshSession::connect(&elastic_ip, 22, "ubuntu", &private_key, 120).await?;
    tracing::info!("[Step 9/{}] SSH connected, configuring WireGuard...", total_steps);

    ssh::configure::configure_wireguard(&ssh_session, &wg_server_conf, &server_keys.public_key)
        .await?;
    tracing::info!("[Step 9/{}] WireGuard configured on server", total_steps);

    state.server_public_key = Some(server_keys.public_key.clone());
    state.client_private_key = Some(client_keys.private_key.clone());
    state.client_public_key = Some(client_keys.public_key.clone());

    // Step 10: Generate client config
    emit_progress(&app, 10, total_steps, "Generating client config...", "running");
    tracing::info!("[Step 10/{}] Rendering WireGuard client config", total_steps);
    let client_conf = client_config::render_client_config(
        &client_keys.private_key,
        &server_keys.public_key,
        &elastic_ip,
        settings.wireguard_port,
    );
    state.client_config = Some(client_conf.clone());
    store::save_client_config(&client_conf)?;

    // Done!
    state.status = DeploymentStatus::Deployed;
    state.deployed_at = Some(chrono::Utc::now());

    if let Some(hours) = auto_destroy_hours {
        let destroy_at = chrono::Utc::now() + chrono::Duration::hours(hours as i64);
        state.auto_destroy_at = Some(destroy_at);
        tracing::info!("Auto-destroy scheduled for {} (in {}h)", destroy_at, hours);
    }

    store::save_state(&state)?;

    if let Some(at) = state.auto_destroy_at {
        timer::spawn_auto_destroy_timer(app.clone(), at);
    }

    tracing::info!("=== VPN deployment complete! Server IP: {} ===", elastic_ip);
    emit_progress(&app, total_steps, total_steps, "VPN deployed successfully!", "done");

    Ok(state)
}

#[tauri::command]
pub async fn get_deployment_state() -> Result<DeploymentState, AppError> {
    store::load_state()
}

/// Clears any stuck/failed deployment state so the user can start fresh.
#[tauri::command]
pub async fn reset_deployment_state() -> Result<(), AppError> {
    store::clear_state()
}
