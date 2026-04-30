import type { Generated } from "@kysely/kysely"

export interface UserSettingsTable {
  id: Generated<number>
  did: string
  targetChannelId: string
  userId: string
}
