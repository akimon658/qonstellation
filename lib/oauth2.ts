import { OAuth2Client } from "@badgateway/oauth2-client"

export const oauth2Client = new OAuth2Client({
  server: "https://q.trap.jp",
  clientId: Deno.env.get("TRAQ_CLIENT_ID")!,
  clientSecret: Deno.env.get("TRAQ_CLIENT_SECRET")!,
  tokenEndpoint: "/api/v3/oauth2/token",
  authorizationEndpoint: "/api/v3/oauth2/authorize",
})
