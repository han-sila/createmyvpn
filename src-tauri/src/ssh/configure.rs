use crate::error::AppError;
use crate::ssh::client::SshSession;

/// Full WireGuard server configuration sequence (replaces Ansible playbook).
pub async fn configure_wireguard(
    ssh: &SshSession,
    server_config: &str,
    server_public_key: &str,
) -> Result<(), AppError> {
    tracing::info!("Starting WireGuard configuration...");

    // 0. Wait for cloud-init to finish so it releases the apt lock.
    //    Ubuntu instances run cloud-init on first boot which holds apt for
    //    several minutes. We must wait before touching apt at all.
    tracing::info!("Waiting for cloud-init to complete (this can take 1-2 min)...");
    ssh.execute("sudo cloud-init status --wait").await?;
    tracing::info!("cloud-init complete, proceeding with package installation");

    // 1. Install WireGuard
    tracing::info!("Installing WireGuard packages...");
    ssh.execute("sudo DEBIAN_FRONTEND=noninteractive apt-get update -y")
        .await?;
    ssh.execute("sudo DEBIAN_FRONTEND=noninteractive apt-get install -y wireguard wireguard-tools")
        .await?;

    // 2. Ensure IP forwarding is enabled (backup in case user_data didn't run)
    tracing::info!("Enabling IP forwarding...");
    ssh.execute("echo 'net.ipv4.ip_forward=1' | sudo tee /etc/sysctl.d/99-vpn.conf")
        .await?;
    ssh.execute("sudo sysctl -p /etc/sysctl.d/99-vpn.conf")
        .await?;

    // 3. Deploy WireGuard server config
    tracing::info!("Deploying wg0.conf...");
    ssh.upload_file("/etc/wireguard/wg0.conf", server_config)
        .await?;
    ssh.execute("sudo chmod 600 /etc/wireguard/wg0.conf").await?;

    // 4. Save server public key for reference
    ssh.upload_file("/etc/wireguard/server_public.key", server_public_key)
        .await?;

    // 5. Enable and start WireGuard
    tracing::info!("Starting WireGuard service...");
    ssh.execute("sudo systemctl enable wg-quick@wg0").await?;
    ssh.execute("sudo systemctl start wg-quick@wg0").await?;

    // 6. Verify WireGuard is running
    tracing::info!("Verifying WireGuard...");
    let output = ssh.execute("sudo wg show wg0").await?;
    if output.contains("interface: wg0") {
        tracing::info!("WireGuard is running successfully");
    } else {
        return Err(AppError::Ssh(format!(
            "WireGuard verification failed. wg show output: {}",
            output
        )));
    }

    Ok(())
}
