import { type Migration, type MigrationProvider } from "@kysely/kysely"
import migration1 from "./migrations/001_initial.ts"
import migration2 from "./migrations/002_refresh_token.ts"
import migration3 from "./migrations/003_unique_posts.ts"
import migration4 from "./migrations/004_bigint.ts"
import migration5 from "./migrations/005_queued_events.ts"

const migrations = {
  "001_initial": migration1,
  "002_refresh_token": migration2,
  "003_unique_posts": migration3,
  "004_bigint": migration4,
  "005_queued_events": migration5,
} satisfies Record<string, Migration>

export const migrationProvider = {
  getMigrations() {
    return Promise.resolve(migrations)
  },
} satisfies MigrationProvider
