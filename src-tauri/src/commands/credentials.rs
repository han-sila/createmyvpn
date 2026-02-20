use crate::aws;
use crate::error::AppError;
use crate::persistence::store;
use crate::state::AwsCredentials;

#[tauri::command]
pub async fn validate_credentials(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<String, AppError> {
    let creds = AwsCredentials {
        access_key_id,
        secret_access_key,
    };

    let account_id = aws::client::validate_credentials(&creds, &region).await?;
    Ok(account_id)
}

#[tauri::command]
pub async fn save_credentials(
    access_key_id: String,
    secret_access_key: String,
) -> Result<(), AppError> {
    let creds = AwsCredentials {
        access_key_id,
        secret_access_key,
    };
    store::save_credentials(&creds)?;
    Ok(())
}

#[tauri::command]
pub async fn load_credentials() -> Result<Option<AwsCredentials>, AppError> {
    store::load_credentials()
}

#[tauri::command]
pub async fn delete_credentials() -> Result<(), AppError> {
    store::delete_credentials()
}
