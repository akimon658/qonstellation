use atproto_record::aturi::ATURI;
use atrium_api::app::bsky::embed::images::Image as AtriumImage;
use atrium_api::app::bsky::embed::record::Main as AtriumRecordMain;
use atrium_api::app::bsky::embed::record_with_media::MainMediaRefs;
use atrium_api::app::bsky::embed::video::Main as AtriumVideoMain;
use atrium_api::app::bsky::feed::post::{
    Record as AtriumPostRecord, RecordEmbedRefs, ReplyRef as AtriumReplyRef,
};
use atrium_api::app::bsky::richtext::facet::{Main as AtriumFacetMain, MainFeaturesItem};
use atrium_api::com::atproto::repo::strong_ref::Main as AtriumStrongRefMain;
use atrium_api::types::{BlobRef as AtriumBlobRef, TypedBlobRef, Union};
use std::str::FromStr;

/// Collection of Skyblur sidecar records.
pub const SKYBLUR_COLLECTION: &str = "uk.skyblur.post";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum QueuedEventType {
    #[serde(rename = "app.bsky.feed.post")]
    Post(PostCreateEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PostCreateEvent {
    pub did: String,
    pub time_us: u64,
    pub rkey: String,
    pub record: PostRecord,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Facet {
    pub index: ByteSlice,
    pub features: Vec<FacetFeature>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByteSlice {
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "$type", rename_all = "lowercase")]
pub enum FacetFeature {
    #[serde(rename = "app.bsky.richtext.facet#mention")]
    Mention { did: String },
    #[serde(rename = "app.bsky.richtext.facet#link")]
    Link { uri: String },
    #[serde(rename = "app.bsky.richtext.facet#tag")]
    Tag { tag: String },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PostRecord {
    pub text: String,
    pub facets: Option<Vec<Facet>>,
    pub reply: Option<ReplyRef>,
    pub embed: Option<PostEmbed>,
    pub skyblur: Option<SkyblurMeta>,
}

/// Skyblur metadata attached to an `app.bsky.feed.post` record.
///
/// Built by [`PostCreateEvent::from_commit`] from the flat
/// `uk.skyblur.post.uri` / `uk.skyblur.post.visibility` keys of the commit
/// record. The URI is parsed and validated up front, so holders can use the
/// decomposed components without re-parsing.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SkyblurMeta {
    /// DID authority of the `uk.skyblur.post` record.
    pub repo: String,
    /// Record key of the `uk.skyblur.post` record.
    pub rkey: String,
    pub visibility: SkyblurVisibility,
}

impl SkyblurMeta {
    /// Parses a `uk.skyblur.post` AT-URI, rejecting other collections.
    pub(crate) fn parse(uri: &str, visibility: SkyblurVisibility) -> anyhow::Result<Self> {
        let parsed = ATURI::from_str(uri)
            .map_err(|e| anyhow::anyhow!("Invalid Skyblur record URI {uri}: {e}"))?;

        if parsed.collection != SKYBLUR_COLLECTION {
            return Err(anyhow::anyhow!("Not a Skyblur record URI: {uri}"));
        }

        Ok(Self {
            repo: parsed.authority,
            rkey: parsed.record_key,
            visibility,
        })
    }

    /// Reconstructs the full AT-URI of the `uk.skyblur.post` record.
    pub fn uri(&self) -> String {
        format!("at://{}/{SKYBLUR_COLLECTION}/{}", self.repo, self.rkey)
    }
}

/// Visibility values defined by `uk.skyblur.post/record.json`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkyblurVisibility {
    Public,
    Password,
    Login,
    Followers,
    Following,
    Mutual,
    List,
    #[serde(other)]
    Unknown,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ReplyRef {
    pub parent: StrongRef,
    pub root: StrongRef,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StrongRef {
    pub uri: String,
    pub cid: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "$type")]
pub enum PostEmbed {
    #[serde(rename = "app.bsky.embed.images")]
    Images { images: Vec<ImageEmbed> },
    #[serde(rename = "app.bsky.embed.video")]
    Video { video: VideoEmbed },
    #[serde(rename = "app.bsky.embed.record")]
    Record { record: RecordRef },
    #[serde(rename = "app.bsky.embed.recordWithMedia")]
    RecordWithMedia {
        media: MediaEmbed,
        record: RecordRef,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "$type")]
pub enum MediaEmbed {
    #[serde(rename = "app.bsky.embed.images")]
    Images { images: Vec<ImageEmbed> },
    #[serde(rename = "app.bsky.embed.video")]
    Video { video: VideoEmbed },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ImageEmbed {
    pub alt: String,
    pub image: BlobRef,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct VideoEmbed {
    pub alt: Option<String>,
    pub video: BlobRef,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RecordRef {
    pub uri: String,
}

/// Simplified blob reference (only stores CID string).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BlobRef {
    pub cid: String,
}

impl From<&AtriumBlobRef> for BlobRef {
    fn from(value: &AtriumBlobRef) -> Self {
        Self {
            cid: match value {
                AtriumBlobRef::Typed(typed) => match typed {
                    TypedBlobRef::Blob(blob) => blob.r#ref.0.to_string(),
                },
                AtriumBlobRef::Untyped(untyped) => untyped.cid.clone(),
            },
        }
    }
}

impl From<&AtriumImage> for ImageEmbed {
    fn from(value: &AtriumImage) -> Self {
        Self {
            alt: value.alt.clone(),
            image: BlobRef::from(&value.image),
        }
    }
}

impl From<&AtriumVideoMain> for VideoEmbed {
    fn from(value: &AtriumVideoMain) -> Self {
        Self {
            alt: value.alt.clone(),
            video: BlobRef::from(&value.video),
        }
    }
}

impl From<&AtriumStrongRefMain> for RecordRef {
    fn from(value: &AtriumStrongRefMain) -> Self {
        Self {
            uri: value.uri.clone(),
        }
    }
}

impl From<&AtriumRecordMain> for RecordRef {
    fn from(value: &AtriumRecordMain) -> Self {
        RecordRef::from(&value.record)
    }
}

impl From<&AtriumFacetMain> for Facet {
    fn from(value: &AtriumFacetMain) -> Self {
        Self {
            index: ByteSlice {
                byte_start: value.index.byte_start,
                byte_end: value.index.byte_end,
            },
            features: value.features.iter().map(FacetFeature::from).collect(),
        }
    }
}

impl From<&Union<MainFeaturesItem>> for FacetFeature {
    fn from(value: &Union<MainFeaturesItem>) -> Self {
        match value {
            Union::Refs(MainFeaturesItem::Mention(m)) => FacetFeature::Mention {
                did: m.did.to_string(),
            },
            Union::Refs(MainFeaturesItem::Link(l)) => FacetFeature::Link { uri: l.uri.clone() },
            Union::Refs(MainFeaturesItem::Tag(t)) => FacetFeature::Tag { tag: t.tag.clone() },
            _ => FacetFeature::Tag { tag: String::new() },
        }
    }
}

impl From<&Union<MainMediaRefs>> for MediaEmbed {
    fn from(value: &Union<MainMediaRefs>) -> Self {
        match value {
            Union::Refs(MainMediaRefs::AppBskyEmbedImagesMain(images)) => MediaEmbed::Images {
                images: images.images.iter().map(ImageEmbed::from).collect(),
            },
            Union::Refs(MainMediaRefs::AppBskyEmbedVideoMain(video)) => MediaEmbed::Video {
                video: VideoEmbed::from(video.as_ref()),
            },
            _ => {
                // AppBskyEmbedExternalMain is not supported in the app
                MediaEmbed::Images { images: vec![] }
            }
        }
    }
}

impl From<&Union<RecordEmbedRefs>> for PostEmbed {
    fn from(value: &Union<RecordEmbedRefs>) -> Self {
        match value {
            Union::Refs(RecordEmbedRefs::AppBskyEmbedImagesMain(images)) => PostEmbed::Images {
                images: images.images.iter().map(ImageEmbed::from).collect(),
            },
            Union::Refs(RecordEmbedRefs::AppBskyEmbedVideoMain(video)) => PostEmbed::Video {
                video: VideoEmbed::from(video.as_ref()),
            },
            Union::Refs(RecordEmbedRefs::AppBskyEmbedRecordMain(record)) => PostEmbed::Record {
                record: RecordRef::from(record.as_ref()),
            },
            Union::Refs(RecordEmbedRefs::AppBskyEmbedRecordWithMediaMain(rwm)) => {
                PostEmbed::RecordWithMedia {
                    media: MediaEmbed::from(&rwm.media),
                    record: RecordRef::from(&rwm.record),
                }
            }
            Union::Refs(RecordEmbedRefs::AppBskyEmbedExternalMain(_)) => {
                // External embeds are not supported in the app
                PostEmbed::Images { images: vec![] }
            }
            _ => PostEmbed::Images { images: vec![] },
        }
    }
}

impl From<&AtriumStrongRefMain> for StrongRef {
    fn from(value: &AtriumStrongRefMain) -> Self {
        Self {
            uri: value.uri.clone(),
            cid: value.cid.as_ref().to_string(),
        }
    }
}

impl From<&AtriumReplyRef> for ReplyRef {
    fn from(value: &AtriumReplyRef) -> Self {
        Self {
            parent: StrongRef::from(&value.parent),
            root: StrongRef::from(&value.root),
        }
    }
}

impl From<&AtriumPostRecord> for PostRecord {
    fn from(value: &AtriumPostRecord) -> Self {
        Self {
            text: value.text.clone(),
            facets: value
                .facets
                .as_ref()
                .map(|facets| facets.iter().map(Facet::from).collect()),
            reply: value.reply.as_ref().map(ReplyRef::from),
            embed: value.embed.as_ref().map(PostEmbed::from),
            // Skyblur's custom fields (`uk.skyblur.post.*`) are not part of
            // the Atrium type; they are filled in by `PostCreateEvent::from_commit`.
            skyblur: None,
        }
    }
}

/// The flat Skyblur key pair as it appears in an `app.bsky.feed.post` commit
/// record. Only used to interpret the wire format in
/// [`PostCreateEvent::from_commit`].
#[derive(serde::Deserialize)]
struct SkyblurWire {
    #[serde(rename = "uk.skyblur.post.uri", default)]
    uri: Option<String>,
    #[serde(rename = "uk.skyblur.post.visibility", default)]
    visibility: Option<SkyblurVisibility>,
}

impl PostCreateEvent {
    pub fn from_commit(
        did: &str,
        time_us: u64,
        commit: &atproto_jetstream::JetstreamEventCommit,
    ) -> anyhow::Result<Self> {
        let atrium_record: AtriumPostRecord = serde_json::from_value(commit.record.clone())
            .map_err(|e| anyhow::anyhow!("Invalid post record: {}", e))?;
        let mut record = PostRecord::from(&atrium_record);
        let wire: SkyblurWire = serde_json::from_value(commit.record.clone())
            .map_err(|e| anyhow::anyhow!("Invalid post record: {}", e))?;
        record.skyblur = match (wire.uri, wire.visibility) {
            (Some(uri), Some(visibility)) => Some(
                SkyblurMeta::parse(&uri, visibility)
                    .map_err(|e| anyhow::anyhow!("Invalid post record: {}", e))?,
            ),
            (None, None) => None,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid post record: uk.skyblur.post.uri and \
                     uk.skyblur.post.visibility must be set together"
                ));
            }
        };

        Ok(Self {
            did: did.to_string(),
            time_us,
            rkey: commit.rkey.clone(),
            record,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atproto_jetstream::JetstreamEventCommit;

    fn commit_with_record(record: serde_json::Value) -> JetstreamEventCommit {
        JetstreamEventCommit {
            rev: "rev".to_string(),
            operation: "create".to_string(),
            collection: "app.bsky.feed.post".to_string(),
            rkey: "3much6qlnokok".to_string(),
            cid: "cid".to_string(),
            record,
        }
    }

    fn skyblur_record_json() -> serde_json::Value {
        serde_json::json!({
            "text": "ワルプルギスの廻天、○○○○好きなんだけど",
            "facets": [],
            "langs": ["ja"],
            "via": "Skyblur",
            "uk.skyblur.post.uri": "at://did:plc:iba6craltg5onugrgwcwfizi/uk.skyblur.post/3much6qlnokok",
            "uk.skyblur.post.visibility": "public",
            "createdAt": "2026-08-30T13:01:41.190Z",
        })
    }

    #[test]
    fn from_commit_extracts_skyblur_meta() -> anyhow::Result<()> {
        let event = PostCreateEvent::from_commit(
            "did:plc:iba6craltg5onugrgwcwfizi",
            1,
            &commit_with_record(skyblur_record_json()),
        )?;

        assert_eq!(
            event.record.skyblur,
            Some(SkyblurMeta {
                repo: "did:plc:iba6craltg5onugrgwcwfizi".to_string(),
                rkey: "3much6qlnokok".to_string(),
                visibility: SkyblurVisibility::Public,
            })
        );
        Ok(())
    }

    #[test]
    fn from_commit_without_skyblur_yields_none() -> anyhow::Result<()> {
        let event = PostCreateEvent::from_commit(
            "did:plc:abc",
            1,
            &commit_with_record(serde_json::json!({
                "text": "plain post",
                "createdAt": "2026-08-30T13:01:41.190Z",
            })),
        )?;

        assert!(event.record.skyblur.is_none());
        Ok(())
    }

    #[test]
    fn from_commit_rejects_half_present_skyblur() -> anyhow::Result<()> {
        let result = PostCreateEvent::from_commit(
            "did:plc:abc",
            1,
            &commit_with_record(serde_json::json!({
                "text": "broken post",
                "createdAt": "2026-08-30T13:01:41.190Z",
                "uk.skyblur.post.uri": "at://did:plc:abc/uk.skyblur.post/xyz",
            })),
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn from_commit_rejects_invalid_skyblur_uri() -> anyhow::Result<()> {
        let result = PostCreateEvent::from_commit(
            "did:plc:abc",
            1,
            &commit_with_record(serde_json::json!({
                "text": "broken post",
                "createdAt": "2026-08-30T13:01:41.190Z",
                "uk.skyblur.post.uri": "not-an-at-uri",
                "uk.skyblur.post.visibility": "public",
            })),
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn from_commit_rejects_non_skyblur_collection_uri() -> anyhow::Result<()> {
        let result = PostCreateEvent::from_commit(
            "did:plc:abc",
            1,
            &commit_with_record(serde_json::json!({
                "text": "broken post",
                "createdAt": "2026-08-30T13:01:41.190Z",
                "uk.skyblur.post.uri": "at://did:plc:abc/app.bsky.feed.post/xyz",
                "uk.skyblur.post.visibility": "public",
            })),
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn skyblur_meta_roundtrips_through_queue_json() -> anyhow::Result<()> {
        let event = PostCreateEvent::from_commit(
            "did:plc:iba6craltg5onugrgwcwfizi",
            1,
            &commit_with_record(skyblur_record_json()),
        )?;

        let json = serde_json::to_value(QueuedEventType::Post(event))?;
        let back: QueuedEventType = serde_json::from_value(json)?;

        let QueuedEventType::Post(post) = back;
        assert!(
            post.record
                .skyblur
                .is_some_and(|meta| matches!(meta.visibility, SkyblurVisibility::Public))
        );
        Ok(())
    }
}
