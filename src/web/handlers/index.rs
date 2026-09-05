use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use maud::html;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl};
use std::sync::Arc;

use crate::AppState;
use crate::repository::user;
use crate::web::handlers::callback::build_redirect_uri;
use crate::web::middleware::auth::extract_user_id_from_headers;

pub async fn handler(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    let redirect_uri = build_redirect_uri(&headers);

    let user_id = extract_user_id_from_headers(&headers, &state.config.jwt_secret)
        .and_then(|s| uuid::Uuid::parse_str(&s).ok());

    if let Some(user_id) = user_id {
        let (did, target_channel_id) =
            match user::get_user_setting_by_user_id(&state.pool, user_id).await {
                Ok(Some((d, c))) => (d, c.to_string()),
                Ok(None) => (String::new(), String::new()),
                Err(e) => {
                    tracing::error!("Failed to load settings: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to load settings",
                    )
                        .into_response();
                }
            };

        let script = r#"document.getElementById('settings-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const did = document.getElementById('did').value;
    const targetChannelId = document.getElementById('target-channel-id').value;
    const res = await fetch('/api/settings', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ did, targetChannelId }),
    });
    document.getElementById('message').textContent = res.ok ? 'Saved' : 'Failed to save';
});"#;

        let html = html! {
            html lang="ja" {
                head {
                    meta charset="UTF-8";
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                    title { "Qonstellation" }
                }
                body {
                    h1 { "Qonstellation" }
                    h2 { "Settings" }
                    form id="settings-form" {
                        div {
                            label { "DID:" }
                            input type="text" id="did" value=(did) style="width: 100%";
                        }
                        div {
                            label { "Target Channel ID:" }
                            input type="text" id="target-channel-id" value=(target_channel_id);
                        }
                        div {
                            button type="submit" { "Save" }
                        }
                    }
                    div id="message" {}
                    script { (maud::PreEscaped(script)) }
                }
            }
        };

        return Html(html.into_string()).into_response();
    }

    // Build OAuth client inline to avoid Typestate issues
    let Ok(auth_url_obj) = AuthUrl::new(format!(
        "{}/api/v3/oauth2/authorize",
        state.config.traq_base_url
    )) else {
        tracing::error!("Invalid auth URL");
        return (StatusCode::INTERNAL_SERVER_ERROR, "OAuth config error").into_response();
    };
    let Ok(token_url_obj) = TokenUrl::new(format!(
        "{}/api/v3/oauth2/token",
        state.config.traq_base_url
    )) else {
        tracing::error!("Invalid token URL");
        return (StatusCode::INTERNAL_SERVER_ERROR, "OAuth config error").into_response();
    };
    let Ok(redirect_url_obj) = RedirectUrl::new(redirect_uri) else {
        tracing::error!("Invalid redirect URI");
        return (StatusCode::INTERNAL_SERVER_ERROR, "OAuth config error").into_response();
    };
    let oauth_client = BasicClient::new(ClientId::new(state.config.traq_client_id.clone()))
        .set_client_secret(ClientSecret::new(state.config.traq_client_secret.clone()))
        .set_auth_uri(auth_url_obj)
        .set_token_uri(token_url_obj)
        .set_redirect_uri(redirect_url_obj);

    let (auth_url, _csrf_token) = oauth_client.authorize_url(CsrfToken::new_random).url();

    let html = html! {
        html lang="ja" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Qonstellation" }
            }
            body {
                h1 { "Qonstellation" }
                a href=(auth_url.as_str()) { "Login with traQ" }
            }
        }
    };

    Html(html.into_string()).into_response()
}
