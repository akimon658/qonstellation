pub mod handlers;
pub mod middleware;

use axum::Router;
use axum::routing::{get, put};
use std::sync::Arc;

use crate::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(handlers::index::handler))
        .route("/callback", get(handlers::callback::handler))
        .route("/api/settings", get(handlers::settings::get_handler))
        .route("/api/settings", put(handlers::settings::put_handler))
        .with_state(state)
}
