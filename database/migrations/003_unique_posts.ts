import { Kysely } from "@kysely/kysely"

const up = async (db: Kysely<unknown>) => {
  await db.schema
    .createIndex("idx_at_proto_uri")
    .on("posts")
    .column("at_proto_uri")
    .unique()
    .execute()
}

const down = async (db: Kysely<unknown>) => {
  await db.schema
    .dropIndex("idx_at_proto_uri")
    .on("posts")
    .execute()
}

export default { up, down }
