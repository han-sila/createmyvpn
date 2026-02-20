use crate::error::AppError;
use crate::state::{AppSettings, AwsCredentials, DeploymentState, DoCredentials};
use std::fs;
use std::path::PathBuf;

fn config_dir() -> Result<PathBuf, AppError> {
    let dir = dirs::home_dir()
        .ok_or_else(|| AppError::State("Cannot find home directory".into()))?
        .join(".createmyvpn");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn state_path() -> Result<PathBuf, AppError> {
    Ok(config_dir()?.join("state.json"))
}

fn credentials_path() -> Result<PathBuf, AppError> {
    Ok(config_dir()?.join("credentials.json"))
}

fn settings_path() -> Result<PathBuf, AppError> {
    Ok(config_dir()?.join("settings.json"))
}

fn do_credentials_path() -> Result<PathBuf, AppError> {
    Ok(config_dir()?.join("do_credentials.json"))
}

// --- Deployment State ---

pub fn load_state() -> Result<DeploymentState, AppError> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(DeploymentState::default());
    }
    let data = fs::read_to_string(&path)?;
    match serde_json::from_str(&data) {
        Ok(state) => Ok(state),
        Err(_) => {
            // Corrupt or schema-incompatible state file — delete it and start fresh.
            let _ = fs::remove_file(&path);
            Ok(DeploymentState::default())
        }
    }
}

pub fn save_state(state: &DeploymentState) -> Result<(), AppError> {
    let path = state_path()?;
    let data = serde_json::to_string_pretty(state)?;
    fs::write(&path, data)?;
    Ok(())
}

pub fn clear_state() -> Result<(), AppError> {
    let path = state_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

// --- Credentials ---

pub fn load_credentials() -> Result<Option<AwsCredentials>, AppError> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path)?;
    let creds: AwsCredentials = serde_json::from_str(&data)?;
    Ok(Some(creds))
}

pub fn save_credentials(creds: &AwsCredentials) -> Result<(), AppError> {
    let path = credentials_path()?;
    let data = serde_json::to_string_pretty(creds)?;
    fs::write(&path, data)?;

    // Restrict permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn delete_credentials() -> Result<(), AppError> {
    let path = credentials_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

// --- DigitalOcean Credentials ---

pub fn load_do_credentials() -> Result<Option<DoCredentials>, AppError> {
    let path = do_credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path)?;
    let creds: DoCredentials = serde_json::from_str(&data)?;
    Ok(Some(creds))
}

pub fn save_do_credentials(creds: &DoCredentials) -> Result<(), AppError> {
    let path = do_credentials_path()?;
    let data = serde_json::to_string_pretty(creds)?;
    fs::write(&path, data)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn delete_do_credentials() -> Result<(), AppError> {
    let path = do_credentials_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

// --- Settings ---

pub fn load_settings() -> Result<AppSettings, AppError> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(AppSettings::new());
    }
    let data = fs::read_to_string(&path)?;
    let settings: AppSettings = serde_json::from_str(&data)?;
    Ok(settings)
}

pub fn save_settings(settings: &AppSettings) -> Result<(), AppError> {
    let path = settings_path()?;
    let data = serde_json::to_string_pretty(settings)?;
    fs::write(&path, data)?;
    Ok(())
}

// --- Logs ---

pub fn logs_dir() -> Result<PathBuf, AppError> {
    let dir = config_dir()?.join("logs");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// --- File Export (saves to Downloads folder, returns the saved path) ---

pub fn save_to_downloads(content: &str, filename: &str) -> Result<std::path::PathBuf, AppError> {
    let dir = dirs::download_dir()
        .or_else(dirs::desktop_dir)
        .or_else(dirs::home_dir)
        .ok_or_else(|| AppError::State("Cannot find Downloads or Desktop directory".into()))?;
    let path = dir.join(filename);
    fs::write(&path, content)?;
    Ok(path)
}

// --- Client Config File ---

pub fn save_client_config(config: &str) -> Result<PathBuf, AppError> {
    let path = config_dir()?.join("client.conf");
    fs::write(&path, config)?;
    Ok(path)
}

pub fn client_config_path() -> Result<PathBuf, AppError> {
    Ok(config_dir()?.join("client.conf"))
}
