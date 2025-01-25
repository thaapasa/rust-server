use axum::Router;
use axum::routing::get;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // Set up a tracing subscriber (logs). This is optional, but helpful.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build our application by creating a Router. 
    // Here, we define one route at the root "/" to respond with "Hello, World!".
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let addr = "0.0.0.0:3000";
    // Address to run our server on
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // Run the server
    axum::serve(listener, app).await.unwrap();
}
