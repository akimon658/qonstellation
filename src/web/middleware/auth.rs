use http::header::{COOKIE, HeaderMap};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tower_cookies::Cookie;
use tower_cookies::cookie::SameSite;

const COOKIE_NAME: &str = "qonstellation_session";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(user_id: &str, secret: &[u8]) -> anyhow::Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow::anyhow!("System time error: {e}"))?
        .as_secs() as usize;

    let claims = Claims {
        user_id: user_id.to_string(),
        exp: now + 86400,
        iat: now,
    };

    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )?)
}

pub fn verify_token(token: &str, secret: &[u8]) -> anyhow::Result<String> {
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(decoded.claims.user_id)
}

pub fn extract_user_id_from_headers(headers: &HeaderMap, secret: &[u8]) -> Option<String> {
    let cookie_str = headers.get(COOKIE)?.to_str().ok()?;
    let token = cookie_str.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        if name == COOKIE_NAME {
            Some(value.to_string())
        } else {
            None
        }
    })?;
    verify_token(&token, secret).ok()
}

pub fn build_session_cookie(token: &str) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, token.to_string()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}
