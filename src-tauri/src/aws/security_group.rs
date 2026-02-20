use aws_sdk_ec2::types::{IpPermission, IpRange};
use aws_sdk_ec2::Client;

use crate::error::AppError;

/// Create security group with SSH (TCP 22) + WireGuard (UDP 51820) inbound rules.
pub async fn create_security_group(
    ec2: &Client,
    vpc_id: &str,
    wireguard_port: u16,
) -> Result<String, AppError> {
    let resp = ec2
        .create_security_group()
        .group_name(format!("createmyvpn-sg-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()))
        .description("CreateMyVpn VPN server - SSH + WireGuard")
        .vpc_id(vpc_id)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to create security group: {}", e)))?;

    let sg_id = resp
        .group_id()
        .ok_or_else(|| AppError::Aws("Security group created but no ID returned".into()))?
        .to_string();

    // SSH rule (TCP 22) - from anywhere for initial setup
    let ssh_rule = IpPermission::builder()
        .ip_protocol("tcp")
        .from_port(22)
        .to_port(22)
        .ip_ranges(IpRange::builder().cidr_ip("0.0.0.0/0").description("SSH access").build())
        .build();

    // WireGuard rule (UDP 51820)
    let wg_rule = IpPermission::builder()
        .ip_protocol("udp")
        .from_port(wireguard_port as i32)
        .to_port(wireguard_port as i32)
        .ip_ranges(IpRange::builder().cidr_ip("0.0.0.0/0").description("WireGuard VPN").build())
        .build();

    ec2.authorize_security_group_ingress()
        .group_id(&sg_id)
        .ip_permissions(ssh_rule)
        .ip_permissions(wg_rule)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to add ingress rules: {}", e)))?;

    // Tag it
    ec2.create_tags()
        .resources(&sg_id)
        .tags(
            aws_sdk_ec2::types::Tag::builder()
                .key("Name")
                .value("createmyvpn-sg")
                .build(),
        )
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to tag security group: {}", e)))?;

    tracing::info!("Created security group: {}", sg_id);
    Ok(sg_id)
}
