import { define } from "../../lib/define.ts"

export const handler = define.middleware((ctx) => {
  if (!ctx.state.userId) {
    return new Response(null, { status: 401 })
  }

  return ctx.next()
})
