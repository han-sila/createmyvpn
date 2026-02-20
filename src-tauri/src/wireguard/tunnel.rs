use crate::error::AppError;
use crate::persistence::store;

/// Activate the WireGuard tunnel using the built-in userspace engine.
/// No `wg-quick`, no kernel module required. Works on Linux, WSL2, macOS, Windows.
pub fn activate_tunnel(client_config: &str) -> Result<(), AppError> {
    store::save_client_config(client_config)?;
    super::userspace::connect(client_config)
}

/// Deactivate the WireGuard tunnel.
pub fn deactivate_tunnel() -> Result<(), AppError> {
    super::userspace::disconnect()
}

/// Returns true if the tunnel is currently active.
pub fn is_tunnel_active() -> bool {
    super::userspace::is_active()
}