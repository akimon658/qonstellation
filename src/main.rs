mod app_config;
mod app_state;
mod database;
mod model;
mod repository;
mod service;
mod web;

use atproto_jetstream::CancellationToken;
use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    signal::ctrl_c,
    sync::mpsc,
    time::{self, Duration},
};
use tracing::{error, info};

use app_config::config::Config;
use app_state::AppState;
use tracing_subscriber::EnvFilter;

use crate::repository::user;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting Qonstellation...");

    let config = Config::from_env()?;
    let pool = database::create_pool(
        &config.db_host,
        config.db_port,
        &config.db_user,
        &config.db_password,
        &config.db_name,
    )
    .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let http_client = Client::new();

    let state = Arc::new(AppState {
        pool: pool.clone(),
        config: config.clone(),
        http_client: http_client.clone(),
    });

    let app = web::create_router(state.clone());

    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    let worker =
        service::event_queue_worker::Worker::new(pool.clone(), config.clone(), http_client.clone());
    let worker_handle = tokio::spawn({
        let worker = worker.clone();
        async move {
            worker.run().await;
        }
    });

    let dids = user::get_all_dids(&pool).await?;
    let cursor = repository::system_state::get_jetstream_cursor(&pool).await?;

    let jetstream_cancel = CancellationToken::new();
    let jetstream_cancel_clone = jetstream_cancel.clone();

    let jetstream_pool = pool.clone();
    let jetstream_config = config.clone();
    let jetstream_notify = worker.notify_handle();
    let jetstream_handle = tokio::spawn(async move {
        if let Err(e) = service::jetstream::start(
            model::jetstream::JETSTREAM_ENDPOINTS,
            &dids,
            cursor,
            &jetstream_pool,
            &jetstream_config,
            jetstream_cancel_clone,
            jetstream_notify,
        )
        .await
        {
            error!("Jetstream error: {}", e);
        }
    });

    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        let _ = shutdown_tx.send(()).await;
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown_rx.recv().await;
            info!("Received shutdown signal, shutting down...");
        })
        .await?;

    info!("Shutting down workers...");
    worker.shutdown().await;
    jetstream_cancel.cancel();

    let _ = time::timeout(Duration::from_secs(10), worker_handle).await;
    let _ = time::timeout(Duration::from_secs(10), jetstream_handle).await;

    info!("Shutdown complete.");

    Ok(())
}

async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to listen for SIGTERM: {}", e);
                // Fall back to SIGINT-only.
                let _ = ctrl_c().await;
                return;
            }
        };

        tokio::select! {
            _ = ctrl_c() => {
                info!("Received SIGINT, shutting down...");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down...");
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = ctrl_c().await;
    }
}
