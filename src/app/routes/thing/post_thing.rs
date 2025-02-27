use crate::app::api_error::ApiError;
use crate::app::extractors::SystemContext;
use crate::app::models::{ApiThing, ApiThingData};
use crate::context::{Context, Transactional};
use crate::service::add_new_thing;
use axum::Json;

pub async fn post_thing_handler(
    SystemContext(mut ctx): SystemContext,
    Json(thing_data): Json<ApiThingData>,
) -> Result<Json<ApiThing>, ApiError> {
    let mut tx_ctx = ctx.begin().await?;
    let thing = add_new_thing(&mut tx_ctx, thing_data.clone().into()).await?;

    let mut double_add_tx = tx_ctx.begin().await?;
    add_new_thing(&mut double_add_tx, thing_data.clone().into()).await?;
    let mut triple_add_tx = double_add_tx.begin().await?;
    add_new_thing(&mut triple_add_tx, thing_data.into()).await?;
    triple_add_tx.commit().await?;
    double_add_tx.rollback().await?;

    tx_ctx.commit().await?;
    Ok(Json(thing.into()))
}
