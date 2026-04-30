import type { Generated } from "@kysely/kysely"

export interface UserTokensTable {
  id: Generated<number>
  accessToken: string
  refreshToken: string
  userId: string
}
