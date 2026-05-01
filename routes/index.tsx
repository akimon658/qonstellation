import { page } from "fresh"
import { define } from "../lib/define.ts"
import { oauth2Client } from "../lib/oauth2.ts"

export const handler = define.handlers({
  GET: async (ctx) => {
    const url = new URL(ctx.req.url)
    const redirectUri = `${url.origin}/callback`
    const authorizeUrl = await oauth2Client.authorizationCode.getAuthorizeUri({
      redirectUri,
    })

    return page({ authorizeUrl })
  },
})

export default define.page<typeof handler>((ctx) => (
  <div>
    <a href={ctx.data.authorizeUrl}>Login with traQ</a>
  </div>
))
