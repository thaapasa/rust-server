use macros::format_uri;

#[test]
fn test_uri_macro() {
    let a = "hello/world";
    assert_eq!(
        format_uri!("http://localhost/path/{a}"),
        "http://localhost/path/hello%2Fworld"
    );
}
