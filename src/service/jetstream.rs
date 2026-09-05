use std::sync::Arc;
use std::time::{Duration, Instant};

use atproto_jetstream::{Consumer, ConsumerTaskConfig, EventHandler, JetstreamEvent};
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::app_config::config::Config;
use crate::database::DbPool;
use crate::model::bluesky_types::build_at_proto_uri;
use crate::model::post_event::{PostCreateEvent, QueuedEventType};
use crate::repository::queued_event;
use crate::repository::system_state;
use crate::repository::user;
use crate::service::bluesky_client::BlueskyClient;

/// Debounce interval for persisting the Jetstream cursor.
/// Replays within this window are masked by the duplicate-post check.
const CURSOR_SAVE_DEBOUNCE: Duration = Duration::from_secs(30);
const NO_DIDS_RETRY: Duration = Duration::from_secs(30);
const MAX_BACKOFF: Duration = Duration::from_secs(300);

pub async fn start(
    endpoints: &[&str],
    initial_dids: &[String],
    initial_cursor: Option<i64>,
    pool: &DbPool,
    config: &Config,
    cancel_token: atproto_jetstream::CancellationToken,
    notify: Arc<Notify>,
) -> anyhow::Result<()> {
    let mut backoff_attempt: u32 = 0;
    // Fall back to the startup snapshot until the first DB refresh succeeds.
    let mut dids = initial_dids.to_vec();
    let mut cursor = initial_cursor;
    let mut dids_loaded = false;
    let cursor_saver = Arc::new(CursorSaver::new(pool.clone()));

    loop {
        if cancel_token.is_cancelled() {
            info!("Jetstream shutting down...");
            if let Err(e) = cursor_saver.flush().await {
                error!("Failed to flush Jetstream cursor on shutdown: {}", e);
            }
            return Ok(());
        }

        match user::get_all_dids(pool).await {
            Ok(fresh) => {
                dids = fresh;
                dids_loaded = true;
            }
            Err(e) => {
                error!("Failed to load DIDs for Jetstream: {}", e);
                if !dids_loaded {
                    // Keep the startup snapshot on first failure; otherwise reuse last known.
                }
            }
        }

        match system_state::get_jetstream_cursor(pool).await {
            Ok(fresh) => cursor = fresh,
            Err(e) => {
                error!("Failed to load Jetstream cursor: {}", e);
            }
        }

        if dids.is_empty() {
            warn!("No DIDs to subscribe. Retrying...");
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    if let Err(e) = cursor_saver.flush().await {
                        error!("Failed to flush Jetstream cursor on shutdown: {}", e);
                    }
                    return Ok(());
                }
                _ = sleep(NO_DIDS_RETRY) => {}
            }
            continue;
        }

        let bluesky_client = match BlueskyClient::new(config).await {
            Ok(c) => c,
            Err(e) => {
                backoff_attempt += 1;
                let backoff = calculate_backoff(backoff_attempt);
                error!(
                    "Failed to create Bluesky client for Jetstream: {}. Retrying in {:?}",
                    e, backoff
                );
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        if let Err(e) = cursor_saver.flush().await {
                            error!("Failed to flush Jetstream cursor on shutdown: {}", e);
                        }
                        return Ok(());
                    }
                    _ = sleep(backoff) => {}
                }
                continue;
            }
        };

        for endpoint in endpoints {
            if cancel_token.is_cancelled() {
                if let Err(e) = cursor_saver.flush().await {
                    error!("Failed to flush Jetstream cursor on shutdown: {}", e);
                }
                return Ok(());
            }

            info!("Connecting to Jetstream endpoint: {}", endpoint);

            let task_config = ConsumerTaskConfig {
                user_agent: format!("qonstellation/{}", env!("CARGO_PKG_VERSION")),
                compression: false,
                zstd_dictionary_location: String::new(),
                jetstream_hostname: endpoint.to_string(),
                collections: vec!["app.bsky.feed.post".to_string()],
                dids: dids.clone(),
                max_message_size_bytes: None,
                cursor,
                require_hello: false,
            };

            let consumer = Consumer::new(task_config);
            let handler = Arc::new(JetstreamEventHandler {
                pool: pool.clone(),
                bluesky_client: bluesky_client.clone(),
                notify: notify.clone(),
                cursor_saver: cursor_saver.clone(),
            });

            if let Err(e) = consumer.register_handler(handler).await {
                error!("Failed to register handler: {}", e);
                continue;
            }

            let token = cancel_token.clone();
            match consumer.run_background(token).await {
                Ok(_) => {
                    info!("Jetstream consumer exited gracefully");
                    if let Err(e) = cursor_saver.flush().await {
                        error!("Failed to flush Jetstream cursor on shutdown: {}", e);
                    }
                    return Ok(());
                }
                Err(e) => {
                    error!("Jetstream endpoint {} failed: {}", endpoint, e);
                }
            }
        }

        backoff_attempt += 1;
        let backoff = calculate_backoff(backoff_attempt);
        error!("All Jetstream endpoints failed. Retrying in {:?}", backoff);
        tokio::select! {
            _ = cancel_token.cancelled() => {
                if let Err(e) = cursor_saver.flush().await {
                    error!("Failed to flush Jetstream cursor on shutdown: {}", e);
                }
                return Ok(());
            }
            _ = sleep(backoff) => {}
        }
    }
}

fn calculate_backoff(attempt: u32) -> Duration {
    let secs = 2u64
        .saturating_pow(attempt.min(8))
        .min(MAX_BACKOFF.as_secs());
    Duration::from_secs(secs.max(1))
}

struct CursorSaver {
    pool: DbPool,
    last_write: Mutex<Option<Instant>>,
    last_cursor: Mutex<Option<i64>>,
}

impl CursorSaver {
    fn new(pool: DbPool) -> Self {
        Self {
            pool,
            last_write: Mutex::new(None),
            last_cursor: Mutex::new(None),
        }
    }

    async fn save(&self, cursor: i64) -> anyhow::Result<()> {
        *self.last_cursor.lock().await = Some(cursor);

        let should_write = match *self.last_write.lock().await {
            None => true,
            Some(t) => t.elapsed() >= CURSOR_SAVE_DEBOUNCE,
        };

        if should_write {
            system_state::save_jetstream_cursor(&self.pool, cursor).await?;
            *self.last_write.lock().await = Some(Instant::now());
        }

        Ok(())
    }

    async fn flush(&self) -> anyhow::Result<()> {
        let cursor = *self.last_cursor.lock().await;
        if let Some(cursor) = cursor {
            system_state::save_jetstream_cursor(&self.pool, cursor).await?;
            *self.last_write.lock().await = Some(Instant::now());
        }

        Ok(())
    }
}

struct JetstreamEventHandler {
    pool: DbPool,
    bluesky_client: BlueskyClient,
    notify: Arc<Notify>,
    cursor_saver: Arc<CursorSaver>,
}

#[async_trait::async_trait]
impl EventHandler for JetstreamEventHandler {
    async fn handle_event(&self, event: Arc<JetstreamEvent>) -> anyhow::Result<()> {
        match &*event {
            JetstreamEvent::Commit {
                did,
                time_us,
                commit,
                ..
            } => {
                if commit.operation != "create" || commit.collection != "app.bsky.feed.post" {
                    self.cursor_saver.save(*time_us as i64).await?;
                    return Ok(());
                }

                let post_event = match PostCreateEvent::from_commit(did, *time_us, commit) {
                    Ok(post_event) => post_event,
                    Err(e) => {
                        warn!("Invalid record: {}", e);
                        self.cursor_saver.save(*time_us as i64).await?;
                        return Ok(());
                    }
                };
                let at_proto_uri = build_at_proto_uri(did, &commit.rkey);

                match self
                    .bluesky_client
                    .is_self_thread(&post_event.record.reply, did)
                    .await
                {
                    Ok(true) => {}
                    Ok(false) => {
                        warn!(
                            "Skipping post {} because it is not a self thread",
                            at_proto_uri
                        );
                        self.cursor_saver.save(*time_us as i64).await?;
                        return Ok(());
                    }
                    Err(e) => {
                        // Transient Bluesky failure: don't advance the cursor so
                        // this event is replayed, and return Err without killing
                        // the connection (the outer loop reconnects on fatal errors).
                        error!("Error checking self thread: {}", e);
                        return Err(e);
                    }
                }

                let event_type = QueuedEventType::Post(post_event);
                let json = serde_json::to_value(&event_type).map_err(|e| {
                    error!("Error handling Jetstream event: {}", e);
                    anyhow::anyhow!("Failed to serialize event: {}", e)
                })?;
                queued_event::add_queued_event(&self.pool, &json)
                    .await
                    .map_err(|e| {
                        error!("Error handling Jetstream event: {}", e);
                        e
                    })?;
                self.notify.notify_one();
                self.cursor_saver.save(*time_us as i64).await?;
                Ok(())
            }
            JetstreamEvent::Delete { time_us, .. }
            | JetstreamEvent::Identity { time_us, .. }
            | JetstreamEvent::Account { time_us, .. } => {
                self.cursor_saver.save(*time_us as i64).await?;
                Ok(())
            }
        }
    }

    fn handler_id(&self) -> &str {
        "qonstellation_jetstream_handler"
    }
}
