//! Expanded version of `service_client.rs` for both sync and async features.

#![allow(dead_code, unused_variables)]
/// To use `maybe-async`, we must know which block of codes is only used on
/// blocking implementation, and which on async. These two implementation should
/// share the same API except for async/await keywords, and use `sync_impl` and
/// `async_impl` to mark these implementation.
type Response = String;
type Url = &'static str;
type Method = String;

/// InnerClient are used to actually send request,
/// which differ a lot between sync and async.
trait InnerClientSync {
    fn request(method: Method, url: Url, data: String) -> Response;
    #[inline]
    fn post(url: Url, data: String) -> Response {
        Self::request(String::from("post"), url, data)
    }
    #[inline]
    fn delete(url: Url, data: String) -> Response {
        Self::request(String::from("delete"), url, data)
    }
}
#[async_trait::async_trait]
trait InnerClientAsync {
    async fn request(method: Method, url: Url, data: String) -> Response;
    #[inline]
    async fn post(url: Url, data: String) -> Response {
        Self::request(String::from("post"), url, data).await
    }
    #[inline]
    async fn delete(url: Url, data: String) -> Response {
        Self::request(String::from("delete"), url, data).await
    }
}

/// The higher level API for end user.
pub struct ServiceClient;

/// Synchronous  implementation, only compiles when `is_sync` feature is off.
/// Else the compiler will complain that *request is defined multiple times* and
/// blabla.
impl InnerClientSync for ServiceClient {
    fn request(method: Method, url: Url, data: String) -> Response {
        // your implementation for sync, like use
        // `reqwest::blocking` to send request
        String::from("pretend we have a response")
    }
}

/// Asynchronous implementation, only compiles when `is_sync` feature is off.
#[async_trait::async_trait]
impl InnerClientAsync for ServiceClient {
    async fn request(method: Method, url: Url, data: String) -> Response {
        // your implementation for async, like use `reqwest::client`
        // or `async_std` to send request
        String::from("pretend we have a response")
    }
}

/// Code of upstream API are almost the same for sync and async,
/// except for async/await keyword.
impl ServiceClient {
    fn create_bucket_sync(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket"))
        // When `is_sync` is toggle on, this block will compiles to:
        // Self::post("http://correct_url4create", String::from("my_bucket"))
    }
    async fn create_bucket_async(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket")).await
        // When `is_sync` is toggle on, this block will compiles to:
        // Self::post("http://correct_url4create", String::from("my_bucket"))
    }

    fn delete_bucket_sync(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket"))
    }
    async fn delete_bucket_async(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket")).await
    }
    // and another thousands of functions that interact with service side
}

#[maybe_async::sync_impl]
fn run() {
    println!("sync impl running");
    let _ = ServiceClient::create_bucket_sync("bucket".to_owned());
}

#[maybe_async::async_impl]
async fn run() {
    println!("async impl running");
    let _ = ServiceClient::create_bucket_async("bucket".to_owned()).await;
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "is_sync")]
    run_sync();

    #[cfg(feature = "is_async")]
    run_async().await;
}
