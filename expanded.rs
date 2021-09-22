#![feature(prelude_import)]
#![allow(dead_code, unused_variables)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
type Response = String;
type Url = &'static str;
type Method = String;
/// To use `maybe-async`, we must know which block of codes is only used on
/// blocking implementation, and which on async. These two implementation should
/// share the same API except for async/await keywords.
///
/// This will generate two traits: `InnerClientSync` and `InnerClientAsync`
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
/// To use `maybe-async`, we must know which block of codes is only used on
/// blocking implementation, and which on async. These two implementation should
/// share the same API except for async/await keywords.
///
/// This will generate two traits: `InnerClientSync` and `InnerClientAsync`
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
/// This will generate a `ServiceClientSync`, which will implement
/// `InnerClientSync`, and a `ServiceClientAsync`, which will implement
/// `InnerClientAsync`.
///
/// If we had a single `ServiceClient` which implemented both `InnerClientSync`
/// and `InnerClientAsync`, calls to methods like `request` would be ambiguous
/// when both async and sync were enabled.
pub struct ServiceClientSync;
/// This will generate a `ServiceClientSync`, which will implement
/// `InnerClientSync`, and a `ServiceClientAsync`, which will implement
/// `InnerClientAsync`.
///
/// If we had a single `ServiceClient` which implemented both `InnerClientSync`
/// and `InnerClientAsync`, calls to methods like `request` would be ambiguous
/// when both async and sync were enabled.
pub struct ServiceClientAsync;
/// Synchronous  implementation.
#[cfg(feature = "is_sync")]
impl InnerClientSync for ServiceClientSync {
    fn request(method: Method, url: Url, data: String) -> Response {
        String::from("pretend we have a response")
    }
}
/// Asynchronous implementation only.
#[cfg(feature = "is_async")]
impl InnerClientAsync for ServiceClientAsync {
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
/// Code of upstream API are almost the same for sync and async, except for
/// async/await keyword. This will generate the same `impl` but for both
/// `ServiceClientAsync` and `ServiceClientSync`.
impl ServiceClientSync {
    fn create_bucket(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket"))
    }
    fn delete_bucket(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket"))
    }
}
/// Code of upstream API are almost the same for sync and async, except for
/// async/await keyword. This will generate the same `impl` but for both
/// `ServiceClientAsync` and `ServiceClientSync`.
impl ServiceClientAsync {
    async fn create_bucket(name: String) -> Response {
        Self::post("http://correct_url4create", String::from("my_bucket")).await
    }
    async fn delete_bucket(name: String) -> Response {
        Self::delete("http://correct_url4delete", String::from("my_bucket")).await
    }
}
#[cfg(feature = "is_sync")]
fn run_sync() {
    {
        ::std::io::_print(
            match match (&ServiceClientSync::get_name(),) {
                (arg0,) => [::core::fmt::ArgumentV1::new(
                    arg0,
                    ::core::fmt::Display::fmt,
                )],
            } {
                ref args => unsafe {
                    ::core::fmt::Arguments::new_v1(&["", ": sync impl running\n"], args)
                },
            },
        );
    };
    let _ = ServiceClientSync::create_bucket("bucket".to_owned());
}
#[cfg(feature = "is_async")]
async fn run_async() {
    {
        ::std::io::_print(
            match match (&ServiceClientAsync::get_name(),) {
                (arg0,) => [::core::fmt::ArgumentV1::new(
                    arg0,
                    ::core::fmt::Display::fmt,
                )],
            } {
                ref args => unsafe {
                    ::core::fmt::Arguments::new_v1(&["", ": async impl running\n"], args)
                },
            },
        );
    };
    let _ = ServiceClientAsync::create_bucket("bucket".to_owned()).await;
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
