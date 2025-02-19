#![cfg(all(feature = "is_sync", feature = "is_async"))]

#[maybe_async::both]
async fn outer_fn() {
    print("outer_fn").await;

    #[maybe_async]
    async fn nested_fn() {
        print("nested_fn").await;
    }

    nested_fn().await;
}

fn print(tag: &str) {
    println!("{}: I'm sync", tag);
}

async fn print_async(tag: &str) {
    println!("{}: I'm async", tag);
}

#[tokio::test]
async fn test_nested_fn() {
    outer_fn();
    outer_fn_async().await;
}
