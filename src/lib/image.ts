/// <reference types="@atcute/atproto" />
import type { AppBskyEmbedImages } from "@atcute/bluesky"
import { type Did } from "@atcute/lexicons"
import { isLegacyBlob } from "@atcute/lexicons/interfaces"
import {
  ImageMagick,
  initializeImageMagick,
  MagickFormat,
} from "@imagemagick/magick-wasm"
import wasmBytes from "@imagemagick/magick-wasm/magick.wasm" with {
  type: "bytes"
}
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

const isUint8ArrayOfArrayBuffer = (
  data: Uint8Array,
): data is Uint8Array<ArrayBuffer> => data.buffer instanceof ArrayBuffer

const initMagickPromise = initializeImageMagick(wasmBytes)

const resizeImage = async (imageBlob: Blob) => {
  await initMagickPromise

  const bytes = ImageMagick.read(await imageBlob.bytes(), (image) => {
    const imagePixels = image.height * image.width

    if (imagePixels > TRAQ_IMAGE_MAX_PIXELS) {
      const scale = Math.sqrt(TRAQ_IMAGE_MAX_PIXELS / imagePixels)
      const newHeight = Math.max(1, Math.floor(image.height * scale))
      const newWidth = Math.max(1, Math.floor(image.width * scale))

      image.resize(newWidth, newHeight)
    }

    return image.write(MagickFormat.WebP, (data) => {
      if (isUint8ArrayOfArrayBuffer(data)) {
        return data
      }

      throw new Error("Unexpected data type from ImageMagick")
    })
  })

  return new Blob([bytes], { type: "image/webp" })
}
