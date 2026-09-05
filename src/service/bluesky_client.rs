use crate::app_config::config::Config;
use crate::model::post_event::ReplyRef;
use atrium_api::agent::Configure;
use atrium_api::app::bsky::feed::defs::{ThreadViewPost, ThreadViewPostParentRefs};
use atrium_api::app::bsky::feed::get_post_thread::{
    OutputThreadRefs, ParametersData as GetPostThreadParams,
};
use atrium_api::com::atproto::sync::get_blob::ParametersData as GetBlobParams;
use atrium_api::types::Union;
use bsky_sdk::BskyAgent;

const MAX_PARENT_HEIGHT: u16 = 1000;

#[derive(Clone)]
pub struct BlueskyClient {
    agent: BskyAgent,
}

impl BlueskyClient {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let agent = BskyAgent::builder()
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to build agent: {}", e))?;

        agent.configure_endpoint(config.bluesky_hosting_provider.clone());

        agent
            .login(
                &config.bluesky_account_identifier,
                &config.bluesky_app_password,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to login: {}", e))?;

        Ok(Self { agent })
    }

    pub async fn get_blob(&self, did: &str, cid: &str) -> anyhow::Result<Vec<u8>> {
        let cid = cid.parse().map_err(|_| anyhow::anyhow!("Invalid CID"))?;
        let did = did.parse().map_err(|_| anyhow::anyhow!("Invalid DID"))?;

        let params = GetBlobParams { cid, did };

        let response = self
            .agent
            .api
            .com
            .atproto
            .sync
            .get_blob(params.into())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get blob: {}", e))?;

        Ok(response)
    }

    /// Returns true if the post is a root post or all ancestors up to the root
    /// are authored by `author_did`.
    pub async fn is_self_thread(
        &self,
        reply: &Option<ReplyRef>,
        author_did: &str,
    ) -> anyhow::Result<bool> {
        let parent_uri = match reply {
            None => return Ok(true),
            Some(reply) => &reply.parent.uri,
        };

        let params = GetPostThreadParams {
            uri: parent_uri.clone(),
            depth: Some(
                0u16.try_into()
                    .map_err(|e| anyhow::anyhow!("Invalid depth: {}", e))?,
            ),
            parent_height: Some(
                MAX_PARENT_HEIGHT
                    .try_into()
                    .map_err(|e| anyhow::anyhow!("Invalid parent_height: {}", e))?,
            ),
        };

        let output = self
            .agent
            .api
            .app
            .bsky
            .feed
            .get_post_thread(params.into())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get post thread: {}", e))?;

        let mut current: Option<ThreadViewPost> = match &output.thread {
            Union::Refs(OutputThreadRefs::AppBskyFeedDefsThreadViewPost(thread_view)) => {
                Some((**thread_view).clone())
            }
            // NotFoundPost or BlockedPost — can't confirm all parents are self
            _ => None,
        };

        while let Some(thread_view) = current {
            if thread_view.post.author.did.as_str() != author_did {
                return Ok(false);
            }

            current = match &thread_view.parent {
                None => return Ok(true),
                Some(Union::Refs(ThreadViewPostParentRefs::ThreadViewPost(parent))) => {
                    Some((**parent).clone())
                }
                // NotFoundPost or BlockedPost — can't confirm all parents are self
                _ => return Ok(false),
            };
        }

        // Initial thread was NotFoundPost or BlockedPost.
        Ok(false)
    }
}
