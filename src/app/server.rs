use axum::Router;
use axum::routing::get;
use tracing::info;
use crate::context::Environment;

pub async fn start_server(env: Environment) {
    // Build our application by creating a Router.
    // Here, we define one route at the root "/" to respond with "Hello, World!".
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let addr = format!("0.0.0.0:{}", env.config.server.port);
    // Address to run our server on
    info!("Listening on {addr}, environment: {}", env.config.environment_name);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // Run the server
    axum::serve(listener, app).await.unwrap();
    info!("Server stopped");
}
