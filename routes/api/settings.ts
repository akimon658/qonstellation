import { define } from "../../lib/define.ts"
import {
  getUserSettingByUserId,
  saveUserSettings,
} from "../../repository/user.ts"

export const handler = define.handlers({
  GET: async (ctx) => {
    const setting = await getUserSettingByUserId(ctx.state.userId!)

    return Response.json(setting ?? {})
  },

  PUT: async (ctx) => {
    const { did, targetChannelId } = await ctx.req.json()

    if (typeof did !== "string" || typeof targetChannelId !== "string") {
      return Response.json(
        { error: "did and targetChannelId are required" },
        { status: 400 },
      )
    }

    await saveUserSettings(ctx.state.userId!, did, targetChannelId)

    return Response.json({ ok: true })
  },
})
