import { Kysely, MysqlDialect } from "@kysely/kysely"
import { createPool } from "mysql2"
import { config } from "../lib/config.ts"
import type { PostsTable } from "./posts.ts"
import type { SystemStatesTable } from "./systemStates.ts"
import type { UsersTable } from "./users.ts"
import type { UserSettingsTable } from "./userSettings.ts"
import type { UserTokensTable } from "./userTokens.ts"

export interface Database {
  posts: PostsTable
  users: UsersTable
  system_states: SystemStatesTable
  user_settings: UserSettingsTable
  user_tokens: UserTokensTable
}

const dialect = new MysqlDialect({
  pool: createPool({
    host: config.dbHost,
    port: config.dbPort,
    user: config.dbUser,
    password: config.dbPassword,
    database: config.dbName,
  }),
})

export const db = new Kysely<Database>({ dialect })
