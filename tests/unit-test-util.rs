#[maybe_async::both]
async fn some_function() -> bool {
    true
}

#[maybe_async::test]
async fn test_fn() {
    let res = some_function().await;
    assert_eq!(res, true);
}
