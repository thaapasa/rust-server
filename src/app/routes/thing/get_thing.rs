use crate::app::api_error::ApiError;
use crate::app::extractors::{InputPath, RequestContext};
use crate::app::models::ApiThing;
use crate::service::find_thing;
use axum::Json;
use uuid::Uuid;

pub async fn get_thing_handler(
    RequestContext(mut ctx): RequestContext,
    InputPath(thing_id): InputPath<Uuid>,
) -> Result<Json<ApiThing>, ApiError> {
    if let Some(thing) = find_thing(&mut ctx, thing_id).await? {
        Ok(Json(thing.into()))
    } else {
        Err(ApiError::not_found())
    }
}
