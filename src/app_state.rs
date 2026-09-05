use crate::app_config::config::Config;
use crate::database::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
    pub http_client: reqwest::Client,
}
