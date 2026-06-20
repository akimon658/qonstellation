import {
  AppBskyEmbedRecord,
  AppBskyEmbedRecordWithMedia,
  type AppBskyFeedPost,
} from "@atcute/bluesky"
import {
  type ParsedResourceUri,
  parseResourceUri,
} from "@atcute/lexicons/syntax"
import { is } from "@atcute/lexicons/validations"
import { config } from "../lib/config.ts"
import { getTraqMessageIdByAtProtoUri } from "../repository/post.ts"
import { getMessages } from "../traq/sdk.gen.ts"

interface MessageBuilderConstructorParams {
  targetChannelId: string
  traqAccessToken: string
}

interface BuildMessageParams {
  post: AppBskyFeedPost.Main
  imageIds?: string[]
}

const encoder = new TextEncoder()
const decoder = new TextDecoder()

export class MessageBuilder {
  private readonly targetChannelId: string
  private readonly traqAccessToken: string

  constructor(
    { targetChannelId, traqAccessToken }: MessageBuilderConstructorParams,
  ) {
    this.targetChannelId = targetChannelId
    this.traqAccessToken = traqAccessToken
  }

  async build({ post, imageIds }: BuildMessageParams) {
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
      const imageLinks = imageIds.map((id) =>
        `${config.traqBaseUrl}/files/${id}`
      )
        .join("\n")

      text = text ? `${text}\n${imageLinks}` : imageLinks
    }

    if (post.reply) {
      let urlToAppend: string | undefined
      const traqMessageId = await getTraqMessageIdByAtProtoUri(
        post.reply.parent.uri,
      )

      if (traqMessageId) {
        const latestMessageInChannel = await getMessages({
          headers: {
            Authorization: `Bearer ${this.traqAccessToken}`,
          },
          path: {
            channelId: this.targetChannelId,
          },
          query: {
            limit: 1,
          },
        })
        const shouldAppendUrl =
          latestMessageInChannel.data?.at(0)?.id !== traqMessageId

        if (shouldAppendUrl) {
          urlToAppend = getTraqMessageUrl(traqMessageId)
        }
      } else {
        // This message is not posted to traQ, so we should append the URL to the original post
        urlToAppend = getBlueskyPostUrl(post.reply.parent.uri)
      }

      if (urlToAppend) {
        text = text ? `${text}\n${urlToAppend}` : urlToAppend
      }
    }

    if (
      is(AppBskyEmbedRecord.mainSchema, post.embed) ||
      is(AppBskyEmbedRecordWithMedia.mainSchema, post.embed)
    ) {
      let embeddedRecordUriStr: string

      if (is(AppBskyEmbedRecord.mainSchema, post.embed.record)) {
        embeddedRecordUriStr = post.embed.record.record.uri
      } else {
        embeddedRecordUriStr = post.embed.record.uri
      }

      const embeddedRecordUri = parseResourceUri(embeddedRecordUriStr)

      if (!embeddedRecordUri.ok) {
        throw new Error("Invalid embedded record URI", {
          cause: embeddedRecordUri.error,
        })
      }

      if (embeddedRecordUri.value.collection === "app.bsky.feed.post") {
        let urlToAppend: string
        const traqMessageId = await getTraqMessageIdByAtProtoUri(
          embeddedRecordUriStr,
        )

        if (traqMessageId) {
          // This message is already posted to traQ, so we can append its URL to the text
          urlToAppend = getTraqMessageUrl(traqMessageId)
        } else {
          // This message is not posted to traQ, so we should append the URL to the original post
          urlToAppend = getBlueskyPostUrl(embeddedRecordUri.value)
        }

        text = text ? `${text}\n${urlToAppend}` : urlToAppend
      }
    }

    return text
  }
}

const getTraqMessageUrl = (messageId: string) => {
  return `${config.traqBaseUrl}/messages/${messageId}`
}

const getBlueskyPostUrl = (resourceUri: string | ParsedResourceUri) => {
  let uri: ParsedResourceUri

  if (typeof resourceUri === "string") {
    const parsed = parseResourceUri(resourceUri)

    if (!parsed.ok) {
      throw new Error("Invalid post URI", { cause: parsed.error })
    }

    uri = parsed.value
  } else {
    uri = resourceUri
  }

  return `https://bsky.app/profile/${uri.repo}/post/${uri.rkey}`
}
