use atrium_api::types::string::Did;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::repository::user;
use crate::web::middleware::auth::extract_user_id_from_headers;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPayload {
    did: String,
    target_channel_id: String,
}

pub async fn get_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_headers(&headers, &state.config.jwt_secret)
        .and_then(|s| Uuid::parse_str(&s).ok())
    {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Unauthorized"})),
            )
                .into_response();
        }
    };

    match user::get_user_setting_by_user_id(&state.pool, user_id).await {
        Ok(Some((did, target_channel_id))) => {
            let response = serde_json::json!({
                "did": did,
                "targetChannelId": target_channel_id.to_string(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({}))).into_response(),
        Err(e) => {
            tracing::error!("Failed to load settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load settings"})),
            )
                .into_response()
        }
    }
}

pub async fn put_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SettingsPayload>,
) -> impl IntoResponse {
    let user_id = match extract_user_id_from_headers(&headers, &state.config.jwt_secret)
        .and_then(|s| Uuid::parse_str(&s).ok())
    {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Unauthorized"})),
            )
                .into_response();
        }
    };

    if payload.did.parse::<Did>().is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid DID"})),
        )
            .into_response();
    }

    let target_channel_id = match Uuid::parse_str(&payload.target_channel_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid target channel ID"})),
            )
                .into_response();
        }
    };

    match user::save_user_settings(&state.pool, user_id, &payload.did, target_channel_id).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(e) => {
            tracing::error!("Failed to save settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to save"})),
            )
                .into_response()
        }
    }
}
