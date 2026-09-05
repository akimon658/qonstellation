use crate::database::DbPool;
use crate::model::jetstream::JETSTREAM_CURSOR_KEY;

pub async fn get_jetstream_cursor(pool: &DbPool) -> anyhow::Result<Option<i64>> {
    let row = sqlx::query_scalar!(
        "SELECT value FROM system_states WHERE `key` = ?",
        JETSTREAM_CURSOR_KEY
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn save_jetstream_cursor(pool: &DbPool, cursor: i64) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO system_states (`key`, value) VALUES (?, ?) ON DUPLICATE KEY UPDATE value = ?",
        JETSTREAM_CURSOR_KEY,
        cursor,
        cursor
    )
    .execute(pool)
    .await?;

    Ok(())
}
