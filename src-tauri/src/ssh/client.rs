use std::sync::Arc;

use russh::client;

use crate::error::AppError;

struct SshHandler;

#[async_trait::async_trait]
impl client::Handler for SshHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all server keys (like StrictHostKeyChecking=no)
        Ok(true)
    }
}

pub struct SshSession {
    session: client::Handle<SshHandler>,
}

impl SshSession {
    /// Connect to an SSH server with retry up to `timeout_secs`.
    pub async fn connect(
        host: &str,
        port: u16,
        user: &str,
        private_key_pem: &str,
        timeout_secs: u64,
    ) -> Result<Self, AppError> {
        let config = Arc::new(client::Config::default());

        let key_pair = russh_keys::decode_secret_key(private_key_pem, None)
            .map_err(|e| AppError::Ssh(format!("Failed to decode SSH key: {}", e)))?;

        let start = std::time::Instant::now();
        let deadline = std::time::Duration::from_secs(timeout_secs);

        loop {
            match client::connect(config.clone(), (host, port), SshHandler).await {
                Ok(mut handle) => {
                    let auth_ok = handle
                        .authenticate_publickey(user, Arc::new(key_pair.clone()))
                        .await
                        .map_err(|e| AppError::Ssh(format!("SSH auth failed: {}", e)))?;

                    if !auth_ok {
                        return Err(AppError::Ssh("SSH authentication rejected".into()));
                    }

                    tracing::info!("SSH connected to {}:{}", host, port);
                    return Ok(SshSession { session: handle });
                }
                Err(e) => {
                    if start.elapsed() > deadline {
                        return Err(AppError::Ssh(format!(
                            "SSH connection timeout after {}s: {}",
                            timeout_secs, e
                        )));
                    }
                    tracing::debug!("SSH connect attempt failed, retrying: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Execute a command and return stdout as a string.
    pub async fn execute(&self, command: &str) -> Result<String, AppError> {
        let mut channel = self
            .session
            .channel_open_session()
            .await
            .map_err(|e| AppError::Ssh(format!("Failed to open channel: {}", e)))?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| AppError::Ssh(format!("Failed to exec command: {}", e)))?;

        let mut output = Vec::new();

        loop {
            match channel.wait().await {
                Some(russh::ChannelMsg::Data { ref data }) => {
                    output.extend_from_slice(data);
                }
                Some(russh::ChannelMsg::ExtendedData { ref data, .. }) => {
                    output.extend_from_slice(data);
                }
                Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                    if exit_status != 0 {
                        let out = String::from_utf8_lossy(&output).to_string();
                        return Err(AppError::Ssh(format!(
                            "Command '{}' exited with status {}: {}",
                            command, exit_status, out
                        )));
                    }
                }
                None => break,
                _ => {}
            }
        }

        Ok(String::from_utf8_lossy(&output).to_string())
    }

    /// Upload file content to a remote path.
    pub async fn upload_file(&self, remote_path: &str, content: &str) -> Result<(), AppError> {
        // Write via echo command to avoid needing SFTP
        let escaped = content.replace('\\', "\\\\").replace('\'', "'\\''");
        let cmd = format!("echo '{}' | sudo tee {} > /dev/null", escaped, remote_path);
        self.execute(&cmd).await?;
        Ok(())
    }
}
