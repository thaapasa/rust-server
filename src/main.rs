use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::start_server;

mod app;

#[tokio::main]
async fn main() {
    configure_logging();
    start_server().await;
}

fn configure_logging() {
    // Set up a tracing subscriber (logs). This is optional, but helpful.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
}