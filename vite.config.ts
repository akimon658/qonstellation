import { fresh } from "@fresh/plugin-vite"
import { heyApiPlugin } from "@hey-api/vite-plugin"
import { defineConfig, type Plugin } from "vite"

const WASM_ID_PREFIX = "\0wasm:"

/**
 * Vite plugin to load WASM files as Uint8Array
 *
 * Deno supports loading WASM files directly, but Vite doesn't.
 * And using params like `?url` or `?raw` doesn't work for external packages,
 * so we need to handle `.wasm` files manually.
 */
const wasmPlugin = (): Plugin => {
  return {
    name: "wasm-loader",
    enforce: "pre",
    async resolveId(id, importer, options) {
      if (!id.endsWith(".wasm")) {
        return null
      }

      const resolved = await this.resolve(id, importer, {
        ...options,
        skipSelf: true,
      })

      if (!resolved) {
        return null
      }

      return WASM_ID_PREFIX + resolved.id
    },
    load: async (id) => {
      if (!id.startsWith(WASM_ID_PREFIX)) {
        return null
      }

      const path = id.slice(WASM_ID_PREFIX.length)
      const bytes = await Deno.readFile(path)

      return `export default new Uint8Array(${
        JSON.stringify(Array.from(bytes))
      })`
    },
  }
}

export default defineConfig({
  resolve: {
    alias: {
      "iconv-lite": "@subframe7536/iconv-lite",
    },
  },
  plugins: [
    wasmPlugin(), // Must be loaded first to handle WASM imports correctly
    fresh(),
    heyApiPlugin(
      {
        config: {
          input:
            "https://github.com/traPtitech/traQ/raw/refs/tags/v3.28.1/docs/v3-api.yaml",
          output: "traq",
        },
      },
    ),
  ],
})
