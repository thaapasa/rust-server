use axum::Router;
use axum::routing::{delete, get, post};

use crate::app::routes::get_root::get_root_route_handler;
use crate::app::routes::thing::{delete_thing_handler, get_thing_handler, post_thing_handler};

mod get_root;
mod thing;

pub fn create_routes() -> Router {
    Router::new()
        .route("/", get(get_root_route_handler))
        .route("/things", post(post_thing_handler))
        .route("/things/{thing_id}", get(get_thing_handler))
        .route("/things/{thing_id}", delete(delete_thing_handler))
}
