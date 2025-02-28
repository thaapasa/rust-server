use crate::app::api_error::ApiError;
use crate::app::extractors::{InputPath, RequestContext};
use crate::context::{Context, Transactional};
use crate::service::delete_thing;
use axum::body::Body;
use http::Response;
use uuid::Uuid;

pub async fn delete_thing_handler(
    RequestContext(mut ctx): RequestContext,
    InputPath(thing_id): InputPath<Uuid>,
) -> Result<Response<Body>, ApiError> {
    let mut tx_ctx = ctx.begin().await?;
    delete_thing(&mut tx_ctx, thing_id).await?;
    tx_ctx.commit().await?;
    Ok(Response::new(Body::empty()))
}
