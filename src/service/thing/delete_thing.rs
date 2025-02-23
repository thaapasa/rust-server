use crate::context::{Context, Transactional};
use crate::error::InternalError;
use macros::sql;
use uuid::Uuid;

pub async fn delete_thing(
    ctx: &mut (impl Context + Transactional),
    thing_id: Uuid,
) -> Result<(), InternalError> {
    ctx.db()
        .execute(sql!("DELETE FROM things WHERE id={thing_id}"))
        .await
}
