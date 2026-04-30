import { Jetstream } from "@skyware/jetstream"
import { buildAtProtoUri } from "../lib/atProto.ts"
import {
  getTraqMessageIdByAtProtoUri,
  savePostMetadata,
} from "../repository/post.ts"
import { saveJetstreamCursor } from "../repository/systemState.ts"
import { getUserAccessToken, getUserSettingByDid } from "../repository/user.ts"
import { postMessage } from "./traq.ts"

export class JetstreamService {
  private jetstream: Jetstream
  private cursor?: number

  constructor(opts: { wantedDids: string[]; cursor?: number }) {
    this.cursor = opts.cursor
    this.jetstream = new Jetstream({
      wantedDids: opts.wantedDids,
      cursor: opts.cursor,
    })
    this.jetstream.onCreate("app.bsky.feed.post", async (event) => {
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
      const messageId = await postMessage({
        token: await getUserAccessToken(userSetting.userId),
        channelId: userSetting.targetChannelId,
        content: event.commit.record.text,
      })

      await savePostMetadata({
        atProtoUri,
        traqMessageId: messageId,
      })

      this.cursor = event.time_us
    })
  }

  start() {
    this.jetstream.start()
  }

  async close() {
    this.jetstream.close()

    if (this.cursor) {
      await saveJetstreamCursor(this.cursor)
    }
  }
}
