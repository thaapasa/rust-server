use crate::app::routes::create_routes;
use crate::context::Environment;
use axum::extract::{MatchedPath, Request};
use axum::http::{header, StatusCode};
use http::Response;

use axum::Extension;
use bytes::Bytes;
use http_body_util::Full;
use std::any::Any;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn start_server(env: Environment) {
    let app = create_routes()
        .layer(
            TraceLayer::new_for_http()
                // Create our own span for the request and include the matched path. The matched
                // path is useful for figuring out which handler the request was routed to.
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    // axum automatically adds this extension.
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                // By default, `TraceLayer` will log 5xx responses, but we're doing our specific
                // logging of errors so disable that
                .on_failure(()),
        )
        .layer(Extension(env.clone()))
        .layer(CatchPanicLayer::custom(handle_panic));

    let addr = format!("0.0.0.0:{}", env.config.server.port);
    // Address to run our server on
    info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    // Run the server
    axum::serve(listener, app).await.unwrap();
    info!("Server stopped");
}

fn handle_panic(_: Box<dyn Any + Send + 'static>) -> Response<Full<Bytes>> {
    let body = serde_json::json!({
        "error": {
            "kind": "panic",
        }
    });
    let body = serde_json::to_string(&body).unwrap();

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::from(body))
        .unwrap()
}
