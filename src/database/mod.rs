use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};

pub type DbPool = MySqlPool;

pub async fn create_pool(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    database: &str,
) -> anyhow::Result<MySqlPool> {
    let options = MySqlConnectOptions::new()
        .host(host)
        .port(port)
        .username(user)
        .password(password)
        .database(database);

    let pool = MySqlPoolOptions::new().connect_with(options).await?;

    Ok(pool)
}
