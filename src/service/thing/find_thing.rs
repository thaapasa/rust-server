use crate::context::Context;
use crate::db::DbThing;
use crate::error::InternalError;
use sql::sql;
use uuid::Uuid;

pub async fn find_thing(
    ctx: &mut impl Context,
    thing_id: Uuid,
) -> Result<Option<DbThing>, InternalError> {
    ctx.db()
        .fetch_optional(sql!(
            // language=postgresql
            "SELECT * FROM things WHERE id = ${thing_id}"
        ))
        .await
}
