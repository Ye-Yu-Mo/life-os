use std::net::SocketAddr;
use std::sync::Arc;

use backend::config::Config;
use backend::error::AppError;
use backend::repository::raw_logs::PgRawLogRepository;
use backend::service::raw_logs::RawLogService;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let pool = backend::create_db_pool(&config).await?;
    backend::run_migrations(&pool).await?;

    let raw_log_repository = Arc::new(PgRawLogRepository::new(pool));
    let raw_log_service = Arc::new(RawLogService::new(raw_log_repository));
    let app = backend::http::build_router(raw_log_service);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>().map_err(|error| {
            AppError::Config(format!("invalid APP_HOST: {error}"))
        })?,
        config.server.port,
    )))
    .await
    .map_err(|_| AppError::Internal)?;

    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        "backend listening"
    );

    axum::serve(listener, app)
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(())
}
