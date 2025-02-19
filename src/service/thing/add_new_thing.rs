use crate::context::Context;
use crate::db::{Database, DbThing};
use crate::error::InternalError;

pub struct ThingData {
    pub name: String,
    pub description: Option<String>,
}

pub async fn add_new_thing(ctx: &mut Context, thing: ThingData) -> Result<DbThing, InternalError> {
    let thing = ctx
        .db()
        .await?
        .fetch_one(
            "INSERT INTO things (name, description) VALUES ('name', NULL) RETURNING *".to_string(),
        )
        .await?;
    Ok(thing)
}
