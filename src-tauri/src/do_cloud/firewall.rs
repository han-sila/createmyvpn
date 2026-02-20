use crate::do_cloud::client::DoClient;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct FirewallSources {
    addresses: Vec<String>,
}

#[derive(Serialize)]
struct InboundRule {
    protocol: String,
    ports: String,
    sources: FirewallSources,
}

#[derive(Serialize)]
struct FirewallDestinations {
    addresses: Vec<String>,
}

#[derive(Serialize)]
struct OutboundRule {
    protocol: String,
    ports: String,
    destinations: FirewallDestinations,
}

#[derive(Serialize)]
struct CreateFirewallRequest {
    name: String,
    inbound_rules: Vec<InboundRule>,
    outbound_rules: Vec<OutboundRule>,
    droplet_ids: Vec<u64>,
}

#[derive(Deserialize)]
struct CreateFirewallResponse {
    firewall: FirewallInfo,
}

#[derive(Deserialize)]
struct FirewallInfo {
    id: String,
}

/// Create a firewall allowing SSH (TCP 22) and WireGuard (UDP port) inbound,
/// all traffic outbound, and attach it to the given droplet.
/// POST /v2/firewalls — returns firewall UUID.
pub async fn create_firewall(
    client: &DoClient,
    droplet_id: u64,
    wireguard_port: u16,
) -> Result<String, AppError> {
    let all_addrs = vec!["0.0.0.0/0".to_string(), "::/0".to_string()];

    let body = CreateFirewallRequest {
        name: "createmyvpn-firewall".to_string(),
        inbound_rules: vec![
            InboundRule {
                protocol: "tcp".to_string(),
                ports: "22".to_string(),
                sources: FirewallSources {
                    addresses: all_addrs.clone(),
                },
            },
            InboundRule {
                protocol: "udp".to_string(),
                ports: wireguard_port.to_string(),
                sources: FirewallSources {
                    addresses: all_addrs.clone(),
                },
            },
        ],
        outbound_rules: vec![
            OutboundRule {
                protocol: "tcp".to_string(),
                ports: "all".to_string(),
                destinations: FirewallDestinations {
                    addresses: all_addrs.clone(),
                },
            },
            OutboundRule {
                protocol: "udp".to_string(),
                ports: "all".to_string(),
                destinations: FirewallDestinations {
                    addresses: all_addrs.clone(),
                },
            },
            OutboundRule {
                protocol: "icmp".to_string(),
                ports: "0".to_string(),
                destinations: FirewallDestinations {
                    addresses: all_addrs,
                },
            },
        ],
        droplet_ids: vec![droplet_id],
    };

    let resp: CreateFirewallResponse = client.post("/firewalls", &body).await?;
    Ok(resp.firewall.id)
}

/// Delete a DigitalOcean firewall.
/// DELETE /v2/firewalls/{id}
pub async fn delete_firewall(client: &DoClient, firewall_id: &str) -> Result<(), AppError> {
    client.delete(&format!("/firewalls/{}", firewall_id)).await
}
