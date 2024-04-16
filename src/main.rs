use tracing::{debug, info};
use url_shortener_rs::{config, sqlite, telemetry, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // init config
    let conf = config::init("config/local.toml");

    // init logger
    telemetry::set_subscriber(&conf)?;
    info!("Starting url-shortener with '{:?}' env", conf.env);
    debug!("Debug messages are enabled");

    // init storage: sqlx, sqlite
    info!("Initializing DB");
    let storage = sqlite::init(&conf).await?;
    let g = storage.get_url("googled").await?;
    info!("Got '{g}' for 'google' alias");
    // TODO: init router: axum
    info!("Initializing server");

    // TODO: run server
    Ok(())
}
