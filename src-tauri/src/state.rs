use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    NotDeployed,
    Deploying,
    Deployed,
    Destroying,
    Failed,
}

impl Default for DeploymentStatus {
    fn default() -> Self {
        DeploymentStatus::NotDeployed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VpnConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

impl Default for VpnConnectionStatus {
    fn default() -> Self {
        VpnConnectionStatus::Disconnected
    }
}

/// Tracks every AWS/DO resource created so we can tear down safely.
/// Each field is set immediately after the resource is created.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentState {
    pub status: DeploymentStatus,
    /// "aws", "byo", or "do". None in old state files is treated as "aws".
    pub deployment_mode: Option<String>,
    pub region: Option<String>,
    // AWS-specific fields
    pub vpc_id: Option<String>,
    pub igw_id: Option<String>,
    pub subnet_id: Option<String>,
    pub route_table_id: Option<String>,
    pub security_group_id: Option<String>,
    pub key_pair_name: Option<String>,
    pub instance_id: Option<String>,
    pub allocation_id: Option<String>,
    pub association_id: Option<String>,
    // Shared fields
    pub elastic_ip: Option<String>,
    pub ssh_private_key: Option<String>,
    pub ssh_user: Option<String>,
    pub server_public_key: Option<String>,
    pub client_private_key: Option<String>,
    pub client_public_key: Option<String>,
    pub client_config: Option<String>,
    pub deployed_at: Option<DateTime<Utc>>,
    pub auto_destroy_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    // DigitalOcean-specific fields
    pub droplet_id: Option<u64>,
    pub do_firewall_id: Option<String>,
    pub do_ssh_key_id: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    pub region: String,
    pub instance_type: String,
    pub wireguard_port: u16,
}

impl AppSettings {
    pub fn new() -> Self {
        AppSettings {
            region: "us-east-1".to_string(),
            instance_type: "t2.micro".to_string(),
            wireguard_port: 51820,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoCredentials {
    pub api_token: String,
}

/// Progress event sent to the frontend during deploy/destroy
#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub step: u32,
    pub total_steps: u32,
    pub message: String,
    pub status: String, // "running", "done", "error"
}
