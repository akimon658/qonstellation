import type { Kysely } from "@kysely/kysely"

const up = async (db: Kysely<unknown>) => {
  await db.schema
    .createTable("queued_events")
    .addColumn(
      "id",
      "integer",
      (col) => col.primaryKey().autoIncrement(),
    )
    .addColumn("event_json", "json", (col) => col.notNull())
    .execute()
}

const down = async (db: Kysely<unknown>) => {
  await db.schema.dropTable("queued_events").execute()
}

export default { down, up }
