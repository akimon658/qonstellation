import { AppBskyFeedPost } from "@atcute/bluesky"
import { JetstreamSubscription } from "@atcute/jetstream"
import { type Did, is } from "@atcute/lexicons"
import { debounce } from "@std/async"
import { buildAtProtoUri } from "../lib/atProto.ts"
import { config } from "../lib/config.ts"
import { isSelfThread } from "../lib/thread.ts"
import { addQueuedEvent } from "../repository/queuedEvent.ts"
import { saveJetstreamCursor } from "../repository/systemState.ts"
import { client } from "../traq/client.gen.ts"
import { EventQueueWorker } from "./eventQueueWorker.ts"

const UPDATE_CURSOR_DEBOUNCE_MS = 30000

client.setConfig({
  baseUrl: `${config.traqBaseUrl}/api/v3`,
})

export class JetstreamService {
  private subscription: JetstreamSubscription
  private readonly subscribingDids: Did[]
  private eventQueueWorker = new EventQueueWorker()
  private stopResolve?: () => void
  private loopPromise?: Promise<void>
  private workerPromise?: Promise<void>

  constructor(opts: { wantedDids: Did[]; cursor?: number }) {
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

  private updateCursor = debounce(
    saveJetstreamCursor,
    UPDATE_CURSOR_DEBOUNCE_MS,
  )

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
          console.debug("Skipping non-create commit event", event.did)
          this.updateCursor(event.time_us)

          continue
        }

        try {
          if (!is(AppBskyFeedPost.mainSchema, event.commit.record)) {
            console.warn("Invalid record", event.commit.record)
            this.updateCursor(event.time_us)

            continue
          }

          const atProtoUri = buildAtProtoUri({
            userDid: event.did,
            recordKey: event.commit.rkey,
          })

          if (
            !await isSelfThread({
              post: event.commit.record,
              authorDid: event.did,
            })
          ) {
            console.warn(
              `Skipping post ${atProtoUri} because it is not a self thread`,
            )
            this.updateCursor(event.time_us)

            continue
          }

          await addQueuedEvent(event)
          this.eventQueueWorker.notify()
          this.updateCursor(event.time_us)
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

    this.workerPromise = this.eventQueueWorker.start()
    this.loopPromise = this.runLoop()
  }

  async close() {
    this.stopResolve?.()
    await this.loopPromise
    this.eventQueueWorker.stop()
    await this.workerPromise
    this.updateCursor.flush()
  }
}
