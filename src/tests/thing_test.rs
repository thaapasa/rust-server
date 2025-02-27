use crate::context::{Context, Transactional};
use crate::service::{add_new_thing, find_thing, ThingData};
use crate::tests::TestEnvironment;
use tokio::test;

#[test]
pub async fn test_create_thing() {
    let env = TestEnvironment::init().await;
    let mut ctx = env.ctx().await;
    let mut tx = ctx.begin().await.unwrap();
    let thing = add_new_thing(
        &mut tx,
        ThingData {
            name: "thingy".to_string(),
            description: Some("This is the real deal".to_string()),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let loaded = find_thing(&mut ctx, thing.id).await.unwrap().unwrap();
    assert_eq!(loaded, thing);
    assert_eq!(thing.name, "thingy".to_string());
    assert_eq!(thing.description, Some("This is the real deal".to_string()));
}
