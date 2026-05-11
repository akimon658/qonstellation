import { createClient } from "@hey-api/openapi-ts"
import { Builder } from "fresh/dev"

await createClient({
  input:
    "https://github.com/traPtitech/traQ/raw/refs/tags/v3.28.1/docs/v3-api.yaml",
  output: "src/traq",
})

const builder = new Builder({
  routeDir: "./src/routes",
  serverEntry: "./src/main.ts",
})

if (Deno.args.includes("build")) {
  await builder.build()
  await Deno.bundle({
    codeSplitting: true,
    entrypoints: ["./_fresh/server.js"],
    outputDir: "./dist",
    platform: "deno",
  })
} else {
  await builder.listen(() => import("../src/main.ts"))
}
