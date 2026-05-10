import { page } from "fresh"
import SettingsForm from "../islands/SettingsForm.tsx"
import { define } from "../lib/define.ts"
import { oauth2Client } from "../lib/oauth2.ts"
import { getUserSettingByUserId } from "../repository/user.ts"

export const handler = define.handlers({
  GET: async (ctx) => {
    if (ctx.state.userId) {
      const setting = await getUserSettingByUserId(ctx.state.userId)
      return page({
        loggedIn: true as const,
        did: setting?.did ?? "",
        targetChannelId: setting?.targetChannelId ?? "",
      })
    }

    const url = new URL(ctx.req.url)
    const redirectUri = `${url.origin}/callback`
    const authorizeUrl = await oauth2Client.authorizationCode.getAuthorizeUri({
      redirectUri,
    })

    return page({ loggedIn: false as const, authorizeUrl })
  },
})

export default define.page<typeof handler>((ctx) => {
  if (!ctx.data.loggedIn) {
    return (
      <div>
        <a href={ctx.data.authorizeUrl}>Login with traQ</a>
      </div>
    )
  }

  return (
    <div>
      <h1>Settings</h1>
      <SettingsForm
        did={ctx.data.did}
        targetChannelId={ctx.data.targetChannelId}
      />
    </div>
  )
})
