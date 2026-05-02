const qonstellationJwtSecret = Deno.env.get("QONSTELLATION_JWT_SECRET")

if (!qonstellationJwtSecret) {
  throw new Error("QONSTELLATION_JWT_SECRET is not set")
}

const traqClientId = Deno.env.get("TRAQ_CLIENT_ID")

if (!traqClientId) {
  throw new Error("TRAQ_CLIENT_ID is not set")
}

const traqClientSecret = Deno.env.get("TRAQ_CLIENT_SECRET")

if (!traqClientSecret) {
  throw new Error("TRAQ_CLIENT_SECRET is not set")
}

export const config = {
  qonstellationJwtSecret,
  dbHost: Deno.env.get("DB_HOST") ?? "localhost",
  dbPort: Number(Deno.env.get("DB_PORT") ?? 3306),
  dbUser: Deno.env.get("DB_USER") ?? "root",
  dbPassword: Deno.env.get("DB_PASSWORD") ?? "password",
  dbName: Deno.env.get("DB_NAME") ?? "qonstellation",
  traqBaseUrl: Deno.env.get("TRAQ_BASE_URL") ?? "http://localhost:3000",
  traqClientId,
  traqClientSecret,
} as const
