import { fresh } from "@fresh/plugin-vite"
import { heyApiPlugin } from "@hey-api/vite-plugin"
import { defineConfig } from "vite"

export default defineConfig({
  plugins: [
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
