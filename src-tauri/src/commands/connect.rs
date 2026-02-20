use crate::error::AppError;
use crate::persistence::store;
use crate::state::VpnConnectionStatus;
use crate::wireguard::tunnel;

#[tauri::command]
pub async fn connect_vpn() -> Result<(), AppError> {
    tracing::info!("=== VPN Connect requested ===");
    let state = store::load_state()?;
    let config = state
        .client_config
        .ok_or_else(|| AppError::State("No client config available".into()))?;
    tracing::info!("Client config loaded, activating tunnel...");
    match tunnel::activate_tunnel(&config) {
        Ok(()) => {
            tracing::info!("=== VPN Connected successfully ===");
            Ok(())
        }
        Err(e) => {
            tracing::error!("VPN connection failed: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn disconnect_vpn() -> Result<(), AppError> {
    tracing::info!("=== VPN Disconnect requested ===");
    tunnel::deactivate_tunnel()?;
    tracing::info!("=== VPN Disconnected ===");
    Ok(())
}

#[tauri::command]
pub async fn get_vpn_status() -> Result<VpnConnectionStatus, AppError> {
    if tunnel::is_tunnel_active() {
        Ok(VpnConnectionStatus::Connected)
    } else {
        Ok(VpnConnectionStatus::Disconnected)
    }
}

#[tauri::command]
pub async fn get_client_config() -> Result<Option<String>, AppError> {
    let state = store::load_state()?;
    Ok(state.client_config)
}
