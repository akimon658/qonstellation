use std::cmp::Reverse;
use std::str::FromStr;

use atproto_record::aturi::ATURI;

use crate::app_config::config::Config;
use crate::database::DbPool;
use crate::model::post_event::{Facet, FacetFeature, PostEmbed, PostRecord, SkyblurVisibility};
use crate::repository;
use crate::service::skyblur;

pub struct MessageBuilder {
    target_channel_id: String,
    traq_access_token: String,
}

impl MessageBuilder {
    pub fn new(target_channel_id: String, traq_access_token: String) -> Self {
        Self {
            target_channel_id,
            traq_access_token,
        }
    }

    pub async fn build(
        &self,
        pool: &DbPool,
        config: &Config,
        http_client: &reqwest::Client,
        record: &PostRecord,
        file_ids: Option<&[String]>,
    ) -> anyhow::Result<String> {
        // Skyblur posts need special handling before the generic flow:
        // - `public`: restore the original text and convert `[hidden]` ranges
        //   to traQ spoilers (`!!hidden!!`). The restored text replaces the
        //   masked text, so facet offsets (which refer to the masked text)
        //   must not be applied to it.
        // - other visibilities (or a failed fetch): keep the masked text and
        //   link to the Skyblur page instead.
        let mut skyblur_page_url: Option<String> = None;
        let mut restored_text: Option<String> = None;

        if let Some(ref meta) = record.skyblur {
            if meta.visibility == SkyblurVisibility::Public {
                match skyblur::fetch_public_original_text(http_client, meta).await {
                    Ok(original) => {
                        restored_text = Some(skyblur::convert_brackets_to_traq_spoiler(&original));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to restore Skyblur post {}: {e}", meta.uri());
                        skyblur_page_url = Some(skyblur::skyblur_page_url(meta));
                    }
                }
            } else {
                skyblur_page_url = Some(skyblur::skyblur_page_url(meta));
            }
        }

        let mut text = match restored_text {
            Some(restored) => restored,
            None => extract_text(record)?,
        };

        if let Some(ids) = file_ids
            && !ids.is_empty()
        {
            let file_links = ids
                .iter()
                .map(|id| format!("{}/files/{id}", config.traq_base_url))
                .collect::<Vec<_>>()
                .join("\n");

            append_url(&mut text, &file_links);
        }

        if let Some(url) = skyblur_page_url {
            append_url(&mut text, &url);
        }

        if let Some(ref reply) = record.reply {
            let parent_uri = &reply.parent.uri;
            let traq_message_id =
                repository::post::get_traq_message_id_by_at_proto_uri(pool, parent_uri).await?;

            let url_to_append: Option<String> = if let Some(traq_id) = traq_message_id {
                let latest_message_id = get_latest_message_id(
                    http_client,
                    &config.traq_base_url,
                    &self.target_channel_id,
                    &self.traq_access_token,
                )
                .await?;

                if latest_message_id.as_deref() != Some(traq_id.to_string().as_str()) {
                    Some(get_traq_message_url(
                        &config.traq_base_url,
                        &traq_id.to_string(),
                    ))
                } else {
                    None
                }
            } else {
                Some(get_bluesky_post_url_from_str(parent_uri)?)
            };

            if let Some(url) = url_to_append {
                append_url(&mut text, &url);
            }
        }

        let embedded_record_uri_str: Option<&str> = match record.embed {
            Some(PostEmbed::Record { ref record }) => Some(&record.uri),
            Some(PostEmbed::RecordWithMedia { ref record, .. }) => Some(&record.uri),
            _ => None,
        };

        if let Some(uri_str) = embedded_record_uri_str {
            let embedded_record_uri = ATURI::from_str(uri_str)
                .map_err(|e| anyhow::anyhow!("Invalid embedded record URI: {e}"))?;

            if embedded_record_uri.collection == "app.bsky.feed.post" {
                let traq_message_id =
                    repository::post::get_traq_message_id_by_at_proto_uri(pool, uri_str).await?;

                let url_to_append = if let Some(traq_id) = traq_message_id {
                    get_traq_message_url(&config.traq_base_url, &traq_id.to_string())
                } else {
                    get_bluesky_post_url(&embedded_record_uri)
                };

                append_url(&mut text, &url_to_append);
            }
        }

        Ok(text)
    }
}

fn extract_text(record: &PostRecord) -> anyhow::Result<String> {
    let mut text_bytes = record.text.as_bytes().to_vec();

    if let Some(ref facets) = record.facets
        && !facets.is_empty()
    {
        // Sort facets in reverse order to avoid affecting the byte offsets of subsequent facets
        let mut sorted: Vec<&Facet> = facets.iter().collect();
        sorted.sort_by_key(|f| Reverse(f.index.byte_start));

        for facet in sorted {
            let link_uri = facet.features.iter().find_map(|f| match f {
                FacetFeature::Link { uri } => Some(uri),
                _ => None,
            });

            let Some(uri) = link_uri else {
                continue;
            };

            let start = facet.index.byte_start;
            let end = facet.index.byte_end;

            if start > end || end > text_bytes.len() {
                continue;
            }

            let uri_bytes = uri.as_bytes();
            let mut new_bytes =
                Vec::with_capacity(start + uri_bytes.len() + (text_bytes.len() - end));
            new_bytes.extend_from_slice(&text_bytes[..start]);
            new_bytes.extend_from_slice(uri_bytes);
            new_bytes.extend_from_slice(&text_bytes[end..]);

            text_bytes = new_bytes;
        }
    }

    Ok(String::from_utf8_lossy(&text_bytes).to_string())
}

fn append_url(text: &mut String, url: &str) {
    if text.is_empty() {
        text.push_str(url);
    } else {
        text.push('\n');
        text.push_str(url);
    }
}

fn get_traq_message_url(traq_base_url: &str, message_id: &str) -> String {
    format!("{traq_base_url}/messages/{message_id}")
}

fn get_bluesky_post_url(uri: &ATURI) -> String {
    format!(
        "https://bsky.app/profile/{}/post/{}",
        uri.authority, uri.record_key
    )
}

fn get_bluesky_post_url_from_str(resource_uri: &str) -> anyhow::Result<String> {
    let uri =
        ATURI::from_str(resource_uri).map_err(|e| anyhow::anyhow!("Invalid post URI: {e}"))?;

    Ok(get_bluesky_post_url(&uri))
}

async fn get_latest_message_id(
    http_client: &reqwest::Client,
    traq_base_url: &str,
    channel_id: &str,
    access_token: &str,
) -> anyhow::Result<Option<String>> {
    let url = format!("{traq_base_url}/api/v3/channels/{channel_id}/messages?limit=1");

    let response = http_client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch latest message: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Failed to fetch latest message: status {status}: {text}"
        ));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse latest message: {e}"))?;
    let id = json
        .as_array()
        .and_then(|a| a.first())
        .and_then(|m| m.get("id"))
        .and_then(|v| v.as_str());

    Ok(id.map(|s| s.to_string()))
}
