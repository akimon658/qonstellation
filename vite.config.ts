import { fresh } from "@fresh/plugin-vite"
import { heyApiPlugin } from "@hey-api/vite-plugin"
import { defineConfig } from "vite"
import { viteStaticCopy } from "vite-plugin-static-copy"

export default defineConfig({
  plugins: [
    fresh({
      serverEntry: "./src/main.ts",
    }),
    heyApiPlugin({
      config: {
        input:
          "https://github.com/traPtitech/traQ/raw/refs/tags/v3.28.1/docs/v3-api.yaml",
        output: "src/traq",
      },
    }),
    viteStaticCopy({
      environment: "ssr",
      targets: [
        {
          src: "node_modules/@imagemagick/magick-wasm/dist/magick.wasm",
          dest: ".",
          rename: { stripBase: true },
        },
      ],
    }),
  ],
  resolve: {
    alias: {
      "iconv-lite": "@subframe7536/iconv-lite",
    },
  },
})
