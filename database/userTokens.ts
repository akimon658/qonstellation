import type { Generated } from "@kysely/kysely"

export interface UserTokensTable {
  id: Generated<number>
  access_token: string
  refresh_token: string
  user_id: string
}
