use axum::Json;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct RootResponse {
    status: String,
}

pub async fn get_root_route_handler() -> Json<RootResponse> {
    Json(RootResponse {
        status: "ok".to_string(),
    })
}
