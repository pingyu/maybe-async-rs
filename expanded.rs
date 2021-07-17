#![feature(prelude_import)]
#![allow(dead_code, unused_variables)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
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
/// InnerClient are used to actually send request,
/// which differ a lot between sync and async.
trait InnerClientAsync {
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn request<'async_trait>(
        method: Method,
        url: Url,
        data: String,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = Response> + ::core::marker::Send + 'async_trait>,
    >;
    #[inline]
    #[must_use]
    #[allow(
        clippy::let_unit_value,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn post<'async_trait>(
        url: Url,
        data: String,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = Response> + ::core::marker::Send + 'async_trait>,
    >
    where
        Self: ::core::marker::Send + 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<Response> {
                return __ret;
            }
            let url = url;
            let data = data;
            let __ret: Response = { Self::request(String::from("post"), url, data).await };
            #[allow(unreachable_code)]
            __ret
        })
    }
    #[inline]
    #[must_use]
    #[allow(
        clippy::let_unit_value,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn delete<'async_trait>(
        url: Url,
        data: String,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = Response> + ::core::marker::Send + 'async_trait>,
    >
    where
        Self: ::core::marker::Send + 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<Response> {
                return __ret;
            }
            let url = url;
            let data = data;
            let __ret: Response = { Self::request(String::from("delete"), url, data).await };
            #[allow(unreachable_code)]
            __ret
        })
    }
}
/// The higher level API for end user.
pub struct ServiceClient;
/// Synchronous  implementation, only compiles when `is_sync` feature is off.
/// Else the compiler will complain that *request is defined multiple times* and
/// blabla.
impl InnerClientSync for ServiceClient {
    fn request(method: Method, url: Url, data: String) -> Response {
        String::from("pretend we have a response")
    }
}
/// Asynchronous implementation, only compiles when `is_sync` feature is off.
impl InnerClientAsync for ServiceClient {
    #[allow(
        clippy::let_unit_value,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn request<'async_trait>(
        method: Method,
        url: Url,
        data: String,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = Response> + ::core::marker::Send + 'async_trait>,
    > {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<Response> {
                return __ret;
            }
            let method = method;
            let url = url;
            let data = data;
            let __ret: Response = { String::from("pretend we have a response") };
            #[allow(unreachable_code)]
            __ret
        })
    }
}
/// Code of upstream API are almost the same for sync and async,
/// except for async/await keyword.
impl ServiceClient {
    fn create_bucket_sync(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket"))
    }
    async fn create_bucket_async(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket")).await
    }
    fn delete_bucket_sync(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket"))
    }
    async fn delete_bucket_async(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket")).await
    }
}
fn run_sync() {
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["sync impl running\n"],
            &match () {
                () => [],
            },
        ));
    };
    let _ = ServiceClient::create_bucket("bucket".to_owned());
}
async fn run_async() {
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["async impl running\n"],
            &match () {
                () => [],
            },
        ));
    };
    let _ = ServiceClient::create_bucket("bucket".to_owned()).await;
}
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async {
            #[cfg(feature = "is_sync")]
            run_sync();
            #[cfg(feature = "is_async")]
            run_async().await;
        })
}
