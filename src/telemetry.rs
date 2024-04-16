use tracing::{subscriber::SetGlobalDefaultError, Level, Subscriber};

use crate::config::{Configuration, Env};

fn init_local() -> impl Subscriber {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(Level::DEBUG)
        .finish()
}
fn init_dev() -> impl Subscriber {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(Level::DEBUG)
        .json()
        .finish()
}
fn init_prod() -> impl Subscriber {
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(Level::INFO)
        .json()
        .finish()
}
pub fn set_subscriber(config: &Configuration) -> Result<(), SetGlobalDefaultError> {
    match config.env {
        Env::Local => tracing::subscriber::set_global_default(init_local()),
        Env::Prod => tracing::subscriber::set_global_default(init_prod()),
        Env::Dev => tracing::subscriber::set_global_default(init_dev()),
    }
}
