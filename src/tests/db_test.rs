use crate::context::Context;
use crate::db::Database;
use crate::tests::TestEnvironment;
use serde::Deserialize;
use sql::sql;
use sqlx::FromRow;
use tokio::test;
use tracing::info;

#[test]
pub async fn test_transactions() {
    let env = TestEnvironment::init().await;
    info!("Starting test");
    env.ctx()
        .await
        .db()
        .execute("CREATE TABLE foo (id INTEGER)")
        .await
        .unwrap();
    let mut ctx = env.ctx().await;
    assert_eq!(foo_count(ctx.db()).await, 0);

    let mut tx = ctx.begin().await.unwrap();
    tx.db()
        .execute("INSERT INTO foo (id) VALUES (1)")
        .await
        .unwrap();
    assert_eq!(foo_count(env.ctx().await.db()).await, 0);
    drop(tx);

    assert_eq!(foo_count(ctx.db()).await, 0);
}

#[derive(Deserialize, FromRow)]
pub struct Count {
    count: i64,
}

async fn foo_count(db: &mut Database) -> i64 {
    db.fetch_one::<Count>(sql!("SELECT COUNT(*) AS count FROM foo"))
        .await
        .unwrap()
        .count
}
