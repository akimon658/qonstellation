use axum::extract::{Query, State};
use axum::http::{
    HeaderMap, StatusCode,
    header::{HOST, SET_COOKIE},
};
use axum::response::{IntoResponse, Redirect, Response};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;
use crate::repository::user;
use crate::web::middleware::auth;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
}

pub fn build_redirect_uri(headers: &HeaderMap) -> String {
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("http");
    let host = headers
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .or_else(|| headers.get(HOST).and_then(|v| v.to_str().ok()))
        .unwrap_or("localhost");
    format!("{}://{}/callback", proto, host)
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let config = &state.config;
    let redirect_uri = build_redirect_uri(&headers);

    let Ok(token_url) = TokenUrl::new(format!("{}/api/v3/oauth2/token", config.traq_base_url))
    else {
        tracing::error!("Invalid token URL");
        return (StatusCode::INTERNAL_SERVER_ERROR, "OAuth config error").into_response();
    };
    let Ok(auth_url) = AuthUrl::new(format!("{}/api/v3/oauth2/authorize", config.traq_base_url))
    else {
        tracing::error!("Invalid auth URL");
        return (StatusCode::INTERNAL_SERVER_ERROR, "OAuth config error").into_response();
    };

    let Ok(redirect_url) = RedirectUrl::new(redirect_uri) else {
        tracing::error!("Invalid redirect URI");
        return (StatusCode::INTERNAL_SERVER_ERROR, "OAuth config error").into_response();
    };
    let oauth_client = BasicClient::new(ClientId::new(config.traq_client_id.clone()))
        .set_client_secret(ClientSecret::new(config.traq_client_secret.clone()))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url);

    let token_result = match oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&state.http_client)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Failed to exchange token: {}", e);
            return (StatusCode::BAD_REQUEST, "Token exchange failed").into_response();
        }
    };

    let access_token = token_result.access_token().secret();

    if access_token.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing access token").into_response();
    }

    let me_url = format!("{}/api/v3/users/me", config.traq_base_url);
    let user_response = match state
        .http_client
        .get(&me_url)
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to get user info: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "User info fetch failed").into_response();
        }
    };

    if !user_response.status().is_success() {
        return (StatusCode::UNAUTHORIZED, "Failed to authenticate").into_response();
    }

    let user_data: serde_json::Value = match user_response.json().await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to parse user data: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "User data parse error").into_response();
        }
    };

    let user_id = user_data["id"].as_str().unwrap_or("");

    if user_id.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing user ID").into_response();
    }

    let user_id = match uuid::Uuid::parse_str(user_id) {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Invalid user ID format: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid user ID").into_response();
        }
    };

    // Save user and tokens. Fail the login if persistence fails so we
    // don't issue a session cookie without a matching DB row.
    if let Err(e) = user::save_user(&state.pool, user_id).await {
        tracing::error!("Failed to save user: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save user").into_response();
    }

    if let Err(e) = user::save_user_tokens(&state.pool, user_id, access_token).await {
        tracing::error!("Failed to save tokens: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save tokens").into_response();
    }

    let token = match auth::create_token(&user_id.to_string(), &config.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create JWT: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed").into_response();
        }
    };

    let cookie = auth::build_session_cookie(&token);

    let mut response = Redirect::to("/").into_response();
    if let Ok(header_value) = cookie.to_string().parse() {
        response.headers_mut().insert(SET_COOKIE, header_value);
    }

    response
}
