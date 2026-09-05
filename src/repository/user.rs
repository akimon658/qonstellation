use crate::database::DbPool;
use uuid::Uuid;

pub async fn get_all_dids(pool: &DbPool) -> anyhow::Result<Vec<String>> {
    let rows = sqlx::query_scalar!("SELECT did FROM user_settings")
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub async fn get_user_setting_by_did(pool: &DbPool, did: &str) -> anyhow::Result<(Uuid, Uuid)> {
    let row = sqlx::query!(
        "SELECT user_id, target_channel_id FROM user_settings WHERE did = ?",
        did
    )
    .fetch_one(pool)
    .await?;

    let user_id = Uuid::from_slice(&row.user_id)?;
    let target_channel_id = Uuid::from_slice(&row.target_channel_id)?;
    Ok((user_id, target_channel_id))
}

pub async fn get_user_setting_by_user_id(
    pool: &DbPool,
    user_id: Uuid,
) -> anyhow::Result<Option<(String, Uuid)>> {
    let row = sqlx::query!(
        "SELECT did, target_channel_id FROM user_settings WHERE user_id = ?",
        user_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else {
        return Ok(None);
    };
    let target_channel_id = Uuid::from_slice(&r.target_channel_id)
        .map_err(|e| anyhow::anyhow!("Invalid UUID bytes in target_channel_id: {e}"))?;
    Ok(Some((r.did, target_channel_id)))
}

pub async fn save_user_settings(
    pool: &DbPool,
    user_id: Uuid,
    did: &str,
    target_channel_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO user_settings (user_id, did, target_channel_id) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE did = ?, target_channel_id = ?",
        user_id, did, target_channel_id, did, target_channel_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_access_token(pool: &DbPool, user_id: Uuid) -> anyhow::Result<String> {
    let token = sqlx::query_scalar!(
        "SELECT access_token FROM user_tokens WHERE user_id = ?",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(token)
}

pub async fn save_user(pool: &DbPool, user_id: Uuid) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO users (id) VALUES (?) ON DUPLICATE KEY UPDATE id = id",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn save_user_tokens(
    pool: &DbPool,
    user_id: Uuid,
    access_token: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO user_tokens (user_id, access_token) VALUES (?, ?) ON DUPLICATE KEY UPDATE access_token = ?",
        user_id, access_token, access_token
    )
    .execute(pool)
    .await?;

    Ok(())
}
