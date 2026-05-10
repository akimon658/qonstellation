import { OAuth2Client } from "@badgateway/oauth2-client"
import { config } from "./config.ts"

export const oauth2Client = new OAuth2Client({
  server: config.traqBaseUrl,
  clientId: config.traqClientId,
  clientSecret: config.traqClientSecret,
  tokenEndpoint: "/api/v3/oauth2/token",
  authorizationEndpoint: "/api/v3/oauth2/authorize",
})
