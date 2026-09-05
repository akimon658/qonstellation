use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::app_config::config::Config;
use crate::database::DbPool;
use crate::model::bluesky_types::build_at_proto_uri;
use crate::model::post_event::{MediaEmbed, PostEmbed, QueuedEventType};
use crate::repository;
use crate::service::{bluesky_client, file_uploader, message_builder};

const MAX_RETRY: usize = 25;
const MAX_TIMEOUT_MS: u64 = 60 * 60 * 1000;

#[derive(Clone)]
pub struct Worker {
    pool: DbPool,
    config: Config,
    http_client: reqwest::Client,
    notify: Arc<Notify>,
    running: Arc<Mutex<bool>>,
}

impl Worker {
    pub fn new(pool: DbPool, config: Config, http_client: reqwest::Client) -> Self {
        Self {
            pool,
            config,
            http_client,
            notify: Arc::new(Notify::new()),
            running: Arc::new(Mutex::new(true)),
        }
    }

    pub async fn shutdown(&self) {
        *self.running.lock().await = false;
        self.notify.notify_one();
    }

    pub fn notify_handle(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    pub async fn run(&self) {
        let mut attempt = 0;
        let mut bluesky_client: Option<bluesky_client::BlueskyClient> = None;

        loop {
            let is_running: bool = *self.running.lock().await;
            if !is_running {
                break;
            }

            // Lazily (re)create the shared Bluesky client so we log in once
            // instead of on every queued event.
            if bluesky_client.is_none() {
                match bluesky_client::BlueskyClient::new(&self.config).await {
                    Ok(client) => {
                        bluesky_client = Some(client);
                    }
                    Err(e) => {
                        attempt += 1;
                        let timeout = calculate_backoff(attempt);
                        error!(
                            "Bluesky client creation failed (attempt {}): {}. Retrying in {:?}",
                            attempt, e, timeout
                        );
                        sleep(timeout).await;
                        if attempt >= MAX_RETRY {
                            error!(
                                "Max retry ({}) exceeded for client creation, resetting counter",
                                MAX_RETRY
                            );
                            attempt = 0;
                        }
                        continue;
                    }
                }
            }
            let client = match bluesky_client.clone() {
                Some(c) => c,
                None => continue,
            };

            match self.process_next_event(&client).await {
                Ok(true) => {
                    attempt = 0;
                }
                Ok(false) => {
                    attempt = 0;
                    tokio::select! {
                        _ = self.notify.notified() => {}
                        _ = sleep(Duration::from_secs(5)) => {}
                    }
                }
                Err(e) => {
                    attempt += 1;
                    // Drop the cached client so the next attempt re-logins.
                    // This avoids reusing a client with an expired/broken session.
                    bluesky_client = None;
                    let timeout = calculate_backoff(attempt);
                    error!(
                        "Event processing failed (attempt {}): {}. Retrying in {:?}",
                        attempt, e, timeout
                    );
                    sleep(timeout).await;

                    if attempt >= MAX_RETRY {
                        // DLQ: skip the poison event so one persistent failure
                        // doesn't block the queue for all users.
                        error!("Max retry ({}) exceeded, skipping oldest event", MAX_RETRY);
                        match repository::queued_event::get_oldest_queued_event(&self.pool).await {
                            Ok(Some(ev)) => {
                                if let Err(del_err) =
                                    repository::queued_event::delete_queued_event(&self.pool, ev.id)
                                        .await
                                {
                                    error!("Failed to delete poison event {}: {}", ev.id, del_err);
                                } else {
                                    warn!("Skipped poison event id={}", ev.id);
                                }
                            }
                            Ok(None) => {}
                            Err(db_err) => {
                                error!("Failed to fetch oldest event for skip: {}", db_err);
                            }
                        }
                        attempt = 0;
                    }
                }
            }
        }

        info!("Event queue worker shutting down...");
    }

    async fn process_next_event(
        &self,
        bluesky_client: &bluesky_client::BlueskyClient,
    ) -> anyhow::Result<bool> {
        let event = match repository::queued_event::get_oldest_queued_event(&self.pool).await? {
            Some(e) => e,
            None => return Ok(false),
        };

        info!("Processing queued event id={}", event.id);

        let event_type: QueuedEventType = serde_json::from_value(event.event_json.clone())
            .map_err(|e| anyhow::anyhow!("Failed to parse queued event: {}", e))?;

        let QueuedEventType::Post(post_event) = event_type;

        let did = &post_event.did;
        let rkey = &post_event.rkey;
        let record = &post_event.record;

        let (user_id, target_channel_id) =
            match repository::user::get_user_setting_by_did(&self.pool, did).await {
                Ok(setting) => setting,
                Err(e)
                    if matches!(
                        e.downcast_ref::<sqlx::Error>(),
                        Some(sqlx::Error::RowNotFound)
                    ) =>
                {
                    warn!(
                        "No user setting found for DID {}, dropping event {}",
                        did, event.id
                    );
                    repository::queued_event::delete_queued_event(&self.pool, event.id).await?;
                    return Ok(true);
                }
                Err(e) => {
                    // Transient DB error: retry without deleting.
                    return Err(anyhow::anyhow!(
                        "Failed to load user setting for {}: {}",
                        did,
                        e
                    ));
                }
            };

        let at_proto_uri = build_at_proto_uri(did, rkey);

        if let Ok(Some(_)) =
            repository::post::get_traq_message_id_by_at_proto_uri(&self.pool, &at_proto_uri).await
        {
            warn!("Post already exists: {}", at_proto_uri);
            repository::queued_event::delete_queued_event(&self.pool, event.id).await?;
            return Ok(true);
        }

        let access_token = repository::user::get_user_access_token(&self.pool, user_id).await?;

        let mut file_ids = Vec::new();
        if let Some(ref embed) = record.embed {
            match embed {
                PostEmbed::Images { images } => {
                    for image in images {
                        let file_id = self
                            .fetch_and_upload_image(
                                bluesky_client,
                                did,
                                &image.image.cid,
                                &target_channel_id.to_string(),
                                &access_token,
                            )
                            .await?;
                        file_ids.push(file_id);
                    }
                }
                PostEmbed::Video { video } => {
                    let file_id = self
                        .fetch_and_upload_video(
                            bluesky_client,
                            did,
                            &video.video.cid,
                            &target_channel_id.to_string(),
                            &access_token,
                        )
                        .await?;
                    file_ids.push(file_id);
                }
                PostEmbed::RecordWithMedia { media, .. } => match media {
                    MediaEmbed::Images { images } => {
                        for image in images {
                            let file_id = self
                                .fetch_and_upload_image(
                                    bluesky_client,
                                    did,
                                    &image.image.cid,
                                    &target_channel_id.to_string(),
                                    &access_token,
                                )
                                .await?;
                            file_ids.push(file_id);
                        }
                    }
                    MediaEmbed::Video { video } => {
                        let file_id = self
                            .fetch_and_upload_video(
                                bluesky_client,
                                did,
                                &video.video.cid,
                                &target_channel_id.to_string(),
                                &access_token,
                            )
                            .await?;
                        file_ids.push(file_id);
                    }
                },
                PostEmbed::Record { .. } => {
                    // No media to upload for record-only embeds
                }
            }
        }

        let builder = message_builder::MessageBuilder::new(
            target_channel_id.to_string(),
            access_token.clone(),
        );
        let message = builder
            .build(
                &self.pool,
                &self.config,
                &self.http_client,
                record,
                Some(&file_ids),
            )
            .await?;

        let traq_message_id = post_to_traq(
            &self.http_client,
            &self.config,
            &target_channel_id.to_string(),
            &message,
            &access_token,
        )
        .await?;

        repository::post::save_post_metadata(&self.pool, &at_proto_uri, traq_message_id).await?;

        info!("Posted to traQ: message_id={}", traq_message_id);

        repository::queued_event::delete_queued_event(&self.pool, event.id).await?;
        Ok(true)
    }

    async fn fetch_and_upload_image(
        &self,
        bluesky_client: &bluesky_client::BlueskyClient,
        did: &str,
        cid: &str,
        channel_id: &str,
        access_token: &str,
    ) -> anyhow::Result<String> {
        let data = bluesky_client
            .get_blob(did, cid)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download image blob {cid}: {e}"))?;
        let (resized, filename, content_type) = file_uploader::resize_image(&data)
            .map_err(|e| anyhow::anyhow!("Failed to resize image {cid}: {e}"))?;
        file_uploader::upload_file(
            &self.http_client,
            &self.config,
            channel_id,
            &resized,
            filename,
            content_type,
            access_token,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload image {cid}: {e}"))
    }

    async fn fetch_and_upload_video(
        &self,
        bluesky_client: &bluesky_client::BlueskyClient,
        did: &str,
        cid: &str,
        channel_id: &str,
        access_token: &str,
    ) -> anyhow::Result<String> {
        let data = bluesky_client
            .get_blob(did, cid)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download video blob {cid}: {e}"))?;
        file_uploader::upload_file(
            &self.http_client,
            &self.config,
            channel_id,
            &data,
            "video.mp4",
            "video/mp4",
            access_token,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to upload video {cid}: {e}"))
    }
}

async fn post_to_traq(
    http_client: &reqwest::Client,
    config: &Config,
    channel_id: &str,
    content: &str,
    access_token: &str,
) -> anyhow::Result<Uuid> {
    let url = format!(
        "{}/api/v3/channels/{}/messages",
        config.traq_base_url, channel_id
    );

    let body = serde_json::json!({
        "content": content,
    });

    let response = http_client
        .post(&url)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        return Err(anyhow::anyhow!("Failed to post message: {}", text));
    }

    let json: serde_json::Value = response.json().await?;
    let message_id = json["id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing message ID in response"))?;

    Uuid::parse_str(message_id)
        .map_err(|e| anyhow::anyhow!("Invalid message ID format '{}': {}", message_id, e))
}

fn calculate_backoff(attempt: usize) -> Duration {
    let timeout_ms = (2u64.pow(attempt.min(31) as u32).saturating_mul(100)).min(MAX_TIMEOUT_MS);
    Duration::from_millis(timeout_ms)
}
