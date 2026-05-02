import type { Kysely } from "@kysely/kysely"

const up = async (db: Kysely<unknown>) => {
  await db.schema
    .alterTable("system_states")
    .alterColumn("value", (col) => col.setDataType("bigint"))
    .execute()
}

export default { up }
