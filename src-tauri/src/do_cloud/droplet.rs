use crate::do_cloud::client::DoClient;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct CreateDropletRequest {
    name: String,
    region: String,
    size: String,
    image: String,
    ssh_keys: Vec<u64>,
}

#[derive(Deserialize)]
struct DropletResponse {
    droplet: DropletInfo,
}

#[derive(Deserialize)]
struct DropletInfo {
    id: u64,
    status: Option<String>,
    networks: Option<Networks>,
}

#[derive(Deserialize)]
struct Networks {
    v4: Vec<NetworkV4>,
}

#[derive(Deserialize)]
struct NetworkV4 {
    ip_address: String,
    #[serde(rename = "type")]
    network_type: String,
}

/// Create a DigitalOcean Droplet and return its ID.
/// POST /v2/droplets
pub async fn create_droplet(
    client: &DoClient,
    name: &str,
    region: &str,
    size: &str,
    ssh_key_id: u64,
) -> Result<u64, AppError> {
    let body = CreateDropletRequest {
        name: name.to_string(),
        region: region.to_string(),
        size: size.to_string(),
        image: "ubuntu-22-04-x64".to_string(),
        ssh_keys: vec![ssh_key_id],
    };

    let resp: DropletResponse = client.post("/droplets", &body).await?;
    Ok(resp.droplet.id)
}

/// Poll the Droplet until it reaches `active` status, then return its public IPv4.
/// Polls every 5 seconds for up to 5 minutes.
pub async fn wait_for_active(client: &DoClient, droplet_id: u64) -> Result<String, AppError> {
    let max_attempts = 60; // 60 × 5s = 5 minutes

    for attempt in 0..max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let resp: DropletResponse = client
            .get(&format!("/droplets/{}", droplet_id))
            .await?;

        tracing::debug!(
            "[DO wait_for_active] attempt {}/{}: status={:?}",
            attempt + 1,
            max_attempts,
            resp.droplet.status
        );

        if resp.droplet.status.as_deref() == Some("active") {
            if let Some(networks) = resp.droplet.networks {
                for net in networks.v4 {
                    if net.network_type == "public" {
                        return Ok(net.ip_address);
                    }
                }
            }
            // Active but no public IP yet — continue polling
        }
    }

    Err(AppError::General(
        "Droplet did not become active within 5 minutes".into(),
    ))
}

/// Delete a DigitalOcean Droplet.
/// DELETE /v2/droplets/{id}
pub async fn delete_droplet(client: &DoClient, droplet_id: u64) -> Result<(), AppError> {
    client.delete(&format!("/droplets/{}", droplet_id)).await
}
