use serde_json::Value;
use sqlx::FromRow;

use crate::database::DbPool;

#[derive(FromRow)]
pub struct QueuedEvent {
    pub id: i64,
    pub event_json: Value,
}

pub async fn get_oldest_queued_event(pool: &DbPool) -> anyhow::Result<Option<QueuedEvent>> {
    let row = sqlx::query_as!(
        QueuedEvent,
        r#"SELECT id, event_json AS "event_json: serde_json::Value" FROM queued_events ORDER BY id ASC LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn add_queued_event(pool: &DbPool, event: &Value) -> anyhow::Result<()> {
    sqlx::query!("INSERT INTO queued_events (event_json) VALUES (?)", event)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_queued_event(pool: &DbPool, id: i64) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM queued_events WHERE id = ?", id)
        .execute(pool)
        .await?;

    Ok(())
}
