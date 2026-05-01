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

export const getUserAccessToken = async (userId: string) => {
  const result = await db
    .selectFrom("userTokens")
    .select("accessToken")
    .where("userId", "=", userId)
    .executeTakeFirstOrThrow()

  return result.accessToken
}
