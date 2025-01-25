use axum::Router;
use axum::routing::get;

pub async fn start_server() {
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
