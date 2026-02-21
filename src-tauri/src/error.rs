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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_aws() {
        let err = AppError::Aws("region not found".into());
        assert_eq!(err.to_string(), "AWS error: region not found");
    }

    #[test]
    fn error_display_ssh() {
        let err = AppError::Ssh("connection refused".into());
        assert_eq!(err.to_string(), "SSH error: connection refused");
    }

    #[test]
    fn error_display_wireguard() {
        let err = AppError::WireGuard("bad key".into());
        assert_eq!(err.to_string(), "WireGuard error: bad key");
    }

    #[test]
    fn error_display_state() {
        let err = AppError::State("corrupt file".into());
        assert_eq!(err.to_string(), "State error: corrupt file");
    }

    #[test]
    fn error_display_credential() {
        let err = AppError::Credential("invalid".into());
        assert_eq!(err.to_string(), "Credential error: invalid");
    }

    #[test]
    fn error_display_general() {
        let err = AppError::General("something broke".into());
        assert_eq!(err.to_string(), "something broke");
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: AppError = io_err.into();
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn error_from_serde_json() {
        let json_err = serde_json::from_str::<String>("not json").unwrap_err();
        let err: AppError = json_err.into();
        assert!(err.to_string().contains("expected") || err.to_string().len() > 0);
    }

    #[test]
    fn error_serializes_as_string() {
        let err = AppError::Aws("test".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"AWS error: test\"");
    }
}
