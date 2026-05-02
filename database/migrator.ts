import { type Migration, type MigrationProvider } from "@kysely/kysely"
import migration1 from "./migrations/001_initial.ts"
import migration2 from "./migrations/002_refresh_token.ts"
import migration3 from "./migrations/003_unique_posts.ts"

const migrations = {
  "001_initial": migration1,
  "002_refresh_token": migration2,
  "003_unique_posts": migration3,
} satisfies Record<string, Migration>

export const migrationProvider = {
  getMigrations() {
    return Promise.resolve(migrations)
  },
} satisfies MigrationProvider
