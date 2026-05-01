import { define } from "../lib/define.ts"
import { getSessionToken, verifySessionToken } from "../lib/session.ts"

export const handler = define.middleware(async (ctx) => {
  const token = getSessionToken(ctx.req)

  if (token) {
    ctx.state.userId = await verifySessionToken(token)
  }

  return ctx.next()
})
