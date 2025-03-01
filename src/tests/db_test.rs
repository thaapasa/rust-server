use std::collections::BTreeSet;

use tokio::test;

use sql::sql;

use crate::context::{Context, Transactional};
use crate::db::{DatabaseAccess, DatabaseAccessExt};
use crate::set;
use crate::tests::TestEnvironment;

#[test]
pub async fn test_transaction_is_rolled_back_on_drop() {
    let env = init_fixtures().await;
    let mut ctx = env.ctx().await;
    assert_eq!(foo_count(&mut ctx).await, 0);

    let mut tx = ctx.begin().await.unwrap();
    add_value(&mut tx, 1).await;
    assert_eq!(foo_count(&mut tx).await, 1);
    assert_eq!(foo_values(&mut tx).await, set![1]);
    assert_eq!(foo_count(&mut env.ctx().await).await, 0);
    drop(tx);

    for _ in 1..10 {
        assert_eq!(foo_count(&mut ctx).await, 0);
    }
}

#[test]
pub async fn test_nested_transaction_is_rolled_back_on_drop_but_main_is_committed() {
    let env = init_fixtures().await;
    let mut ctx = env.ctx().await;
    assert_eq!(foo_count(&mut ctx).await, 0);

    let mut tx = ctx.begin().await.unwrap();
    add_value(&mut tx, 1).await;

    let mut nested = tx.begin().await.unwrap();
    add_value(&mut nested, 2).await;
    assert_eq!(foo_values(&mut nested).await, set![1, 2]);
    drop(nested);

    assert_eq!(foo_values(&mut tx).await, set![1]);
    add_value(&mut tx, 3).await;
    assert_eq!(foo_values(&mut tx).await, set![1, 3]);
    tx.commit().await.unwrap();

    assert_eq!(foo_count(&mut ctx).await, 2);
    assert_eq!(foo_values(&mut ctx).await, set![1, 3]);
}

#[test]
pub async fn test_nested_transaction_is_rolled_back_but_main_is_committed() {
    let env = init_fixtures().await;
    let mut ctx = env.ctx().await;
    assert_eq!(foo_count(&mut ctx).await, 0);

    let mut tx = ctx.begin().await.unwrap();
    add_value(&mut tx, 1).await;

    let mut nested = tx.begin().await.unwrap();
    add_value(&mut nested, 2).await;
    assert_eq!(foo_values(&mut nested).await, set![1, 2]);
    nested.rollback().await.unwrap();

    assert_eq!(foo_values(&mut tx).await, set![1]);
    add_value(&mut tx, 3).await;
    assert_eq!(foo_values(&mut tx).await, set![1, 3]);
    tx.commit().await.unwrap();

    assert_eq!(foo_count(&mut ctx).await, 2);
    assert_eq!(foo_values(&mut ctx).await, set![1, 3]);
}

async fn init_fixtures() -> TestEnvironment {
    let env = TestEnvironment::init().await;
    env.ctx()
        .await
        .db()
        .execute(sql!("CREATE TABLE foo (value INTEGER)"))
        .await
        .unwrap();
    env
}

async fn add_value(ctx: &mut impl Context, value: i32) {
    ctx.db()
        .execute(sql!(
            // language=postgresql
            "INSERT INTO foo (value) VALUES (${value})"
        ))
        .await
        .unwrap();
}

async fn foo_count(ctx: &mut impl Context) -> i64 {
    ctx.db()
        .fetch_one::<(i64,)>(sql!(
            // language=postgresql
            "SELECT COUNT(*) AS count FROM foo"
        ))
        .await
        .unwrap()
        .0
}

async fn foo_values(ctx: &mut impl Context) -> BTreeSet<i32> {
    ctx.db()
        .fetch_all::<(i32,)>(sql!(
            // language=postgresql
            "SELECT value FROM foo"
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.0)
        .collect()
}
