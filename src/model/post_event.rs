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
        }
    }
}

impl PostCreateEvent {
    pub fn from_commit(
        did: &str,
        time_us: u64,
        commit: &atproto_jetstream::JetstreamEventCommit,
    ) -> anyhow::Result<Self> {
        let atrium_record: AtriumPostRecord = serde_json::from_value(commit.record.clone())
            .map_err(|e| anyhow::anyhow!("Invalid post record: {}", e))?;
        let record = PostRecord::from(&atrium_record);

        Ok(Self {
            did: did.to_string(),
            time_us,
            rkey: commit.rkey.clone(),
            record,
        })
    }
}
