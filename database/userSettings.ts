import type { Generated } from "@kysely/kysely"

export interface UserSettingsTable {
  id: Generated<number>
  did: string
  target_channel_id: string
  user_id: string
}
