import { AppBskyFeedPost } from "@atcute/bluesky"
import { Jetstream } from "@skyware/jetstream"
import { buildAtProtoUri } from "../lib/atProto.ts"
import { buildMessageContent } from "../lib/buildMessageContent.ts"
import { config } from "../lib/config.ts"
import { uploadImages } from "../lib/image.ts"
import {
  getTraqMessageIdByAtProtoUri,
  savePostMetadata,
} from "../repository/post.ts"
import { saveJetstreamCursor } from "../repository/systemState.ts"
import { getUserAccessToken, getUserSettingByDid } from "../repository/user.ts"
import { client } from "../traq/client.gen.ts"
import { postMessage } from "../traq/index.ts"

type EmbedType = NonNullable<AppBskyFeedPost.Main["embed"]>["$type"]

const SUPPORTED_EMBED_TYPES: readonly EmbedType[] = [
  "app.bsky.embed.external",
  "app.bsky.embed.images",
] as const

client.setConfig({
  baseUrl: `${config.traqBaseUrl}/api/v3`,
})

export class JetstreamService {
  private jetstream: Jetstream
  private cursor?: number
  private readonly subscribingDids: string[]

  constructor(opts: { wantedDids: string[]; cursor?: number }) {
    this.cursor = opts.cursor
    this.subscribingDids = opts.wantedDids
    this.jetstream = new Jetstream({
      wantedCollections: ["app.bsky.feed.post"],
      wantedDids: opts.wantedDids,
      cursor: opts.cursor,
    })
    this.jetstream.onCreate("app.bsky.feed.post", async (event) => {
      const atProtoUri = buildAtProtoUri({
        userDid: event.did,
        recordKey: event.commit.rkey,
      })
      const hasNonSupportedEmbed = event.commit.record.embed &&
        !SUPPORTED_EMBED_TYPES.includes(event.commit.record.embed.$type)
      const isReply = !!event.commit.record.reply?.parent

      if (hasNonSupportedEmbed || isReply) {
        console.warn(
          `Skipping post ${atProtoUri} because it has unsupported embed or is a reply.`,
        )
        return
      }

      const traqMessageId = await getTraqMessageIdByAtProtoUri(atProtoUri)

      if (traqMessageId) {
        // This message is already posted to traQ
        return
      }

      const userSetting = await getUserSettingByDid(event.did)
      const accessToken = await getUserAccessToken(userSetting.userId)
      let imageIds: string[] | undefined

      if (event.commit.record.embed?.$type === "app.bsky.embed.images") {
        imageIds = await uploadImages({
          accessToken,
          did: event.did,
          images: event.commit.record.embed.images,
          targetChannelId: userSetting.targetChannelId,
        })

        console.debug(`Uploaded images for post ${atProtoUri}: ${imageIds}`)
      }

      const { data, error } = await postMessage({
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
        path: {
          channelId: userSetting.targetChannelId,
        },
        body: {
          content: buildMessageContent({
            imageIds,
            post: event.commit.record,
          }),
        },
      })

      if (!data) {
        throw new Error("Failed to post message to traQ", { cause: error })
      }

      await savePostMetadata({
        atProtoUri,
        traqMessageId: data.id,
      })

      this.cursor = event.time_us
    })
  }

  start() {
    if (this.subscribingDids.length === 0) {
      console.warn("No DIDs to subscribe. Jetstream will not start.")
      return
    }

    this.jetstream.start()
  }

  async close() {
    this.jetstream.close()

    if (this.cursor) {
      await saveJetstreamCursor(this.cursor)
    }
  }
}
