import type { AppBskyEmbedImages } from "@atcute/bluesky"
import { Client, simpleFetchHandler } from "@atcute/client"
import { type Did } from "@atcute/lexicons"
import { isLegacyBlob } from "@atcute/lexicons/interfaces"
import { postFile } from "../traq/index.ts"

const client = new Client({
  handler: simpleFetchHandler({
    service: "https://bsky.social",
  }),
})

interface UploadImageParams {
  accessToken: string
  did: Did
  images: AppBskyEmbedImages.Image[]
  targetChannelId: string
}

export const uploadImages = async (
  { accessToken, did, images, targetChannelId }: UploadImageParams,
) => {
  const imageIds = await Promise.all(
    images.map(async (imageMeta) => {
      if (isLegacyBlob(imageMeta.image)) {
        throw new Error("Legacy blobs are not supported")
      }

      const { data: downloadRes, ok } = await client.get(
        "com.atproto.sync.getBlob",
        {
          as: "blob",
          params: {
            did,
            cid: imageMeta.image.ref.$link,
          },
        },
      )

      if (!ok) {
        throw new Error(
          `Failed to download image: ${imageMeta.image.ref.$link}`,
        )
      }

      if (!(downloadRes instanceof Blob)) {
        throw new Error(
          `Unexpected response type when downloading image: ${imageMeta.image.ref.$link}`,
        )
      }

      const { data: uploadedFile } = await postFile({
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
        body: {
          channelId: targetChannelId,
          file: downloadRes,
        },
      })

      if (!uploadedFile) {
        throw new Error(`Failed to upload image: ${imageMeta.image.ref.$link}`)
      }

      return uploadedFile.id
    }),
  )

  return imageIds
}
