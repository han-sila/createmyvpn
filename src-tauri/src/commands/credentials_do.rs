use crate::do_cloud::client::DoClient;
use crate::error::AppError;
use crate::persistence::store;
use crate::state::DoCredentials;

#[tauri::command]
pub async fn validate_do_credentials(api_token: String) -> Result<String, AppError> {
    let email = DoClient::validate(&api_token).await?;
    Ok(email)
}

#[tauri::command]
pub async fn save_do_credentials(api_token: String) -> Result<(), AppError> {
    let creds = DoCredentials { api_token };
    store::save_do_credentials(&creds)
}

#[tauri::command]
pub async fn load_do_credentials() -> Result<Option<DoCredentials>, AppError> {
    store::load_do_credentials()
}

#[tauri::command]
pub async fn delete_do_credentials() -> Result<(), AppError> {
    store::delete_do_credentials()
}
