import type { Did } from "@atcute/lexicons"
import type { Generated } from "@kysely/kysely"

export interface UserSettingsTable {
  id: Generated<number>
  did: Did
  target_channel_id: string
  user_id: string
}
