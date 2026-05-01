import { db } from "../database/db.ts"

export const getAllDids = async () => {
  const result = await db.selectFrom("userSettings").select("did").execute()

  return result.map((row) => row.did)
}

interface UserSetting {
  userId: string
  did: string
  targetChannelId: string
}

export const getUserSettingByDid = async (
  did: string,
): Promise<UserSetting> => {
  const result = await db
    .selectFrom("userSettings")
    .selectAll()
    .where("did", "=", did)
    .executeTakeFirstOrThrow()

  return result
}

export const getUserSettingByUserId = async (userId: string) => {
  return await db
    .selectFrom("userSettings")
    .select(["did", "targetChannelId"])
    .where("userId", "=", userId)
    .executeTakeFirst()
}

export const saveUserSettings = async (
  userId: string,
  did: string,
  targetChannelId: string,
) => {
  await db
    .insertInto("userSettings")
    .values({ userId, did, targetChannelId })
    .onDuplicateKeyUpdate({ did, targetChannelId })
    .execute()
}

export const getUserAccessToken = async (userId: string) => {
  const result = await db
    .selectFrom("userTokens")
    .select("accessToken")
    .where("userId", "=", userId)
    .executeTakeFirstOrThrow()

  return result.accessToken
}

export const saveUser = async (userId: string) => {
  await db
    .insertInto("users")
    .values({ id: userId })
    .onDuplicateKeyUpdate({ id: userId })
    .execute()
}

export const saveUserTokens = async (
  userId: string,
  accessToken: string,
  refreshToken: string,
) => {
  await db
    .insertInto("userTokens")
    .values({ userId, accessToken, refreshToken })
    .onDuplicateKeyUpdate({ accessToken, refreshToken })
    .execute()
}
