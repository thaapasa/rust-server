use axum::Json;

use crate::app::api_error::ApiError;
use crate::app::extractors::RequestContext;
use crate::app::models::{ApiThing, ApiThingData};
use crate::context::{Context, Transactional};
use crate::service::add_new_thing;

pub async fn post_thing_handler(
    RequestContext(mut ctx): RequestContext,
    Json(thing_data): Json<ApiThingData>,
) -> Result<Json<ApiThing>, ApiError> {
    let mut tx_ctx = ctx.begin().await?;
    let thing = add_new_thing(&mut tx_ctx, thing_data.clone().into()).await?;

    let mut double_add_tx = tx_ctx.begin().await?;
    add_new_thing(&mut double_add_tx, thing_data.clone().into()).await?;
    let mut triple_add_tx = double_add_tx.begin().await?;
    add_new_thing(&mut triple_add_tx, thing_data.clone().into()).await?;
    triple_add_tx.commit().await?;
    add_new_thing(&mut double_add_tx, thing_data.clone().into()).await?;
    double_add_tx.rollback().await?;
    tx_ctx.commit().await?;

    Ok(Json(thing.into()))
}
