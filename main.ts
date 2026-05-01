import { Migrator } from "@kysely/kysely"
import { App, staticFiles } from "fresh"
import { db } from "./database/db.ts"
import { migrationProvider } from "./database/migrator.ts"
import { getJetstreamCursor } from "./repository/systemState.ts"
import { getAllDids } from "./repository/user.ts"
import { JetstreamService } from "./service/jestream.ts"

const migrator = new Migrator({ db, provider: migrationProvider })
const { error } = await migrator.migrateToLatest()

if (error) {
  throw error
}

console.log("Database migrated")

export const app = new App().use(staticFiles()).fsRoutes()
const jetstreamService = new JetstreamService({
  wantedDids: await getAllDids(),
  cursor: await getJetstreamCursor(),
})

jetstreamService.start()

Deno.addSignalListener("SIGTERM", async () => {
  console.log("SIGTERM received, shutting down...")

  await jetstreamService.close()

  console.log("Shutdown complete")
})
