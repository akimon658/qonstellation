import { AppBskyFeedDefs, type AppBskyFeedPost } from "@atcute/bluesky"
import { ok } from "@atcute/client"
import { type Did, is } from "@atcute/lexicons"
import { client } from "./blueskyClient.ts"

const MAX_PARENT_HEIGHT = 1000

interface IsSelfThreadParams {
  post: AppBskyFeedPost.Main
  authorDid: Did
}

export const isSelfThread = async (
  { post, authorDid }: IsSelfThreadParams,
): Promise<boolean> => {
  if (!post.reply) {
    return true
  }

  const { thread } = await ok(client.get("app.bsky.feed.getPostThread", {
    params: {
      uri: post.reply.parent.uri,
      depth: 0,
      parentHeight: MAX_PARENT_HEIGHT,
    },
  }))

  let current = thread

  while (is(AppBskyFeedDefs.threadViewPostSchema, current)) {
    if (current.post.author.did !== authorDid) {
      return false
    }

    if (!current.parent) {
      return true
    }

    current = current.parent
  }

  // NotFoundPost or BlockedPost — can't confirm all parents are self
  return false
}
