/// <reference types="@atcute/atproto" />
import type { AppBskyEmbedImages, AppBskyEmbedVideo } from "@atcute/bluesky"
import { type Did } from "@atcute/lexicons"
import { isLegacyBlob } from "@atcute/lexicons/interfaces"
import {
  ImageMagick,
  initializeImageMagick,
  MagickFormat,
} from "@imagemagick/magick-wasm"
import { postFile } from "../traq/index.ts"
import { client } from "./blueskyClient.ts"

const TRAQ_IMAGE_MAX_PIXELS = 2560 * 1600

declare global {
  var gc: (() => void) | undefined
}

interface UploadImageParams {
  accessToken: string
  did: Did
  images: AppBskyEmbedImages.Image[]
  targetChannelId: string
}

export const uploadImages = async (
  { accessToken, did, images, targetChannelId }: UploadImageParams,
) => {
  const imageIds = []

  // Process each image sequentially to reduce memory usage and avoid overwhelming the server with concurrent requests.
  for (const imageMeta of images) {
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

    imageIds.push(uploadedFile.id)

    if (typeof gc === "function") {
      gc()
    }
  }

  return imageIds
}

interface UploadVideoParams {
  accessToken: string
  did: Did
  video: AppBskyEmbedVideo.Main
  targetChannelId: string
}

export const uploadVideo = async (
  { accessToken, did, video, targetChannelId }: UploadVideoParams,
) => {
  if (isLegacyBlob(video.video)) {
    throw new Error("Legacy blobs are not supported")
  }

  const { data: downloadRes, ok } = await client.get(
    "com.atproto.sync.getBlob",
    {
      as: "blob",
      params: {
        did,
        cid: video.video.ref.$link,
      },
    },
  )

  if (!ok) {
    throw new Error(
      `Failed to download video: ${video.video.ref.$link}`,
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
    throw new Error(`Failed to upload video: ${video.video.ref.$link}`)
  }

  return uploadedFile.id
}

const isUint8ArrayOfArrayBuffer = (
  data: Uint8Array,
): data is Uint8Array<ArrayBuffer> => data.buffer instanceof ArrayBuffer

const wasmUrl = import.meta.env.DEV
  ? new URL(
    "../../node_modules/@imagemagick/magick-wasm/dist/magick.wasm",
    import.meta.url,
  )
  : new URL("./magick.wasm", import.meta.url)
// `initializeImageMagick` accepts URL, but does not support protocols other than http(s).
// So we fetch the wasm file ourselves.
const initMagickPromise = fetch(wasmUrl).then((res) => res.arrayBuffer())
  .then((buf) => initializeImageMagick(buf))

const resizeImage = async (imageBlob: Blob) => {
  await initMagickPromise

  return ImageMagick.read(await imageBlob.bytes(), (image) => {
    const imagePixels = image.height * image.width

    if (imagePixels > TRAQ_IMAGE_MAX_PIXELS) {
      const scale = Math.sqrt(TRAQ_IMAGE_MAX_PIXELS / imagePixels)
      const newHeight = Math.max(1, Math.floor(image.height * scale))
      const newWidth = Math.max(1, Math.floor(image.width * scale))

      image.resize(newWidth, newHeight)
    }

    return image.write(MagickFormat.WebP, (data) => {
      if (isUint8ArrayOfArrayBuffer(data)) {
        return new Blob([data], { type: "image/webp" })
      }

      throw new Error("Unexpected data type from ImageMagick")
    })
  })
}
