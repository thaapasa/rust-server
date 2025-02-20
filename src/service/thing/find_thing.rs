use crate::context::Context;
use crate::db::{Database, DbThing};
use crate::error::InternalError;
use sqlx::QueryBuilder;
use uuid::Uuid;

pub async fn find_thing(
    ctx: &mut Context,
    thing_id: Uuid,
) -> Result<Option<DbThing>, InternalError> {
    ctx.db()
        .fetch_optional(
            QueryBuilder::new("SELECT * FROM things WHERE id = ")
                .push_bind(thing_id)
                .build(),
        )
        .await
}
