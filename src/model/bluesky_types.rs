pub fn build_at_proto_uri(user_did: &str, record_key: &str) -> String {
    format!("at://{user_did}/app.bsky.feed.post/{record_key}")
}
