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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deployment_status_default_is_not_deployed() {
        assert_eq!(DeploymentStatus::default(), DeploymentStatus::NotDeployed);
    }

    #[test]
    fn vpn_connection_status_default_is_disconnected() {
        assert_eq!(VpnConnectionStatus::default(), VpnConnectionStatus::Disconnected);
    }

    #[test]
    fn deployment_state_default_has_no_resources() {
        let state = DeploymentState::default();
        assert_eq!(state.status, DeploymentStatus::NotDeployed);
        assert!(state.vpc_id.is_none());
        assert!(state.instance_id.is_none());
        assert!(state.elastic_ip.is_none());
        assert!(state.client_config.is_none());
        assert!(state.droplet_id.is_none());
        assert!(state.deployment_mode.is_none());
    }

    #[test]
    fn deployment_state_serde_roundtrip() {
        let mut state = DeploymentState::default();
        state.status = DeploymentStatus::Deployed;
        state.region = Some("us-east-1".to_string());
        state.vpc_id = Some("vpc-123".to_string());
        state.elastic_ip = Some("1.2.3.4".to_string());
        state.deployment_mode = Some("aws".to_string());

        let json = serde_json::to_string(&state).unwrap();
        let restored: DeploymentState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.status, DeploymentStatus::Deployed);
        assert_eq!(restored.region, Some("us-east-1".to_string()));
        assert_eq!(restored.vpc_id, Some("vpc-123".to_string()));
        assert_eq!(restored.elastic_ip, Some("1.2.3.4".to_string()));
        assert_eq!(restored.deployment_mode, Some("aws".to_string()));
    }

    #[test]
    fn deployment_status_serializes_as_snake_case() {
        let json = serde_json::to_string(&DeploymentStatus::NotDeployed).unwrap();
        assert_eq!(json, "\"not_deployed\"");

        let json = serde_json::to_string(&DeploymentStatus::Deploying).unwrap();
        assert_eq!(json, "\"deploying\"");

        let json = serde_json::to_string(&DeploymentStatus::Deployed).unwrap();
        assert_eq!(json, "\"deployed\"");
    }

    #[test]
    fn vpn_status_serializes_as_snake_case() {
        let json = serde_json::to_string(&VpnConnectionStatus::Disconnected).unwrap();
        assert_eq!(json, "\"disconnected\"");

        let json = serde_json::to_string(&VpnConnectionStatus::Connected).unwrap();
        assert_eq!(json, "\"connected\"");
    }

    #[test]
    fn app_settings_default_values() {
        let settings = AppSettings::new();
        assert_eq!(settings.region, "us-east-1");
        assert_eq!(settings.instance_type, "t2.micro");
        assert_eq!(settings.wireguard_port, 51820);
    }

    #[test]
    fn app_settings_serde_roundtrip() {
        let settings = AppSettings {
            region: "eu-west-1".to_string(),
            instance_type: "t3.micro".to_string(),
            wireguard_port: 9999,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.region, "eu-west-1");
        assert_eq!(restored.wireguard_port, 9999);
    }

    #[test]
    fn credentials_serde_roundtrip() {
        let creds = AwsCredentials {
            access_key_id: "AKID".to_string(),
            secret_access_key: "SECRET".to_string(),
        };
        let json = serde_json::to_string(&creds).unwrap();
        let restored: AwsCredentials = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.access_key_id, "AKID");
    }

    #[test]
    fn do_credentials_serde_roundtrip() {
        let creds = DoCredentials {
            api_token: "dop_v1_abc".to_string(),
        };
        let json = serde_json::to_string(&creds).unwrap();
        let restored: DoCredentials = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.api_token, "dop_v1_abc");
    }

    #[test]
    fn progress_event_serializes() {
        let event = ProgressEvent {
            step: 3,
            total_steps: 10,
            message: "Creating VPC".to_string(),
            status: "running".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Creating VPC"));
        assert!(json.contains("\"step\":3"));
    }
}
