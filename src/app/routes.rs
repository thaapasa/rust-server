use axum::Router;
use axum::routing::get;

use crate::app::routes::get_root::get_root_route_handler;

mod get_root;

pub fn create_routes() -> Router {
    Router::new().route("/", get(get_root_route_handler))
}
