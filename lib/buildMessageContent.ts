import { AppBskyFeedPost } from "@atcute/bluesky"

interface BuildMessageParams {
  post: AppBskyFeedPost.Main
}

const encoder = new TextEncoder()
const decoder = new TextDecoder()

export const buildMessageContent = ({ post }: BuildMessageParams) => {
  if (!post.facets?.length) {
    return post.text
  }

  let textBytes = encoder.encode(post.text)
  const facets = post.facets.sort((a, b) =>
    a.index.byteStart - b.index.byteStart
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

  return decoder.decode(textBytes)
}
