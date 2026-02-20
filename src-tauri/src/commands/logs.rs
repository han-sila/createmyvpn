use crate::error::AppError;
use crate::persistence::store;

/// Returns the full content of the application log file.
/// If the file is very large, only the last ~500 KB is returned.
#[tauri::command]
pub async fn get_logs() -> Result<String, AppError> {
    let path = store::logs_dir()?.join("createmyvpn.log");
    if !path.exists() {
        return Ok(String::new());
    }
    let content = std::fs::read_to_string(&path)?;

    const MAX_CHARS: usize = 500_000;
    if content.len() > MAX_CHARS {
        let truncated = &content[content.len() - MAX_CHARS..];
        if let Some(idx) = truncated.find('\n') {
            return Ok(format!(
                "[... truncated, showing last ~500 KB ...]\n{}",
                &truncated[idx + 1..]
            ));
        }
        return Ok(truncated.to_string());
    }

    Ok(content)
}

/// Saves the log file to the user's Downloads folder and returns the saved path.
#[tauri::command]
pub async fn export_logs() -> Result<String, AppError> {
    let content = get_logs().await?;
    if content.trim().is_empty() {
        return Err(AppError::State("No logs to export".into()));
    }
    let filename = format!(
        "createmyvpn-logs-{}.log",
        chrono::Utc::now().format("%Y-%m-%d")
    );
    let path = store::save_to_downloads(&content, &filename)?;
    Ok(path.to_string_lossy().into_owned())
}

/// Clears the application log file.
#[tauri::command]
pub async fn clear_logs() -> Result<(), AppError> {
    let path = store::logs_dir()?.join("createmyvpn.log");
    if path.exists() {
        std::fs::write(&path, "")?;
    }
    Ok(())
}
