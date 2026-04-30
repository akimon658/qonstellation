import { Migrator } from "@kysely/kysely"
import { App, staticFiles } from "fresh"
import { db } from "./database/db.ts"
import { migrationProvider } from "./database/migrator.ts"

const migrator = new Migrator({ db, provider: migrationProvider })
const { error } = await migrator.migrateToLatest()

if (error) {
  throw error
}

console.log("Database migrated")

export const app = new App().use(staticFiles()).fsRoutes()
