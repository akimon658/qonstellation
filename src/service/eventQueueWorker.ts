import {
  AppBskyEmbedImages,
  AppBskyEmbedRecordWithMedia,
  AppBskyFeedPost,
} from "@atcute/bluesky"
import { is } from "@atcute/lexicons/validations"
import { retry } from "@std/async/retry"
import { buildAtProtoUri } from "../lib/atProto.ts"
import { uploadImages } from "../lib/image.ts"
import {
  getTraqMessageIdByAtProtoUri,
  savePostMetadata,
} from "../repository/post.ts"
import {
  deleteQueuedEvent,
  getOldestQueuedEvent,
} from "../repository/queuedEvent.ts"
import { getUserAccessToken, getUserSettingByDid } from "../repository/user.ts"
import { postMessage } from "../traq/sdk.gen.ts"
import { MessageBuilder } from "./messageBuilder.ts"

const MAX_RETRIES = 25
const MAX_TIMEOUT_MS = 3600 * 1000 // 1 hour

export class EventQueueWorker {
  private isRunning = false
  private notificationResolver?: () => void
  private abortController = new AbortController()

  public notify() {
    if (this.notificationResolver) {
      this.notificationResolver()
      this.notificationResolver = undefined
    }
  }

  private waitForNotification() {
    return new Promise<void>((resolve) => {
      this.notificationResolver = resolve
    })
  }

  public async start() {
    this.isRunning = true

    while (this.isRunning) {
      const top = await getOldestQueuedEvent()

      if (!top) {
        await this.waitForNotification()

        continue
      }

      const { id, event } = top

      if (
        event.kind !== "commit" || event.commit.operation !== "create" ||
        !is(AppBskyFeedPost.mainSchema, event.commit.record)
      ) {
        // This should never happen
        throw new Error(`Invalid event in queue: ${JSON.stringify(event)}`)
      }

      const record = event.commit.record
      const atProtoUri = buildAtProtoUri({
        userDid: event.did,
        recordKey: event.commit.rkey,
      })
      const traqMessageId = await getTraqMessageIdByAtProtoUri(atProtoUri)

      if (traqMessageId) {
        // This message is already posted to traQ
        await deleteQueuedEvent(id)
        console.warn(
          `Skipping post ${atProtoUri} because it is already posted to traQ with message ID ${traqMessageId}`,
        )

        continue
      }

      const userSetting = await getUserSettingByDid(event.did)
      const accessToken = await getUserAccessToken(userSetting.userId)

      const postToTraq = async () => {
        console.debug("Attempting to post message to traQ", atProtoUri)

        let images: AppBskyEmbedImages.Image[] | undefined
        let imageIds: string[] | undefined

        if (is(AppBskyEmbedImages.mainSchema, record.embed)) {
          images = record.embed.images
        } else if (
          is(
            AppBskyEmbedRecordWithMedia.mainSchema,
            record.embed,
          ) &&
          is(AppBskyEmbedImages.mainSchema, record.embed.media)
        ) {
          images = record.embed.media.images
        }

        if (images?.length) {
          imageIds = await uploadImages({
            accessToken,
            did: event.did,
            images,
            targetChannelId: userSetting.targetChannelId,
          })

          console.debug(
            `Uploaded images for post ${atProtoUri}: ${imageIds}`,
          )
        }

        const messageBuilder = new MessageBuilder({
          traqAccessToken: accessToken,
          targetChannelId: userSetting.targetChannelId,
        })
        const { data, error } = await postMessage({
          headers: {
            Authorization: `Bearer ${accessToken}`,
          },
          path: {
            channelId: userSetting.targetChannelId,
          },
          body: {
            content: await messageBuilder.build({
              imageIds,
              post: record,
            }),
          },
        })

        if (!data) {
          throw new Error("Failed to post message to traQ", { cause: error })
        }

        return data
      }

      try {
        const data = await retry(postToTraq, {
          maxAttempts: MAX_RETRIES,
          maxTimeout: MAX_TIMEOUT_MS,
          signal: this.abortController.signal,
        })

        await savePostMetadata({
          atProtoUri,
          traqMessageId: data.id,
        })
        await deleteQueuedEvent(id)
        console.info("Posted message to traQ", atProtoUri)
      } catch (err) {
        // Don't throw if the error is caused by aborting
        if (!this.abortController.signal.aborted) {
          throw err
        }
      }
    }
  }

  public stop() {
    this.isRunning = false

    this.notify() // Wake up the worker if it's waiting for notification
    this.abortController.abort()
  }
}
