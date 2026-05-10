import { Kysely } from "@kysely/kysely"

const up = async (db: Kysely<unknown>) => {
  await db.schema
    .alterTable("user_tokens")
    .dropColumn("refresh_token")
    .execute()
}

export default { up }
