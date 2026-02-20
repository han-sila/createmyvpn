use crate::error::AppError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const BASE_URL: &str = "https://api.digitalocean.com/v2";

pub struct DoClient {
    http: reqwest::Client,
    token: String,
}

impl DoClient {
    pub fn new(token: &str) -> Self {
        DoClient {
            http: reqwest::Client::new(),
            token: token.to_string(),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        let url = format!("{}{}", BASE_URL, path);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| AppError::General(format!("DO API request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::General(format!(
                "DO API error {}: {}",
                status, body
            )));
        }

        resp.json::<T>()
            .await
            .map_err(|e| AppError::General(format!("DO API response parse error: {}", e)))
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let url = format!("{}{}", BASE_URL, path);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::General(format!("DO API request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::General(format!(
                "DO API error {}: {}",
                status, body
            )));
        }

        resp.json::<T>()
            .await
            .map_err(|e| AppError::General(format!("DO API response parse error: {}", e)))
    }

    pub async fn delete(&self, path: &str) -> Result<(), AppError> {
        let url = format!("{}{}", BASE_URL, path);
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| AppError::General(format!("DO API request failed: {}", e)))?;

        let status = resp.status();
        // DELETE returns 204 No Content on success
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::General(format!(
                "DO API error {}: {}",
                status, body
            )));
        }

        Ok(())
    }

    /// Validate a DigitalOcean API token by calling GET /v2/account.
    /// Returns the account email on success.
    pub async fn validate(token: &str) -> Result<String, AppError> {
        let client = DoClient::new(token);

        #[derive(Deserialize)]
        struct AccountResponse {
            account: Account,
        }

        #[derive(Deserialize)]
        struct Account {
            email: String,
        }

        let resp: AccountResponse = client.get("/account").await?;
        Ok(resp.account.email)
    }
}
