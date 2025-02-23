use crate::app::api_error::ApiError;
use crate::app::extractors::InputPath;
use crate::context::{Context, SystemContext, Transactional};
use crate::service::delete_thing;
use axum::body::Body;
use http::Response;
use uuid::Uuid;

pub async fn delete_thing_handler(
    SystemContext(mut ctx): SystemContext,
    InputPath(thing_id): InputPath<Uuid>,
) -> Result<Response<Body>, ApiError> {
    let mut tx_ctx = ctx.begin().await?;
    delete_thing(&mut tx_ctx, thing_id).await?;
    tx_ctx.commit().await?;
    Ok(Response::new(Body::empty()))
}
