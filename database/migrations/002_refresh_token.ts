import { Kysely } from "@kysely/kysely"

export const up = async (db: Kysely<unknown>) => {
  await db.schema
    .alterTable("user_tokens")
    .dropColumn("refresh_token")
    .execute()
}
