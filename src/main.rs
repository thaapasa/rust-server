use tracing::error;
use crate::context::Environment;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::start_server;
use crate::error::InternalError;

mod app;
mod context;
mod error;

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

fn configure_logging() {
    // Set up a tracing subscriber (logs). This is optional, but helpful.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
}