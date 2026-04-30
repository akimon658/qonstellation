import { type Migration, type MigrationProvider } from "@kysely/kysely"
import { down, up } from "./migrations/001_initial.ts"

const migrations = {
  "001_initial": { up, down },
} satisfies Record<string, Migration>

export const migrationProvider = {
  getMigrations() {
    return Promise.resolve(migrations)
  },
} satisfies MigrationProvider
