use uuid::Uuid;

use sql::sql;

use crate::context::{Context, Transactional};
use crate::db::DatabaseAccess;
use crate::error::InternalError;

pub async fn delete_thing(
    ctx: &mut (impl Context + Transactional),
    thing_id: Uuid,
) -> Result<(), InternalError> {
    ctx.db()
        .execute(sql!(
            // language=postgresql
            "DELETE FROM things WHERE id=${thing_id}"
        ))
        .await?;
    Ok(())
}
