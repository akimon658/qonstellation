import { Kysely, MysqlDialect } from "@kysely/kysely"
import { createPool } from "mysql2"
import type { PostsTable } from "./posts.ts"
import type { SystemStatesTable } from "./systemStates.ts"
import type { UsersTable } from "./users.ts"
import type { UserSettingsTable } from "./userSettings.ts"
import type { UserTokensTable } from "./userTokens.ts"

export interface Database {
  posts: PostsTable
  users: UsersTable
  systemStates: SystemStatesTable
  userSettings: UserSettingsTable
  userTokens: UserTokensTable
}

const dialect = new MysqlDialect({
  pool: createPool({
    host: Deno.env.get("DB_HOST") ?? "localhost",
    port: Number(Deno.env.get("DB_PORT") ?? 3306),
    user: Deno.env.get("DB_USER") ?? "root",
    password: Deno.env.get("DB_PASSWORD") ?? "password",
    database: Deno.env.get("DB_NAME") ?? "qonstellation",
  }),
})

export const db = new Kysely<Database>({ dialect })
