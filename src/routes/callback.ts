import { define } from "../lib/define.ts"
import { oauth2Client } from "../lib/oauth2.ts"
import { createSessionToken, setSessionToken } from "../lib/session.ts"
import { saveUser, saveUserTokens } from "../repository/user.ts"
import { getMe } from "../traq/index.ts"

export const handler = define.handlers({
  GET: async (ctx) => {
    const url = new URL(ctx.req.url)
    const redirectUri = `${url.origin}/callback`

    const token = await oauth2Client.authorizationCode.getTokenFromCodeRedirect(
      ctx.req.url,
      { redirectUri },
    )

    const { data: user } = await getMe({
      headers: {
        Authorization: `Bearer ${token.accessToken}`,
      },
    })

    if (!user) {
      throw new Error("Failed to get user info from traQ")
    }

    await saveUser(user.id)
    await saveUserTokens({
      userId: user.id,
      accessToken: token.accessToken,
    })

    const sessionToken = await createSessionToken(user.id)
    const headers = new Headers({ Location: "/" })

    setSessionToken(headers, sessionToken)

    return new Response(null, { status: 302, headers })
  },
})
