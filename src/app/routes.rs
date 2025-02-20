use axum::routing::{get, post};
use axum::Router;

use crate::app::routes::get_root::get_root_route_handler;
use crate::app::routes::thing::{get_thing_handler, post_thing_handler};

mod get_root;
mod thing;

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(get_root_route_handler))
        .route("/things", post(post_thing_handler))
        .route("/things/{thing_id}", post(get_thing_handler))
}
