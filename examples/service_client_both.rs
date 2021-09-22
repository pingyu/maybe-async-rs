//! Manually expanded version of `service_client.rs` for both sync and async
//! features.

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

/// The higher level API for end user, synchronous version.
pub struct ServiceClientSync;

/// The higher level API for end user, asynchronous version.
pub struct ServiceClientAsync;

/// Synchronous  implementation, only compiles when `is_sync` feature is on.
impl InnerClientSync for ServiceClientSync {
    fn request(method: Method, url: Url, data: String) -> Response {
        // your implementation for sync, like use
        // `reqwest::blocking` to send request
        String::from("pretend we have a response")
    }
}

/// Asynchronous implementation, only compiles when `is_async` feature is on.
#[async_trait::async_trait]
impl InnerClientAsync for ServiceClientAsync {
    async fn request(method: Method, url: Url, data: String) -> Response {
        // your implementation for async, like use `reqwest::client`
        // or `async_std` to send request
        String::from("pretend we have a response")
    }
}

/// Code of upstream API are almost the same for sync and async,
/// except for async/await keyword.
impl ServiceClientSync {
    fn create_bucket(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket"))
    }

    fn delete_bucket(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket"))
    }
    // and another thousands of functions that interact with service side
}

impl ServiceClientAsync {
    async fn create_bucket(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket")).await
    }

    async fn delete_bucket(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket")).await
    }
    // and another thousands of functions that interact with service side
}

fn run_sync() {
    println!("sync impl running");
    let _ = ServiceClientSync::create_bucket("bucket".to_owned());
}

async fn run_async() {
    println!("async impl running");
    let _ = ServiceClientAsync::create_bucket("bucket".to_owned()).await;
}

#[tokio::main]
async fn main() {
    run_sync();
    run_async().await;
}
