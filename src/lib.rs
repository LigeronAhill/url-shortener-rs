pub mod config;
mod error;
pub use error::{AppError, Result};
pub mod gen_alias;
pub mod server;
pub mod storage;
pub mod telemetry;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{debug, error, info};
pub async fn run() -> Result<()> {
    // init config
    let conf = config::init("config/local.toml");

    // init logger
    telemetry::set_subscriber(&conf)?;
    info!("Starting url-shortener with '{:?}' env", conf.env);
    debug!("Debug messages are enabled");
    error!("Error messages are enabled");

    // init storage: sqlx, sqlite
    info!("Initializing DB");
    let storage = storage::sqlite::init(&conf).await?;

    // init router: axum
    info!("Initializing server");
    let app = server::app(&conf, storage);
    let listener = TcpListener::bind(&conf.http_server.address).await?;

    // run server
    info!("Starting server at '{}'", conf.http_server.address);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
pub async fn test_run(listener: TcpListener) -> Result<()> {
    let conf = config::Configuration::default();
    let storage = storage::sqlite::init(&conf).await?;
    let app = server::app(&conf, storage);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
