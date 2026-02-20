use crate::do_cloud::client::DoClient;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct CreateKeyRequest {
    name: String,
    public_key: String,
}

#[derive(Deserialize)]
struct CreateKeyResponse {
    ssh_key: SshKeyInfo,
}

#[derive(Deserialize)]
struct SshKeyInfo {
    id: u64,
}

/// Upload an SSH public key to DigitalOcean and return the key ID.
/// POST /v2/account/keys
pub async fn upload_ssh_key(
    client: &DoClient,
    name: &str,
    public_key: &str,
) -> Result<u64, AppError> {
    let body = CreateKeyRequest {
        name: name.to_string(),
        public_key: public_key.to_string(),
    };

    let resp: CreateKeyResponse = client.post("/account/keys", &body).await?;
    Ok(resp.ssh_key.id)
}

/// Delete an SSH key from DigitalOcean.
/// DELETE /v2/account/keys/{id}
pub async fn delete_ssh_key(client: &DoClient, key_id: u64) -> Result<(), AppError> {
    client.delete(&format!("/account/keys/{}", key_id)).await
}
