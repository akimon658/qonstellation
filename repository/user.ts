import { db } from "../database/db.ts"

export const getAllDids = async () => {
  const result = await db.selectFrom("user_settings").select("did").execute()

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
    .selectFrom("user_settings")
    .selectAll()
    .where("did", "=", did)
    .executeTakeFirstOrThrow()

  return {
    userId: result.user_id,
    did: result.did,
    targetChannelId: result.target_channel_id,
  }
}

export const getUserSettingByUserId = async (userId: string) => {
  return await db
    .selectFrom("user_settings")
    .select(["did", "target_channel_id"])
    .where("user_id", "=", userId)
    .executeTakeFirst()
}

export const saveUserSettings = async (
  userId: string,
  did: string,
  targetChannelId: string,
) => {
  await db
    .insertInto("user_settings")
    .values({ user_id: userId, did, target_channel_id: targetChannelId })
    .onDuplicateKeyUpdate({ did, target_channel_id: targetChannelId })
    .execute()
}

export const getUserAccessToken = async (userId: string) => {
  const result = await db
    .selectFrom("user_tokens")
    .select("access_token")
    .where("user_id", "=", userId)
    .executeTakeFirstOrThrow()

  return result.access_token
}

export const saveUser = async (userId: string) => {
  await db
    .insertInto("users")
    .values({ id: userId })
    .onDuplicateKeyUpdate({ id: userId })
    .execute()
}

export const saveUserTokens = async (
  data: {
    userId: string
    accessToken: string
    refreshToken: string
  },
) => {
  await db
    .insertInto("user_tokens")
    .values({
      user_id: data.userId,
      access_token: data.accessToken,
      refresh_token: data.refreshToken,
    })
    .onDuplicateKeyUpdate({
      access_token: data.accessToken,
      refresh_token: data.refreshToken,
    })
    .execute()
}
