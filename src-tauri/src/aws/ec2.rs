use aws_sdk_ec2::Client;

use crate::error::AppError;

/// Generate an SSH key pair via EC2 API and return (key_pair_name, private_key_pem).
pub async fn create_key_pair(ec2: &Client) -> Result<(String, String), AppError> {
    let key_name = format!("createmyvpn-key-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

    let resp = ec2
        .create_key_pair()
        .key_name(&key_name)
        .key_type(aws_sdk_ec2::types::KeyType::Rsa)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to create key pair: {}", e)))?;

    let private_key = resp
        .key_material()
        .ok_or_else(|| AppError::Aws("Key pair created but no key material returned".into()))?
        .to_string();

    tracing::info!("Created key pair: {}", key_name);
    Ok((key_name, private_key))
}

/// Launch a t2.micro instance with user_data that enables IP forwarding.
pub async fn launch_instance(
    ec2: &Client,
    ami_id: &str,
    instance_type: &str,
    subnet_id: &str,
    security_group_id: &str,
    key_name: &str,
) -> Result<String, AppError> {
    let user_data = r#"#!/bin/bash
set -e
exec > /var/log/user-data.log 2>&1
echo "=== CreateMyVpn VPN Server Bootstrap ==="
apt-get update -y
echo 'net.ipv4.ip_forward=1' > /etc/sysctl.d/99-vpn.conf
echo 'net.ipv6.conf.all.disable_ipv6=1' >> /etc/sysctl.d/99-vpn.conf
sysctl -p /etc/sysctl.d/99-vpn.conf
echo "=== IP Forwarding Enabled ==="
touch /tmp/user-data-complete
echo "=== Bootstrap Complete ==="
"#;

    let user_data_b64 = base64_encode(user_data);

    let resp = ec2
        .run_instances()
        .image_id(ami_id)
        .instance_type(aws_sdk_ec2::types::InstanceType::from(instance_type))
        .min_count(1)
        .max_count(1)
        .subnet_id(subnet_id)
        .security_group_ids(security_group_id)
        .key_name(key_name)
        .user_data(&user_data_b64)
        .block_device_mappings(
            aws_sdk_ec2::types::BlockDeviceMapping::builder()
                .device_name("/dev/sda1")
                .ebs(
                    aws_sdk_ec2::types::EbsBlockDevice::builder()
                        .volume_type(aws_sdk_ec2::types::VolumeType::Gp3)
                        .volume_size(20)
                        .delete_on_termination(true)
                        .encrypted(true)
                        .build(),
                )
                .build(),
        )
        .tag_specifications(
            aws_sdk_ec2::types::TagSpecification::builder()
                .resource_type(aws_sdk_ec2::types::ResourceType::Instance)
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("Name")
                        .value("createmyvpn-vpn-server")
                        .build(),
                )
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("ManagedBy")
                        .value("createmyvpn")
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to launch instance: {}", e)))?;

    let instance_id = resp
        .instances()
        .first()
        .and_then(|i| i.instance_id())
        .ok_or_else(|| AppError::Aws("Instance launched but no ID returned".into()))?
        .to_string();

    tracing::info!("Launched instance: {}", instance_id);
    Ok(instance_id)
}

/// Wait for instance to reach "running" state.
pub async fn wait_for_instance_running(ec2: &Client, instance_id: &str) -> Result<(), AppError> {
    tracing::info!("Waiting for instance {} to be running...", instance_id);

    for attempt in 0..60 {
        let resp = ec2
            .describe_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(|e| AppError::Aws(format!("Failed to describe instance: {}", e)))?;

        if let Some(reservation) = resp.reservations().first() {
            if let Some(instance) = reservation.instances().first() {
                if let Some(state) = instance.state() {
                    let state_name = state.name().map(|n| n.as_str()).unwrap_or("unknown");
                    if state_name == "running" {
                        tracing::info!("Instance {} is running (attempt {})", instance_id, attempt);
                        return Ok(());
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    Err(AppError::Aws(format!(
        "Instance {} did not reach running state within timeout",
        instance_id
    )))
}

/// Allocate an Elastic IP and associate it with the instance.
pub async fn allocate_and_associate_eip(
    ec2: &Client,
    instance_id: &str,
) -> Result<(String, String, String), AppError> {
    // Allocate EIP
    let alloc_resp = ec2
        .allocate_address()
        .domain(aws_sdk_ec2::types::DomainType::Vpc)
        .tag_specifications(
            aws_sdk_ec2::types::TagSpecification::builder()
                .resource_type(aws_sdk_ec2::types::ResourceType::ElasticIp)
                .tags(
                    aws_sdk_ec2::types::Tag::builder()
                        .key("Name")
                        .value("createmyvpn-eip")
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to allocate EIP: {}", e)))?;

    let allocation_id = alloc_resp
        .allocation_id()
        .ok_or_else(|| AppError::Aws("EIP allocated but no allocation ID returned".into()))?
        .to_string();

    let elastic_ip = alloc_resp
        .public_ip()
        .ok_or_else(|| AppError::Aws("EIP allocated but no public IP returned".into()))?
        .to_string();

    tracing::info!("Allocated EIP: {} ({})", elastic_ip, allocation_id);

    // Associate with instance
    let assoc_resp = ec2
        .associate_address()
        .allocation_id(&allocation_id)
        .instance_id(instance_id)
        .send()
        .await
        .map_err(|e| AppError::Aws(format!("Failed to associate EIP: {}", e)))?;

    let association_id = assoc_resp
        .association_id()
        .ok_or_else(|| AppError::Aws("EIP associated but no association ID returned".into()))?
        .to_string();

    tracing::info!("Associated EIP with instance: {}", association_id);
    Ok((allocation_id, association_id, elastic_ip))
}

fn base64_encode(input: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(input.as_bytes())
}
