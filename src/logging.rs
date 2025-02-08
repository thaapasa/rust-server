use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn configure_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
}
