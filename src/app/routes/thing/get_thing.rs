use crate::app::api_error::ApiError;
use crate::app::models::ApiThing;
use crate::context::SystemContext;
use crate::service::find_thing;
use axum::Json;
use uuid::uuid;

pub async fn get_thing_handler(
    SystemContext(mut ctx): SystemContext,
) -> Result<Json<ApiThing>, ApiError> {
    if let Some(thing) = find_thing(&mut ctx, uuid!("019522f3-6ef5-70ab-aa97-80c668ebe6ed"))
        .await
        .unwrap()
    {
        Ok(Json(thing.into()))
    } else {
        Err(ApiError::not_found())
    }
}
