import type { JetstreamEvent } from "@atcute/jetstream"
import { db } from "../database/db.ts"

interface QueuedEvent {
  id: number
  event: JetstreamEvent
}

export const getOldestQueuedEvent = async (): Promise<
  QueuedEvent | undefined
> => {
  const result = await db
    .selectFrom("queued_events")
    .selectAll()
    .orderBy("id", "asc")
    .executeTakeFirst()

  return result
    ? {
      id: result.id,
      event: result.event_json,
    }
    : undefined
}

export const addQueuedEvent = async (event: JetstreamEvent) => {
  await db
    .insertInto("queued_events")
    .values({
      event_json: JSON.stringify(event),
    })
    .execute()
}

export const deleteQueuedEvent = async (id: number) => {
  await db
    .deleteFrom("queued_events")
    .where("id", "=", id)
    .execute()
}
