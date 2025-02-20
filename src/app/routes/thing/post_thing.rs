use crate::app::models::{ApiThing, ApiThingData};
use crate::context::SystemContext;
use crate::service::add_new_thing;
use axum::Json;

pub async fn post_thing_handler(
    SystemContext(mut ctx): SystemContext,
    Json(thing_data): Json<ApiThingData>,
) -> Json<ApiThing> {
    let thing = add_new_thing(&mut ctx, thing_data.into()).await.unwrap();
    Json(thing.into())
}
