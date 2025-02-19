use crate::context::Context;
use crate::db::{Database, DbThing};
use crate::error::InternalError;
use sqlx::QueryBuilder;

pub struct ThingData {
    pub name: String,
    pub description: Option<String>,
}

pub async fn add_new_thing(ctx: &mut Context, thing: ThingData) -> Result<DbThing, InternalError> {
    let thing = ctx
        .db()
        .fetch_one(
            QueryBuilder::new("INSERT INTO things (name, description) VALUES (")
                .push_bind(thing.name)
                .push(", ")
                .push_bind(thing.description)
                .push(") RETURNING *")
                .build(),
        )
        .await?;
    Ok(thing)
}
