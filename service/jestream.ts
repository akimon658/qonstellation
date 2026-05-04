import {
  AppBskyEmbedImages,
  AppBskyEmbedRecordWithMedia,
  AppBskyEmbedVideo,
  AppBskyFeedPost,
} from "@atcute/bluesky"
import { JetstreamSubscription } from "@atcute/jetstream"
import { type Did, is } from "@atcute/lexicons"
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

client.setConfig({
  baseUrl: `${config.traqBaseUrl}/api/v3`,
})

export class JetstreamService {
  private subscription: JetstreamSubscription
  private cursor?: number
  private readonly subscribingDids: Did[]
  private stopResolve?: () => void
  private loopPromise?: Promise<void>

  constructor(opts: { wantedDids: Did[]; cursor?: number }) {
    this.cursor = opts.cursor
    this.subscribingDids = opts.wantedDids
    this.subscription = new JetstreamSubscription({
      url: [
        "wss://jetstream1.us-east.bsky.network",
        "wss://jetstream2.us-east.bsky.network",
        "wss://jetstream1.us-west.bsky.network",
        "wss://jetstream2.us-west.bsky.network",
      ],
      wantedCollections: ["app.bsky.feed.post"],
      wantedDids: opts.wantedDids,
      cursor: opts.cursor,
    })
  }

  private async runLoop(): Promise<void> {
    const stopPromise = new Promise<void>((resolve) => {
      this.stopResolve = resolve
    })

    const iterator = this.subscription[Symbol.asyncIterator]()

    try {
      while (true) {
        const result = await Promise.race([
          iterator.next(),
          stopPromise.then(
            (): IteratorReturnResult<undefined> => ({
              done: true,
              value: undefined,
            }),
          ),
        ])

        if (result.done) {
          break
        }

        const event = result.value

        if (event.kind !== "commit" || event.commit.operation !== "create") {
          this.cursor = event.time_us

          continue
        }

        try {
          if (!is(AppBskyFeedPost.mainSchema, event.commit.record)) {
            console.warn("Invalid record", event.commit.record)

            this.cursor = event.time_us

            continue
          }

          const atProtoUri = buildAtProtoUri({
            userDid: event.did,
            recordKey: event.commit.rkey,
          })
          const isReply = !!event.commit.record.reply?.parent

          if (
            is(AppBskyEmbedVideo.mainSchema, event.commit.record.embed) ||
            isReply
          ) {
            console.warn(
              `Skipping post ${atProtoUri} because it has video or is a reply.`,
            )

            this.cursor = event.time_us

            continue
          }

          const traqMessageId = await getTraqMessageIdByAtProtoUri(atProtoUri)

          if (traqMessageId) {
            // This message is already posted to traQ

            this.cursor = event.time_us

            continue
          }

          const userSetting = await getUserSettingByDid(event.did)
          const accessToken = await getUserAccessToken(userSetting.userId)
          let images: AppBskyEmbedImages.Image[] | undefined
          let imageIds: string[] | undefined

          if (is(AppBskyEmbedImages.mainSchema, event.commit.record.embed)) {
            images = event.commit.record.embed.images
          } else if (
            is(
              AppBskyEmbedRecordWithMedia.mainSchema,
              event.commit.record.embed,
            ) &&
            is(AppBskyEmbedImages.mainSchema, event.commit.record.embed.media)
          ) {
            images = event.commit.record.embed.media.images
          }

          if (images?.length) {
            imageIds = await uploadImages({
              accessToken,
              did: event.did,
              images: images,
              targetChannelId: userSetting.targetChannelId,
            })

            console.debug(
              `Uploaded images for post ${atProtoUri}: ${imageIds}`,
            )
          }

          const { data, error } = await postMessage({
            headers: {
              Authorization: `Bearer ${accessToken}`,
            },
            path: {
              channelId: userSetting.targetChannelId,
            },
            body: {
              content: await buildMessageContent({
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
        } catch (err) {
          console.error("Error handling Jetstream event:", err)
        }
      }
    } finally {
      await iterator.return()
    }
  }

  start() {
    if (this.subscribingDids.length === 0) {
      console.warn("No DIDs to subscribe. Jetstream will not start.")
      return
    }

    this.loopPromise = this.runLoop()
  }

  async close() {
    this.stopResolve?.()
    await this.loopPromise

    if (this.cursor) {
      await saveJetstreamCursor(this.cursor)
    }
  }
}
