import type { Kysely } from "@kysely/kysely"

const up = async (db: Kysely<unknown>) => {
  await db.schema
    .createTable("users")
    .addColumn("id", "uuid", (col) => col.primaryKey())
    .execute()

  await db.schema
    .createTable("posts")
    .addColumn("id", "integer", (col) => col.primaryKey().autoIncrement())
    .addColumn("at_proto_uri", "varchar(8192)", (col) => col.notNull())
    .addColumn("traq_message_id", "uuid", (col) => col.notNull())
    .execute()

  await db.schema
    .createTable("system_states")
    .addColumn("key", "varchar(16)", (col) => col.primaryKey())
    .addColumn("value", "integer", (col) => col.notNull())
    .execute()

  await db.schema
    .createTable("user_settings")
    .addColumn(
      "id",
      "integer",
      (col) => col.primaryKey().autoIncrement(),
    )
    .addColumn("did", "varchar(2048)", (col) => col.notNull())
    .addColumn("target_channel_id", "uuid", (col) => col.notNull())
    .addColumn(
      "user_id",
      "uuid",
      (col) => col.notNull().unique().references("users.id"),
    )
    .execute()

  await db.schema
    .createTable("user_tokens")
    .addColumn(
      "id",
      "integer",
      (col) => col.primaryKey().autoIncrement(),
    )
    .addColumn("access_token", "varchar(36)", (col) => col.notNull())
    .addColumn("refresh_token", "varchar(36)", (col) => col.notNull())
    .addColumn(
      "user_id",
      "uuid",
      (col) => col.notNull().unique().references("users.id"),
    )
    .execute()
}

const down = async (db: Kysely<unknown>) => {
  await db.schema.dropTable("user_tokens").execute()
  await db.schema.dropTable("user_settings").execute()
  await db.schema.dropTable("system_states").execute()
  await db.schema.dropTable("posts").execute()
  await db.schema.dropTable("users").execute()
}

export default { up, down }
