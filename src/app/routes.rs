use axum::routing::{get, post};
use axum::Router;

use crate::app::routes::get_root::get_root_route_handler;
use crate::app::routes::post_thing::post_thing_handler;

mod get_root;
mod post_thing;

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(get_root_route_handler))
        .route("/things", post(post_thing_handler))
}
