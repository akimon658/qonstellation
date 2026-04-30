import { UserSettingsTable } from "../database/userSettings.ts"

export const getAllDids = (): Promise<string[]> => {
  // TODO
  return Promise.resolve([])
}

export const getUserSettingByDid = (
  did: string,
): Promise<UserSettingsTable> => {
  // TODO
  return Promise.resolve({
    did,
    targetChannelId: "",
    userId: "",
  })
}

export const getUserAccessToken = (userId: string) => {
  // TODO
  return Promise.resolve("")
}
