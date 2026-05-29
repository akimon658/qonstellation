/// <reference types="@atcute/atproto" />
import type { AppBskyEmbedImages } from "@atcute/bluesky"
import { type Did } from "@atcute/lexicons"
import { isLegacyBlob } from "@atcute/lexicons/interfaces"
import { imageSize } from "image-size"
import { postFile } from "../traq/index.ts"
import { client } from "./blueskyClient.ts"

const TRAQ_IMAGE_MAX_PIXELS = 2560 * 1600

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

      const { data: uploadedFile } = await postFile({
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
        body: {
          channelId: targetChannelId,
          file: await resizeImage(downloadRes),
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

const resizeImage = async (imageBlob: Blob) => {
  const { height, width } = imageSize(await imageBlob.bytes())
  const imagePixels = height * width

  if (imagePixels <= TRAQ_IMAGE_MAX_PIXELS) {
    return imageBlob
  }

  const scale = Math.sqrt(TRAQ_IMAGE_MAX_PIXELS / imagePixels)
  const resizeHeight = Math.max(1, Math.floor(height * scale))
  const resizeWidth = Math.max(1, Math.floor(width * scale))
  const resizedBitmap = await createImageBitmap(imageBlob, {
    resizeHeight,
    resizeQuality: "medium",
    resizeWidth,
  })
  const offscreen = new OffscreenCanvas(resizeWidth, resizeHeight)
  const ctx = offscreen.getContext("bitmaprenderer")

  if (!ctx) {
    throw new Error("Failed to get bitmaprenderer context")
  }

  ctx.transferFromImageBitmap(resizedBitmap)

  const blob = await offscreen.convertToBlob({
    type: "image/webp",
  })

  resizedBitmap.close()

  return blob
}
