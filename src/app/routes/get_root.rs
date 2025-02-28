use crate::app::extractors::RequestContext;
use crate::context::Context;
use axum::Json;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct RootResponse {
    status: String,
    env: String,
}

pub async fn get_root_route_handler(RequestContext(ctx): RequestContext) -> Json<RootResponse> {
    Json(RootResponse {
        status: "ok".to_string(),
        env: ctx.env().config.environment_name.clone(),
    })
}
