use tracing::error;

use crate::app::start_server;
use crate::context::Environment;
use crate::error::InternalError;
use crate::logging::configure_logging;

mod app;
mod context;
mod error;
mod logging;

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
    let env = Environment::init()?;
    start_server(env).await;
    Ok(())
}
