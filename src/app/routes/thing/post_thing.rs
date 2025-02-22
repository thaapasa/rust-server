use crate::app::api_error::ApiError;
use crate::app::models::{ApiThing, ApiThingData};
use crate::context::{Context, SystemContext, Transactional};
use crate::service::add_new_thing;
use axum::Json;

pub async fn post_thing_handler(
    SystemContext(mut ctx): SystemContext,
    Json(thing_data): Json<ApiThingData>,
) -> Result<Json<ApiThing>, ApiError> {
    let mut tx_ctx = ctx.begin().await?;
    let thing = add_new_thing(&mut tx_ctx, thing_data.clone().into()).await?;

    let mut double_add_tx = tx_ctx.begin().await?;
    add_new_thing(&mut double_add_tx, thing_data.into()).await?;
    double_add_tx.rollback().await?;

    tx_ctx.commit().await?;
    Ok(Json(thing.into()))
}
