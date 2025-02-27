use crate::app::extractors::SystemContext;
use crate::context::Context;
use axum::Json;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct RootResponse {
    status: String,
    env: String,
}

pub async fn get_root_route_handler(SystemContext(ctx): SystemContext) -> Json<RootResponse> {
    Json(RootResponse {
        status: "ok".to_string(),
        env: ctx.env().config.environment_name.clone(),
    })
}
