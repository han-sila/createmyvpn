use crate::error::AppError;
use crate::persistence::store;
use crate::state::AppSettings;
use serde::Serialize;

/// Saves the WireGuard client config to the user's Downloads folder.
/// Returns the full path of the saved file.
#[tauri::command]
pub async fn export_client_config() -> Result<String, AppError> {
    let config_path = store::client_config_path()?;
    if !config_path.exists() {
        return Err(AppError::State(
            "No VPN config found. Deploy a server first.".into(),
        ));
    }
    let content = std::fs::read_to_string(&config_path)?;
    let path = store::save_to_downloads(&content, "createmyvpn-client.conf")?;
    Ok(path.to_string_lossy().into_owned())
}

#[derive(Serialize)]
pub struct AwsRegion {
    pub code: String,
    pub name: String,
}

#[tauri::command]
pub async fn get_regions() -> Vec<AwsRegion> {
    vec![
        AwsRegion { code: "us-east-1".into(), name: "US East (N. Virginia)".into() },
        AwsRegion { code: "us-east-2".into(), name: "US East (Ohio)".into() },
        AwsRegion { code: "us-west-1".into(), name: "US West (N. California)".into() },
        AwsRegion { code: "us-west-2".into(), name: "US West (Oregon)".into() },
        AwsRegion { code: "eu-west-1".into(), name: "Europe (Ireland)".into() },
        AwsRegion { code: "eu-west-2".into(), name: "Europe (London)".into() },
        AwsRegion { code: "eu-central-1".into(), name: "Europe (Frankfurt)".into() },
        AwsRegion { code: "eu-north-1".into(), name: "Europe (Stockholm)".into() },
        AwsRegion { code: "ap-southeast-1".into(), name: "Asia Pacific (Singapore)".into() },
        AwsRegion { code: "ap-southeast-2".into(), name: "Asia Pacific (Sydney)".into() },
        AwsRegion { code: "ap-northeast-1".into(), name: "Asia Pacific (Tokyo)".into() },
        AwsRegion { code: "ap-south-1".into(), name: "Asia Pacific (Mumbai)".into() },
        AwsRegion { code: "sa-east-1".into(), name: "South America (São Paulo)".into() },
        AwsRegion { code: "ca-central-1".into(), name: "Canada (Central)".into() },
        AwsRegion { code: "me-south-1".into(), name: "Middle East (Bahrain)".into() },
        AwsRegion { code: "af-south-1".into(), name: "Africa (Cape Town)".into() },
    ]
}

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, AppError> {
    store::load_settings()
}

#[tauri::command]
pub async fn update_settings(region: String, instance_type: String, wireguard_port: u16) -> Result<(), AppError> {
    let settings = AppSettings {
        region,
        instance_type,
        wireguard_port,
    };
    store::save_settings(&settings)
}
