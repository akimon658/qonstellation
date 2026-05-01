import { db } from "../database/db.ts"

export const getTraqMessageIdByAtProtoUri = async (
  atProtoUri: string,
): Promise<string | undefined> => {
  const result = await db.selectFrom("posts")
    .select("traq_message_id")
    .where("at_proto_uri", "=", atProtoUri)
    .executeTakeFirst()

  return result?.traq_message_id
}

export const savePostMetadata = async (data: {
  atProtoUri: string
  traqMessageId: string
}): Promise<void> => {
  await db.insertInto("posts").values({
    at_proto_uri: data.atProtoUri,
    traq_message_id: data.traqMessageId,
  }).execute()
}
