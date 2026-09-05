use base64::{Engine, engine::general_purpose};
use std::env;

#[derive(Clone)]
pub struct Config {
    pub bluesky_account_identifier: String,
    pub bluesky_app_password: String,
    pub bluesky_hosting_provider: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub jwt_secret: Vec<u8>,
    pub traq_base_url: String,
    pub traq_client_id: String,
    pub traq_client_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_secret_b64 = env::var("QONSTELLATION_JWT_SECRET")?;
        let jwt_secret = general_purpose::STANDARD
            .decode(&jwt_secret_b64)
            .map_err(|e| anyhow::anyhow!("QONSTELLATION_JWT_SECRET is not valid base64: {}", e))?;

        Ok(Self {
            bluesky_account_identifier: env::var("BLUESKY_ACCOUNT_IDENTIFIER")?,
            bluesky_app_password: env::var("BLUESKY_APP_PASSWORD")?,
            bluesky_hosting_provider: env::var("BLUESKY_HOSTING_PROVIDER")
                .unwrap_or_else(|_| "https://bsky.social".to_string()),
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            db_port: env::var("DB_PORT")
                .unwrap_or_else(|_| "3306".to_string())
                .parse()?,
            db_user: env::var("DB_USER").unwrap_or_else(|_| "root".to_string()),
            db_password: env::var("DB_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            db_name: env::var("DB_NAME").unwrap_or_else(|_| "qonstellation".to_string()),
            jwt_secret,
            traq_base_url: env::var("TRAQ_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            traq_client_id: env::var("TRAQ_CLIENT_ID")?,
            traq_client_secret: env::var("TRAQ_CLIENT_SECRET")?,
        })
    }
}
