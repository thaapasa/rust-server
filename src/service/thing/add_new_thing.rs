use sqlx::FromRow;
use uuid::Uuid;

use sql::sql;

use crate::context::{Context, Transactional};
use crate::db::{DatabaseAccessExt, DbThing};
use crate::error::InternalError;
use crate::service::find_thing;

pub struct ThingData {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct InsertResult {
    id: Uuid,
}

pub async fn add_new_thing(
    ctx: &mut (impl Context + Transactional),
    thing: ThingData,
) -> Result<DbThing, InternalError> {
    let res = ctx
        .db()
        .fetch_one::<InsertResult>(sql!(
            // language=postgresql
            "INSERT INTO things (name, description)
             VALUES (${name}, ${description})
             RETURNING id",
            name = thing.name,
            description = thing.description
        ))
        .await?;
    let thing = find_thing(ctx, res.id).await?;
    thing.ok_or(InternalError::message(
        "Inserted thing not found from db".to_string(),
    ))
}
