use tracing::info;

use crate::app::routes::create_routes;
use crate::context::Environment;

pub async fn start_server(env: Environment) {
    let app = create_routes();

    let addr = format!("0.0.0.0:{}", env.config.server.port);
    // Address to run our server on
    info!(
        "Listening on {addr}, environment: {}",
        env.config.environment_name
    );
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // Run the server
    axum::serve(listener, app).await.unwrap();
    info!("Server stopped");
}
