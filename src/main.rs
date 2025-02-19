use tracing::{error, info};

use crate::app::start_server;
use crate::context::Environment;
use crate::db::run_db_migrations;
use crate::error::InternalError;
use crate::logging::configure_logging;

mod app;
mod context;
mod db;
mod error;
mod logging;
mod service;

#[tokio::main]
async fn main() {
    configure_logging();
    match start_main().await {
        Ok(_) => (),
        Err(e) => {
            error!("Error starting server: {e}")
        }
    }
}

async fn start_main() -> Result<(), InternalError> {
    let env = Environment::init().await?;
    info!("Initialized environment {}", env.config.environment_name);
    run_db_migrations(&env).await?;
    start_server(env).await;
    Ok(())
}
