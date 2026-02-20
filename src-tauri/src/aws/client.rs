use aws_config::SdkConfig;
use aws_credential_types::Credentials;

use crate::error::AppError;
use crate::state::AwsCredentials;

/// Build an AWS SDK config from user-provided access key + secret.
pub async fn build_config(creds: &AwsCredentials, region: &str) -> Result<SdkConfig, AppError> {
    let credentials = Credentials::new(
        &creds.access_key_id,
        &creds.secret_access_key,
        None,
        None,
        "createmyvpn-user-credentials",
    );

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new(region.to_string()))
        .credentials_provider(credentials)
        .load()
        .await;

    Ok(config)
}

/// Validate credentials via STS GetCallerIdentity
pub async fn validate_credentials(
    creds: &AwsCredentials,
    region: &str,
) -> Result<String, AppError> {
    let config = build_config(creds, region).await?;
    let sts_client = aws_sdk_sts::Client::new(&config);

    let resp = sts_client
        .get_caller_identity()
        .send()
        .await
        .map_err(|e| AppError::Credential(format!("Invalid credentials: {}", e)))?;

    let account_id = resp
        .account()
        .unwrap_or("unknown")
        .to_string();

    Ok(account_id)
}
