import { OAuth2Client } from "@badgateway/oauth2-client"

export const oauth2Client = new OAuth2Client({
  server: "https://q.trap.jp/api/v3/oauth2",
  clientId: Deno.env.get("OAUTH_CLIENT_ID")!,
  clientSecret: Deno.env.get("OAUTH_CLIENT_SECRET")!,
})
