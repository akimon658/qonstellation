import type { Kysely } from "@kysely/kysely"

const up = async (db: Kysely<unknown>) => {
  await db.schema
    .alterTable("system_states")
    .modifyColumn("value", "bigint", (col) => col.notNull())
    .execute()
}

export default { up }
