use crate::db::encode_sql;
use macros::{format_uri, sql};
use sqlx::query::Query;
use sqlx::{Execute, Postgres};

#[test]
fn test_uri_macro() {
    let a = "hello/world";
    assert_eq!(
        format_uri!("http://localhost/path/{a}"),
        "http://localhost/path/hello%2Fworld"
    );
}

#[test]
fn test_sql_encode() {
    let a = "sp-1";
    assert_eq!(encode_sql("sp-1"), "sp-1");
}

#[test]
fn test_sql_macro() {
    let sp = "sp-1";
    assert_eq!(
        (sql!("SAVEPOINT {sp:raw}") as Query<Postgres, _>).sql(),
        "SAVEPOINT sp-1"
    );
    let sp_str = "sp-2".to_string();
    assert_eq!(
        (sql!("SAVEPOINT {sp_str:raw}") as Query<Postgres, _>).sql(),
        "SAVEPOINT sp-2"
    );
}
