use aws_sdk_ec2::Client;

use crate::error::AppError;
use crate::state::DeploymentState;

/// Ordered teardown of all AWS resources. Reads state to know what exists.
/// Order: EIP association → EIP → Instance → Key pair → SG → Subnet → RT → IGW → VPC
pub async fn teardown_all(ec2: &Client, state: &DeploymentState) -> Result<(), AppError> {
    // 1. Disassociate EIP
    if let Some(ref assoc_id) = state.association_id {
        tracing::info!("Disassociating EIP: {}", assoc_id);
        let _ = ec2
            .disassociate_address()
            .association_id(assoc_id)
            .send()
            .await;
    }

    // 2. Release EIP
    if let Some(ref alloc_id) = state.allocation_id {
        tracing::info!("Releasing EIP: {}", alloc_id);
        let _ = ec2
            .release_address()
            .allocation_id(alloc_id)
            .send()
            .await;
    }

    // 3. Terminate instance and wait
    if let Some(ref instance_id) = state.instance_id {
        tracing::info!("Terminating instance: {}", instance_id);
        let _ = ec2
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await;

        // Wait for termination
        for _ in 0..60 {
            let resp = ec2
                .describe_instances()
                .instance_ids(instance_id)
                .send()
                .await;

            if let Ok(resp) = resp {
                if let Some(reservation) = resp.reservations().first() {
                    if let Some(instance) = reservation.instances().first() {
                        if let Some(s) = instance.state() {
                            let name = s.name().map(|n| n.as_str()).unwrap_or("");
                            if name == "terminated" {
                                break;
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    // 4. Delete key pair
    if let Some(ref key_name) = state.key_pair_name {
        tracing::info!("Deleting key pair: {}", key_name);
        let _ = ec2.delete_key_pair().key_name(key_name).send().await;
    }

    // 5. Delete security group (retry with backoff - may need instance to fully terminate)
    if let Some(ref sg_id) = state.security_group_id {
        tracing::info!("Deleting security group: {}", sg_id);
        for attempt in 0..10 {
            match ec2.delete_security_group().group_id(sg_id).send().await {
                Ok(_) => break,
                Err(e) => {
                    if attempt == 9 {
                        tracing::warn!("Failed to delete security group after retries: {}", e);
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }
    }

    // 6. Delete subnet
    if let Some(ref subnet_id) = state.subnet_id {
        tracing::info!("Deleting subnet: {}", subnet_id);
        let _ = ec2.delete_subnet().subnet_id(subnet_id).send().await;
    }

    // 7. Delete route table
    if let Some(ref rt_id) = state.route_table_id {
        tracing::info!("Deleting route table: {}", rt_id);
        let _ = ec2.delete_route_table().route_table_id(rt_id).send().await;
    }

    // 8. Detach and delete IGW
    if let Some(ref igw_id) = state.igw_id {
        if let Some(ref vpc_id) = state.vpc_id {
            tracing::info!("Detaching IGW: {} from VPC: {}", igw_id, vpc_id);
            let _ = ec2
                .detach_internet_gateway()
                .internet_gateway_id(igw_id)
                .vpc_id(vpc_id)
                .send()
                .await;
        }
        tracing::info!("Deleting IGW: {}", igw_id);
        let _ = ec2
            .delete_internet_gateway()
            .internet_gateway_id(igw_id)
            .send()
            .await;
    }

    // 9. Delete VPC
    if let Some(ref vpc_id) = state.vpc_id {
        tracing::info!("Deleting VPC: {}", vpc_id);
        let _ = ec2.delete_vpc().vpc_id(vpc_id).send().await;
    }

    tracing::info!("Teardown complete");
    Ok(())
}
