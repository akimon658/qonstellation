import { Client } from "@atcute/client"
import { PasswordSession } from "@atcute/password-session"
import { config } from "./config.ts"

export const client = new Client({
  handler: await PasswordSession.login({
    service: config.blueskyHostingProvider,
    identifier: config.blueskyAccountIdentifier,
    password: config.blueskyAppPassword,
  }),
})
