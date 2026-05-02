import { Jetstream } from "@skyware/jetstream"
import { buildAtProtoUri } from "../lib/atProto.ts"
import { config } from "../lib/config.ts"
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
  private jetstream: Jetstream
  private cursor?: number
  private readonly subscribingDids: string[]

  constructor(opts: { wantedDids: string[]; cursor?: number }) {
    this.cursor = opts.cursor
    this.subscribingDids = opts.wantedDids
    this.jetstream = new Jetstream({
      wantedDids: opts.wantedDids,
      cursor: opts.cursor,
    })
    this.jetstream.onCreate("app.bsky.feed.post", async (event) => {
      if (event.commit.record.embed || event.commit.record.reply?.parent) {
        // Ignore replies posts with embed for now
        return
      }

      const atProtoUri = buildAtProtoUri({
        userDid: event.did,
        recordKey: event.commit.rkey,
      })
      const traqMessageId = await getTraqMessageIdByAtProtoUri(atProtoUri)

      if (traqMessageId) {
        // This message is already posted to traQ
        return
      }

      const userSetting = await getUserSettingByDid(event.did)
      const accessToken = await getUserAccessToken(userSetting.userId)
      const { data } = await postMessage({
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
        path: {
          channelId: userSetting.targetChannelId,
        },
        body: {
          content: event.commit.record.text,
        },
      })

      if (!data) {
        throw new Error("Failed to post message to traQ")
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
