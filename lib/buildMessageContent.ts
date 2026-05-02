import type { AppBskyFeedPost } from "@atcute/bluesky"
import { config } from "./config.ts"

interface BuildMessageParams {
  post: AppBskyFeedPost.Main
  imageIds?: string[]
}

const encoder = new TextEncoder()
const decoder = new TextDecoder()

export const buildMessageContent = ({ post, imageIds }: BuildMessageParams) => {
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

  if (imageIds) {
    const imageLinks = imageIds.map((id) => `${config.traqBaseUrl}/files/${id}`)
      .join("\n")

    text = text ? `${text}\n${imageLinks}` : imageLinks
  }

  return text
}
