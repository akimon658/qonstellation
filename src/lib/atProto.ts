export const buildAtProtoUri = (params: {
  userDid: string
  recordKey: string
}): string => {
  return `at://${params.userDid}/app.bsky.feed.post/${params.recordKey}`
}
