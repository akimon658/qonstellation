use crate::database::DbPool;
use uuid::Uuid;

pub async fn get_traq_message_id_by_at_proto_uri(
    pool: &DbPool,
    at_proto_uri: &str,
) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query_scalar!(
        "SELECT traq_message_id FROM posts WHERE at_proto_uri = ?",
        at_proto_uri
    )
    .fetch_optional(pool)
    .await?;

    let Some(bytes) = row else {
        return Ok(None);
    };
    let id = Uuid::from_slice(&bytes)
        .map_err(|e| anyhow::anyhow!("Invalid UUID bytes in traq_message_id: {e}"))?;
    Ok(Some(id))
}

pub async fn save_post_metadata(
    pool: &DbPool,
    at_proto_uri: &str,
    traq_message_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO posts (at_proto_uri, traq_message_id) VALUES (?, ?) ON DUPLICATE KEY UPDATE traq_message_id = ?",
        at_proto_uri, traq_message_id, traq_message_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
