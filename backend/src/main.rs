use std::net::SocketAddr;

use backend::config::Config;
use backend::error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let _pool = backend::create_db_pool(&config).await?;
    let app = backend::http::build_router();
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
