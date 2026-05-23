import type { JetstreamEvent } from "@atcute/jetstream"
import type { Generated, JSONColumnType } from "@kysely/kysely"

export interface QueuedEventsTable {
  id: Generated<number>
  event_json: JSONColumnType<JetstreamEvent>
}
