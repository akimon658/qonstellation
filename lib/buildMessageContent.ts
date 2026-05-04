import { AppBskyEmbedRecord, type AppBskyFeedPost } from "@atcute/bluesky"
import { is, parseResourceUri } from "@atcute/lexicons"
import { getTraqMessageIdByAtProtoUri } from "../repository/post.ts"
import { config } from "./config.ts"

interface BuildMessageParams {
  post: AppBskyFeedPost.Main
  imageIds?: string[]
}

const encoder = new TextEncoder()
const decoder = new TextDecoder()

export const buildMessageContent = async (
  { post, imageIds }: BuildMessageParams,
) => {
  let text = post.text

  if (post.facets?.length) {
    let textBytes = encoder.encode(post.text)
    // Sort facets in reverse order to avoid affecting the byte offsets of subsequent facets
    const facets = post.facets.sort((a, b) =>
      b.index.byteStart - a.index.byteStart
    )

    for (const facet of facets) {
      const linkFeature = facet.features.find((f) =>
        f.$type === "app.bsky.richtext.facet#link"
      )

      if (!linkFeature) {
        continue
      }

      const uriBytes = encoder.encode(linkFeature.uri)
      const newBytes = new Uint8Array(
        facet.index.byteStart + uriBytes.length +
          (textBytes.length - facet.index.byteEnd),
      )

      newBytes.set(textBytes.subarray(0, facet.index.byteStart), 0)
      newBytes.set(uriBytes, facet.index.byteStart)
      newBytes.set(
        textBytes.subarray(facet.index.byteEnd),
        facet.index.byteStart + uriBytes.length,
      )

      textBytes = newBytes
    }

    text = decoder.decode(textBytes)
  }

  if (imageIds?.length) {
    const imageLinks = imageIds.map((id) => `${config.traqBaseUrl}/files/${id}`)
      .join("\n")

    text = text ? `${text}\n${imageLinks}` : imageLinks
  }

  if (
    post.embed?.$type === "app.bsky.embed.record" ||
    post.embed?.$type === "app.bsky.embed.recordWithMedia"
  ) {
    let embeddedRecordUriStr: string

    if (is(AppBskyEmbedRecord.mainSchema, post.embed.record)) {
      embeddedRecordUriStr = post.embed.record.record.uri
    } else {
      embeddedRecordUriStr = post.embed.record.uri
    }

    let urlToAppend: string
    const traqMessageId = await getTraqMessageIdByAtProtoUri(
      embeddedRecordUriStr,
    )

    if (traqMessageId) {
      // This message is already posted to traQ, so we can append its URL to the text
      urlToAppend = `${config.traqBaseUrl}/messages/${traqMessageId}`
    } else {
      // This message is not posted to traQ, so we should append the URL to the original post
      const embeddedRecordUri = parseResourceUri(embeddedRecordUriStr)

      if (embeddedRecordUri.ok) {
        urlToAppend =
          `https://bsky.app/profile/${embeddedRecordUri.value.repo}/post/${embeddedRecordUri.value.rkey}`
      } else {
        throw new Error("Invalid embedded record URI", {
          cause: embeddedRecordUri.error,
        })
      }
    }

    text = text ? `${text}\n${urlToAppend}` : urlToAppend
  }

  return text
}
