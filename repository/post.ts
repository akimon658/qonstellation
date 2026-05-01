import { db } from "../database/db.ts"

export const getTraqMessageIdByAtProtoUri = async (
  atProtoUri: string,
): Promise<string | undefined> => {
  const result = await db.selectFrom("posts")
    .select("traqMessageId")
    .where("atProtoUri", "=", atProtoUri)
    .executeTakeFirst()

  return result?.traqMessageId
}

export const savePostMetadata = async (data: {
  atProtoUri: string
  traqMessageId: string
}): Promise<void> => {
  await db.insertInto("posts").values({
    atProtoUri: data.atProtoUri,
    traqMessageId: data.traqMessageId,
  }).execute()
}
