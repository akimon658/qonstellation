//! Skyblur (`uk.skyblur.post`) integration.
//!
//! Skyblur stores the original post text — with spoiler ranges enclosed in
//! `[...]` — in a separate `uk.skyblur.post` record
//! (see `lexicon/uk/skyblur/post/record.json` in
//! <https://github.com/usounds/Skyblur>), while the `app.bsky.feed.post`
//! record only contains the masked text (e.g. `○○○○`).
//!
//! For `public` posts the sidecar record is world-readable, so it can be
//! fetched with the standard `com.atproto.repo.getRecord` XRPC (whose
//! `value` is `unknown`, hence no atrium codegen is needed — a hand-written
//! serde struct matching the lexicon suffices). Other visibilities require
//! authentication or a password and are intentionally not fetched.

use crate::model::post_event::{SKYBLUR_COLLECTION, SkyblurMeta};
use atproto_identity::{plc, web};

const SKYBLUR_POST_PAGE_BASE: &str = "https://skyblur.uk/post";

/// Subset of the `uk.skyblur.post` record value we care about.
///
/// Full lexicon (`record.json`): `uri`, `text`, `createdAt`, `visibility`,
/// plus optional `additional`, `encryptBody`, `listUri`.
#[derive(serde::Deserialize, Debug)]
struct SkyblurPostRecordValue {
    text: String,
}

#[derive(serde::Deserialize, Debug)]
struct GetRecordOutput {
    value: SkyblurPostRecordValue,
}

/// Builds the human-readable Skyblur page URL for a post.
///
/// Used for non-`public` posts (and as a fallback) instead of restoring the
/// spoiler text.
pub fn skyblur_page_url(meta: &SkyblurMeta) -> String {
    format!("{SKYBLUR_POST_PAGE_BASE}/{}/{}", meta.repo, meta.rkey)
}

/// Converts Skyblur spoiler markup (`[hidden]`) to traQ spoiler (`!!hidden!!`).
///
/// traQ spoilers may span multiple lines, so bracketed ranges containing
/// newlines are wrapped as a whole. Unclosed brackets and nested brackets
/// are left untouched. Empty `[]` is removed entirely: converting it to
/// `!!!!` would just render as literal exclamation marks in traQ.
///
/// Literal `!!` in the original text is escaped as `\!!` beforehand so that
/// traQ does not misinterpret it as spoiler markup.
pub fn convert_brackets_to_traq_spoiler(text: &str) -> String {
    let text = text.replace("!!", "\\!!");
    let mut output = String::with_capacity(text.len());
    let mut rest = &text[..];

    while let Some(open) = rest.find('[') {
        let after_open = &rest[open + 1..];

        // Find the matching `]` with depth counting. A nested `[` means the
        // markup is invalid (Skyblur validation forbids nesting); leave such
        // segments untouched.
        let mut depth = 1_usize;
        let mut nested = false;
        let mut close_at: Option<usize> = None;
        for (i, ch) in after_open.char_indices() {
            if ch == '[' {
                depth += 1;
                nested = true;
            } else if ch == ']' {
                depth -= 1;
                if depth == 0 {
                    close_at = Some(i);
                    break;
                }
            }
        }

        match close_at {
            // `]` is ASCII so `end + 1` is a char boundary.
            Some(end) if !nested && !after_open[..end].is_empty() => {
                output.push_str(&rest[..open]);
                output.push_str("!!");
                output.push_str(&after_open[..end]);
                output.push_str("!!");
                rest = &after_open[end + 1..];
            }
            Some(end) if !nested && after_open[..end].is_empty() => {
                // Empty `[]`: remove it entirely (traQ renders `!!!!` as
                // literal exclamation marks, so there is nothing to wrap).
                output.push_str(&rest[..open]);
                rest = &after_open[end + 1..];
            }
            Some(end) => {
                // Nested brackets: emit the whole segment literally.
                let seg_end = open + 1 + end + 1;
                output.push_str(&rest[..seg_end]);
                rest = &rest[seg_end..];
            }
            // Unclosed `[`: leave the remainder untouched.
            None => break,
        }
    }

    output.push_str(rest);
    output
}

async fn resolve_pds_endpoint(http_client: &reqwest::Client, did: &str) -> anyhow::Result<String> {
    let doc = if did.starts_with("did:plc:") {
        plc::query(http_client, "plc.directory", did)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to resolve DID {did}: {e}"))?
    } else if did.starts_with("did:web:") {
        web::query(http_client, did)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to resolve DID {did}: {e}"))?
    } else {
        return Err(anyhow::anyhow!("Unsupported DID method: {did}"));
    };

    doc.pds_endpoints()
        .first()
        .copied()
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No PDS endpoint found for {did}"))
}

/// Fetches the original (unmasked, `[...]`-annotated) text of a `public`
/// Skyblur post via `com.atproto.repo.getRecord` on the author's PDS.
pub async fn fetch_public_original_text(
    http_client: &reqwest::Client,
    meta: &SkyblurMeta,
) -> anyhow::Result<String> {
    let pds = resolve_pds_endpoint(http_client, &meta.repo).await?;

    let output: GetRecordOutput = http_client
        .get(format!("{pds}/xrpc/com.atproto.repo.getRecord"))
        .query(&[
            ("repo", meta.repo.as_str()),
            ("collection", SKYBLUR_COLLECTION),
            ("rkey", meta.rkey.as_str()),
        ])
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch Skyblur record {}: {e}", meta.uri()))?
        .error_for_status()
        .map_err(|e| anyhow::anyhow!("Failed to fetch Skyblur record {}: {e}", meta.uri()))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse Skyblur record {}: {e}", meta.uri()))?;

    Ok(output.value.text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::post_event::SkyblurVisibility;

    fn test_meta(repo: &str, rkey: &str, visibility: SkyblurVisibility) -> SkyblurMeta {
        SkyblurMeta {
            repo: repo.to_string(),
            rkey: rkey.to_string(),
            visibility,
        }
    }

    #[test]
    fn converts_bracketed_ranges_to_traq_spoiler() -> anyhow::Result<()> {
        assert_eq!(
            convert_brackets_to_traq_spoiler(
                "ワルプルギスの廻天、[ほげほげ]好きなんだけど、[ふがふが]"
            ),
            "ワルプルギスの廻天、!!ほげほげ!!好きなんだけど、!!ふがふが!!"
        );
        Ok(())
    }

    #[test]
    fn converts_multiline_bracketed_range_as_a_whole() -> anyhow::Result<()> {
        assert_eq!(
            convert_brackets_to_traq_spoiler("見出し\n[秘密の本文\n続き]"),
            "見出し\n!!秘密の本文\n続き!!"
        );
        Ok(())
    }

    #[test]
    fn leaves_unclosed_and_nested_brackets_untouched() -> anyhow::Result<()> {
        assert_eq!(
            convert_brackets_to_traq_spoiler("これは[秘密です"),
            "これは[秘密です"
        );
        assert_eq!(
            convert_brackets_to_traq_spoiler("これは[秘密[です]]"),
            "これは[秘密[です]]"
        );
        assert_eq!(
            convert_brackets_to_traq_spoiler("括弧なしです"),
            "括弧なしです"
        );
        Ok(())
    }

    #[test]
    fn escapes_literal_exclamation_marks() -> anyhow::Result<()> {
        assert_eq!(
            convert_brackets_to_traq_spoiler("これは!!テストです"),
            "これは\\!!テストです"
        );
        assert_eq!(
            convert_brackets_to_traq_spoiler("[秘密!!です]"),
            "!!秘密\\!!です!!"
        );
        assert_eq!(convert_brackets_to_traq_spoiler("!![秘密]"), "\\!!!!秘密!!");
        Ok(())
    }

    #[test]
    fn removes_empty_brackets() -> anyhow::Result<()> {
        assert_eq!(convert_brackets_to_traq_spoiler("空[]です"), "空です");
        assert_eq!(convert_brackets_to_traq_spoiler("[]"), "");
        assert_eq!(convert_brackets_to_traq_spoiler("[]空[]です"), "空です");
        Ok(())
    }

    #[test]
    fn builds_skyblur_page_url() -> anyhow::Result<()> {
        let meta = test_meta(
            "did:plc:iba6craltg5onugrgwcwfizi",
            "3much6qlnokok",
            SkyblurVisibility::Public,
        );

        assert_eq!(
            skyblur_page_url(&meta),
            "https://skyblur.uk/post/did:plc:iba6craltg5onugrgwcwfizi/3much6qlnokok".to_string()
        );
        Ok(())
    }
}
