import { createDefine } from "fresh"

interface State {
  userId?: string
}

export const define = createDefine<State>()
