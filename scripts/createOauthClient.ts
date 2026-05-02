import { client } from "../traq/client.gen.ts"
import { createClient, login } from "../traq/index.ts"

client.setConfig({
  baseUrl: "http://localhost:3000/api/v3",
})

const loginRes = await login({
  body: {
    name: "traq",
    password: "traq",
  },
})

if (!loginRes.response?.ok) {
  console.log(loginRes.response)

  throw new Error("Failed to login")
}

const clientRes = await createClient({
  body: {
    name: "Qonstellation",
    description: "Test Client",
    callbackUrl: "http://localhost:5173/callback",
    scopes: ["read", "write"],
  },
  headers: {
    Cookie: loginRes.response?.headers.get("set-cookie"),
  },
})

if (!clientRes.data) {
  throw new Error(`Failed to create client: ${clientRes.error}`)
}

console.log(`TRAQ_CLIENT_ID=${clientRes.data.id}`)
console.log(`TRAQ_CLIENT_SECRET=${clientRes.data.secret}`)
