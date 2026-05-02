import type { Kysely } from "@kysely/kysely"

const up = async (db: Kysely<unknown>) => {
  await db.schema
    .alterTable("system_states")
    .alterColumn("value", (col) => col.setDataType("bigint"))
    .execute()
}

const down = async (db: Kysely<unknown>) => {
  await db.schema
    .alterTable("system_states")
    .alterColumn("value", (col) => col.setDataType("integer"))
    .execute()
}

export default { down, up }
