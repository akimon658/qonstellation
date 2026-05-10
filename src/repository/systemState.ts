import { db } from "../database/db.ts"

const SYSTEM_STATE_KEY_JETSTREAM_CURSOR = "jetstream_cursor"

export const getJetstreamCursor = async (): Promise<number | undefined> => {
  const result = await db.selectFrom("system_states")
    .select("value")
    .where("key", "=", SYSTEM_STATE_KEY_JETSTREAM_CURSOR)
    .executeTakeFirst()

  return result?.value
}

export const saveJetstreamCursor = async (cursor: number): Promise<void> => {
  await db.insertInto("system_states")
    .values({
      key: SYSTEM_STATE_KEY_JETSTREAM_CURSOR,
      value: cursor,
    })
    .onDuplicateKeyUpdate({
      value: cursor,
    })
    .execute()
}
