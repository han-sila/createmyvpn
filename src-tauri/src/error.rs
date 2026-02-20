use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("AWS error: {0}")]
    Aws(String),

    #[error("SSH error: {0}")]
    Ssh(String),

    #[error("WireGuard error: {0}")]
    WireGuard(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("State error: {0}")]
    State(String),

    #[error("Credential error: {0}")]
    Credential(String),

    #[error("{0}")]
    General(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::State(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::General(e.to_string())
    }
}
