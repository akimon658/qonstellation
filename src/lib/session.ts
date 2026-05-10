import { decodeBase64 } from "@std/encoding"
import { getCookies, setCookie } from "@std/http"
import { jwtVerify, SignJWT } from "jose"
import { config } from "./config.ts"

const secret = decodeBase64(config.qonstellationJwtSecret)
const COOKIE_NAME = "qonstellation_session"

export const createSessionToken = (userId: string) =>
  new SignJWT({ userId })
    .setProtectedHeader({ alg: "HS256" })
    .setIssuedAt()
    .setExpirationTime("24h")
    .sign(secret)

export const verifySessionToken = async (
  token: string,
) => {
  try {
    const { payload } = await jwtVerify(token, secret)

    if (typeof payload.userId === "string") {
      return payload.userId
    }
  } catch {
    return undefined
  }
}

export const getSessionToken = (req: Request) =>
  getCookies(req.headers)[COOKIE_NAME]

export const setSessionToken = (headers: Headers, token: string) =>
  setCookie(headers, {
    name: COOKIE_NAME,
    value: token,
    httpOnly: true,
    secure: true,
    sameSite: "Lax",
    path: "/",
  })
