//!
//! # Maybe-Async Procedure Macro
//!
//! **Why bother writing similar code twice for blocking and async code?**
//!
//! [![Build Status](https://github.com/fMeow/maybe-async-rs/workflows/CI%20%28Linux%29/badge.svg?branch=master)](https://github.com/fMeow/maybe-async-rs/actions)
//! [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
//! [![Latest Version](https://img.shields.io/crates/v/maybe-async.svg)](https://crates.io/crates/maybe-async)
//! [![maybe-async](https://docs.rs/maybe-async/badge.svg)](https://docs.rs/maybe-async)
//!
//! When implementing both sync and async versions of API in a crate, most API
//! of the two version are almost the same except for some async/await keyword.
//!
//! `maybe-async` help unifying async and sync implementation by **procedural
//! macro**.
//! - Write async code with normal `async`, `await`, and let `maybe_async`
//!   handles
//! those `async` and `await` when you need a blocking code.
//! - Switch between sync and async by toggling `is_sync` feature gate in
//!   `Cargo.toml`.
//! - use `must_be_async` and `must_be_sync` to keep code in specified version
//! - use `impl_async` and `impl_sync` to only compile code block on specified
//!   version
//! - A handy macro to unify unit test code is also provided.
//!
//! These procedural macros can be applied to the following codes:
//! - trait item declaration
//! - trait implmentation
//! - function definition
//! - struct definition
//!
//! **RECOMMENDATION**: Enable **resolver ver2** in your crate, which is
//! introduced in Rust 1.51. If not, two crates in dependency with conflict
//! version (one async and another blocking) can fail complilation.
//!
//!
//! ## Motivation
//!
//! The async/await language feature alters the async world of rust.
//! Comparing with the map/and_then style, now the async code really resembles
//! sync version code.
//!
//! In many crates, the async and sync version of crates shares the same API,
//! but the minor difference that all async code must be awaited prevent the
//! unification of async and sync code. In other words, we are forced to write
//! an async and an sync implementation repectively.
//!
//! ## Macros in Detail
//!
//! `maybe-async` offers 4 set of attribute macros: `maybe_async`,
//! `sync_impl`/`async_impl`, `must_be_sync`/`must_be_async`,  and `test`.
//!
//! To use `maybe-async`, we must know which block of codes is only used on
//! blocking implementation, and which on async. These two implementation should
//! share the same function signatures except for async/await keywords, and use
//! `sync_impl` and `async_impl` to mark these implementation.
//!
//! Use `maybe_async` macro on codes that share the same API on both async and
//! blocking code except for async/await keywords. And use feature gate
//! `is_sync` in `Cargo.toml` to toggle between async and blocking code.
//!
//! - `maybe_async`
//!
//!     Offers a unified feature gate to provide sync and async conversion on
//!     demand by feature gate `is_sync`, with **async first** policy.
//!
//!     Want to keep async code? add `maybe_async` in dependencies with default
//!     features, which means `maybe_async` is the same as `must_be_async`:
//!
//!     ```toml
//!     [dependencies]
//!     maybe_async = "0.2"
//!     ```
//!
//!     Wanna convert async code to sync? Add `maybe_async` to dependencies with
//!     an `is_sync` feature gate. In this way, `maybe_async` is the same as
//!     `must_be_sync`:
//!
//!     ```toml
//!     [dependencies]
//!     maybe_async = { version = "0.2", features = ["is_sync"] }
//!     ```
//!
//!     Not all async traits need futures that are `dyn Future + Send`.
//!     To avoid having "Send" and "Sync" bounds placed on the async trait
//!     methods, invoke the maybe_async macro as #[maybe_async(?Send)] on both
//!     the trait and the impl blocks.
//!
//!
//! - `must_be_async`
//!
//!     **Keep async**. Add `async_trait` attribute macro for trait declaration
//!     or implementation to bring async fn support in traits.
//!
//!     To avoid having "Send" and "Sync" bounds placed on the async trait
//!     methods, invoke the maybe_async macro as #[must_be_async(?Send)].
//!
//! - `must_be_sync`
//!
//!     **Convert to sync code**. Convert the async code into sync code by
//!     removing all `async move`, `async` and `await` keyword
//!
//!
//! - `sync_impl`
//!
//!     An sync implementation should on compile on blocking implementation and
//! must     simply disappear when we want async version.
//!
//!     Although most of the API are almost the same, there definitely come to a
//!     point when the async and sync version should differ greatly. For
//!     example, a MongoDB client may use the same API for async and sync
//!     verison, but the code to actually send reqeust are quite different.
//!
//!     Here, we can use `sync_impl` to mark a synchronous implementation, and a
//!     sync implementation shoule disappear when we want async version.
//!
//! - `async_impl`
//!
//!     An async implementation should on compile on async implementation and
//! must     simply disappear when we want sync version.
//!
//!     To avoid having "Send" and "Sync" bounds placed on the async trait
//!     methods, invoke the maybe_async macro as #[async_impl(?Send)].
//!
//!
//! - `test`
//!
//!     Handy macro to unify async and sync **unit and e2e test** code.
//!
//!     You can specify the condition to compile to sync test code
//!     and also the conditions to compile to async test code with given test
//!     macro, e.x. `tokio::test`, `async_std::test` and etc. When only sync
//!     condition is specified,the test code only compiles when sync condition
//!     is met.
//!
//!     ```rust
//!     # #[maybe_async::maybe_async]
//!     # async fn async_fn() -> bool {
//!     #    true
//!     # }
//!
//!     #[maybe_async::test(
//!         feature="is_sync",
//!         async(all(not(feature="is_sync"), feature="async_std"), async_std::test),
//!         async(all(not(feature="is_sync"), feature="tokio"), tokio::test)
//!     )]
//!     async fn test_async_fn() {
//!         let res = async_fn().await;
//!         assert_eq!(res, true);
//!     }
//!     ```
//!
//! ## What's Under the Hook
//!
//! `maybe-async` compiles your code in different way with the `is_sync` feature
//! gate. It remove all `await` and `async` keywords in your code under
//! `maybe_async` macro and conditionally compiles codes under `async_impl` and
//! `sync_impl`.
//!
//! Here is an detailed example on what's going on whe the `is_sync` feature
//! gate set or not.
//!
//! ```rust
//! #[maybe_async::maybe_async(?Send)]
//! trait A {
//!     async fn async_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//!     fn sync_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//! }
//!
//! struct Foo;
//!
//! #[maybe_async::maybe_async(?Send)]
//! impl A for Foo {
//!     async fn async_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//!     fn sync_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//! }
//!
//! #[maybe_async::maybe_async]
//! async fn maybe_async_fn() -> Result<(), ()> {
//!     let a = Foo::async_fn_name().await?;
//!
//!     let b = Foo::sync_fn_name()?;
//!     Ok(())
//! }
//! ```
//!
//! When `maybe-async` feature gate `is_sync` is **NOT** set, the generated code
//! is async code:
//!
//! ```rust
//! // Compiled code when `is_sync` is toggled off.
//! #[async_trait::async_trait(?Send)]
//! trait A {
//!     async fn maybe_async_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//!     fn sync_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//! }
//!
//! struct Foo;
//!
//! #[async_trait::async_trait(?Send)]
//! impl A for Foo {
//!     async fn maybe_async_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//!     fn sync_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//! }
//!
//! async fn maybe_async_fn() -> Result<(), ()> {
//!     let a = Foo::maybe_async_fn_name().await?;
//!     let b = Foo::sync_fn_name()?;
//!     Ok(())
//! }
//! ```
//!
//! When `maybe-async` feature gate `is_sync` is set, all async keyword is
//! ignored and yields a sync version code:
//!
//! ```rust
//! // Compiled code when `is_sync` is toggled on.
//! trait A {
//!     fn maybe_async_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//!     fn sync_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//! }
//!
//! struct Foo;
//!
//! impl A for Foo {
//!     fn maybe_async_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//!     fn sync_fn_name() -> Result<(), ()> {
//!         Ok(())
//!     }
//! }
//!
//! fn maybe_async_fn() -> Result<(), ()> {
//!     let a = Foo::maybe_async_fn_name()?;
//!     let b = Foo::sync_fn_name()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Examples
//!
//! ### rust client for services
//!
//! When implementing rust client for any services, like awz3. The higher level
//! API of async and sync version is almost the same, such as creating or
//! deleting a bucket, retrieving an object and etc.
//!
//! The example `service_client` is a proof of concept that `maybe_async` can
//! actually free us from writing almost the same code for sync and async. We
//! can toggle between a sync AWZ3 client and async one by `is_sync` feature
//! gate when we add `maybe-async` to dependency.
//!
//!
//! # License
//! MIT

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, AttributeArgs, Ident, ImplItem, ItemImpl,
    Lit, Meta, NestedMeta, Path, TraitItem, Type, TypePath,
};

use crate::{
    parse::Item,
    visit::{AsyncAwaitRemoval, AsyncIdentAdder},
};
use quote::quote;

mod parse;
mod visit;

fn ident_add_suffix(ident: &Ident, suffix: &str) -> Ident {
    // TODO: not sure if ident.span() is the way to go
    Ident::new(&format!("{}{}", ident, suffix), ident.span())
}

fn ident_try_remove_suffix(ident: &Ident, suffix: &str) -> Option<Ident> {
    let ident_str = ident.to_string();
    ident_str
        .ends_with(suffix)
        .then(|| Ident::new(&ident_str[..ident_str.len() - suffix.len()], ident.span()))
}

// Appends a suffix to the last segment in the impl's path
#[allow(dead_code)]
fn impl_add_suffix(input: &mut ItemImpl, suffix: &str) {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = &mut *input.self_ty
    {
        if let Some(last) = segments.last_mut() {
            last.ident = ident_add_suffix(&last.ident, suffix);
        }
    }

    // TODO: Only `impl X` blocks are supported for now, not with traits
    // if let Some((_, Path { segments, .. }, _)) = &mut input.trait_ {
    //     if let Some(last) = segments.last_mut() {
    //         last.ident = ident_add_suffix(&last.ident, suffix);
    //     }
    // }
}

fn convert_async(mut input: Item, send: bool, recursion: bool) -> TokenStream2 {
    let prefix = match (send, &input) {
        (true, Item::Impl(_) | Item::Trait(_)) => quote!(#[async_trait::async_trait]),
        (false, Item::Impl(_) | Item::Trait(_)) => quote!(#[async_trait::async_trait(?Send)]),
        _ => quote!(),
    };

    let prefix_recursion = if recursion {
        quote!(#[async_recursion::async_recursion])
    } else {
        quote!()
    };

    match &mut input {
        Item::Impl(item) => {
            for inner in &mut item.items {
                if let ImplItem::Method(ref mut method) = inner {
                    if let Some(pos) = method
                        .attrs
                        .iter()
                        .position(|attr| attr.path.is_ident("maybe_async"))
                    {
                        method.attrs.remove(pos);
                        method.sig.ident = ident_add_suffix(&method.sig.ident, "_async");
                        let expanded = AsyncIdentAdder.add_async_ident(quote!(#method));
                        *method = parse_quote! { #prefix_recursion #expanded };
                    }
                }
            }

            if item.trait_.is_none() {
                quote!(#item)
            } else {
                quote!(#prefix #item)
            }
        }
        Item::Struct(item) => {
            quote!(#item)
        }
        Item::Enum(item) => {
            quote!(#item)
        }
        Item::Trait(item) => {
            quote!(#prefix #item)
        }
        Item::Fn(item) => {
            item.sig.ident = ident_add_suffix(&item.sig.ident, "_async");
            AsyncIdentAdder.add_async_ident(quote!(#prefix_recursion #item))
        }
    }
    .into()
}

fn convert_sync(mut input: Item) -> TokenStream2 {
    match &mut input {
        Item::Impl(item) => {
            for inner in &mut item.items {
                if let ImplItem::Method(ref mut method) = inner {
                    if let Some(pos) = method
                        .attrs
                        .iter()
                        .position(|attr| attr.path.is_ident("maybe_async"))
                    {
                        method.attrs.remove(pos);

                        if let Some(new_ident) =
                            ident_try_remove_suffix(&method.sig.ident, "_async")
                        {
                            method.sig.ident = new_ident;
                        }
                        if method.sig.asyncness.is_some() {
                            method.sig.asyncness = None;
                        }
                        let expanded = AsyncAwaitRemoval.remove_async_await(quote!(#method));
                        *method = parse_quote! { #expanded };
                    }
                }
            }
            AsyncAwaitRemoval.remove_async_await(quote!(#item))
        }
        Item::Struct(item) => {
            item.ident = ident_add_suffix(&item.ident, "Sync");
            quote!(#item)
        }
        Item::Enum(item) => {
            item.ident = ident_add_suffix(&item.ident, "Sync");
            quote!(#item)
        }
        Item::Trait(item) => {
            item.ident = ident_add_suffix(&item.ident, "Sync");
            for inner in &mut item.items {
                if let TraitItem::Method(ref mut method) = inner {
                    if method.sig.asyncness.is_some() {
                        method.sig.asyncness = None;
                    }
                }
            }
            AsyncAwaitRemoval.remove_async_await(quote!(#item))
        }
        Item::Fn(item) => {
            if let Some(new_ident) = ident_try_remove_suffix(&item.sig.ident, "_async") {
                item.sig.ident = new_ident;
            }
            if item.sig.asyncness.is_some() {
                item.sig.asyncness = None;
            }
            AsyncAwaitRemoval.remove_async_await(quote!(#item))
        }
    }
    .into()
}

fn convert_trait(mut input: Item, send: bool) -> TokenStream2 {
    let prefix = match (send, &input) {
        (true, Item::Impl(_) | Item::Trait(_)) => quote!(#[async_trait::async_trait]),
        (false, Item::Impl(_) | Item::Trait(_)) => quote!(#[async_trait::async_trait(?Send)]),
        _ => quote!(),
    };

    match &mut input {
        Item::Impl(item) => {
            let mut expanded_items = Vec::with_capacity(item.items.len());
            for inner in item.items.drain(..) {
                if let ImplItem::Method(mut method) = inner {
                    if let Some(pos) = method.attrs.iter().position(|attr| {
                        attr.path.is_ident("maybe_async")
                            || attr.path.is_ident("maybe_async_recursion")
                    }) {
                        let is_recursion = method
                            .attrs
                            .remove(pos)
                            .path
                            .is_ident("maybe_async_recursion");
                        let prefix_recursion = if is_recursion {
                            quote!(#[async_recursion::async_recursion])
                        } else {
                            quote!()
                        };

                        if cfg!(feature = "is_async") {
                            let mut method = method.clone();
                            method.sig.ident = ident_add_suffix(&method.sig.ident, "_async");
                            let expanded = AsyncIdentAdder.add_async_ident(quote!(#method));
                            let method = parse_quote! { #prefix_recursion #expanded };
                            expanded_items.push(ImplItem::Method(method));
                        }

                        if cfg!(feature = "is_sync") {
                            if let Some(new_ident) =
                                ident_try_remove_suffix(&method.sig.ident, "_async")
                            {
                                method.sig.ident = new_ident;
                            }
                            if method.sig.asyncness.is_some() {
                                method.sig.asyncness = None;
                            }
                            let expanded = AsyncAwaitRemoval.remove_async_await(quote!(#method));
                            let method = parse_quote! { #expanded };
                            expanded_items.push(ImplItem::Method(method));
                        }
                    } else {
                        expanded_items.push(ImplItem::Method(method));
                    }
                } else {
                    expanded_items.push(inner);
                }
            }

            item.items = expanded_items;

            if item.trait_.is_none() {
                quote!(#item)
            } else {
                quote!(#prefix #item)
            }
        }

        Item::Trait(item) => {
            let mut expanded_items = Vec::with_capacity(item.items.len());
            for inner in item.items.drain(..) {
                if let TraitItem::Method(mut method) = inner {
                    if let Some(pos) = method
                        .attrs
                        .iter()
                        .position(|attr| attr.path.is_ident("maybe_async"))
                    {
                        method.attrs.remove(pos);

                        if cfg!(feature = "is_async") {
                            let mut method = method.clone();

                            method.sig.ident = ident_add_suffix(&method.sig.ident, "_async");

                            let is_sync = method.sig.asyncness.is_none();
                            if is_sync {
                                method.sig.asyncness = Some(Default::default());
                            }

                            let method = if method.default.is_some() {
                                let expanded = AsyncIdentAdder.add_async_ident(quote!(#method));
                                parse_quote! { #expanded }
                            } else if is_sync {
                                // TODO: generate default implementation as invoke the sync version.
                                method.default = Some(parse_quote!({
                                    #[allow(clippy::diverging_sub_expression)]
                                    unimplemented!();
                                }));
                                method.attrs.push(parse_quote!(#[allow(unused)]));
                                method
                            } else {
                                method
                            };

                            expanded_items.push(TraitItem::Method(method));
                        }

                        if cfg!(feature = "is_sync") {
                            if let Some(new_ident) =
                                ident_try_remove_suffix(&method.sig.ident, "_async")
                            {
                                method.sig.ident = new_ident;
                            }

                            if method.sig.asyncness.is_some() {
                                method.sig.asyncness = None;
                            }

                            let method = if method.default.is_some() {
                                let expanded =
                                    AsyncAwaitRemoval.remove_async_await(quote!(#method));
                                parse_quote! { #expanded }
                            } else {
                                method
                            };

                            expanded_items.push(TraitItem::Method(method));
                        }
                    } else {
                        expanded_items.push(TraitItem::Method(method));
                    }
                } else {
                    expanded_items.push(inner);
                }
            }

            item.items = expanded_items;

            quote!(#prefix #item)
        }

        _ => syn::Error::new(Span::call_site(), "Only accepts trait or trait impl")
            .to_compile_error()
            .into(),
    }
}

/// `maybe_async::both` attribute macro
///
/// Can be applied to traits, trait impls, structs, struct impls and functions.
#[proc_macro_attribute]
pub fn both(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut send = None;
    let mut recursion = false;
    for arg in args.to_string().replace(" ", "").split(',') {
        match arg {
            "Send" => send = Some(true),
            "?Send" => send = Some(false),
            "Recursion" => recursion = true,
            "" => {}
            _ => {
                return syn::Error::new(
                    Span::call_site(),
                    "Only accepts `Send`, `?Send`, or `Recursion`",
                )
                .to_compile_error()
                .into();
            }
        }
    }
    let send = send.unwrap_or(true);

    let item = parse_macro_input!(input as Item);

    let mut token = TokenStream2::new();

    if cfg!(all(feature = "is_sync", feature = "is_async")) {
        // We need a `clone` if both are enabled
        token.extend(convert_sync(item.clone()));
        token.extend(convert_async(item, send, recursion));
    } else if cfg!(feature = "is_sync") {
        token.extend(convert_sync(item));
    } else if cfg!(feature = "is_async") {
        token.extend(convert_async(item, send, recursion));
    }
    token.into()
}

/// `maybe_async::async_trait` attribute macro
///
/// Can be applied to traits, trait impls.
#[proc_macro_attribute]
pub fn async_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    let send = match args.to_string().replace(" ", "").as_str() {
        "" | "Send" => true,
        "?Send" => false,
        _ => {
            return syn::Error::new(Span::call_site(), "Only accepts `Send` or `?Send`")
                .to_compile_error()
                .into();
        }
    };

    let item = parse_macro_input!(input as Item);
    convert_trait(item, send).into()
}

/// convert marked async code to async code with `async-trait`
#[proc_macro_attribute]
pub fn must_be_async(args: TokenStream, input: TokenStream) -> TokenStream {
    let send = match args.to_string().replace(" ", "").as_str() {
        "" | "Send" => true,
        "?Send" => false,
        _ => {
            return syn::Error::new(Span::call_site(), "Only accepts `Send` or `?Send`")
                .to_compile_error()
                .into();
        }
    };
    let item = parse_macro_input!(input as Item);
    convert_async(item, send, false).into()
}

/// convert marked async code to sync code
#[proc_macro_attribute]
pub fn must_be_sync(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    convert_sync(item).into()
}

/// mark sync implementation
///
/// only compiled when `is_sync` feature gate is set.
/// When `is_sync` is not set, marked code is removed.
#[proc_macro_attribute]
pub fn sync_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let token = if cfg!(feature = "is_sync") {
        let item = parse_macro_input!(input as Item);
        let input = convert_sync(item);
        quote!(#input)
    } else {
        quote!()
    };
    token.into()
}

/// mark async implementation
///
/// only compiled when `is_sync` feature gate is not set.
/// When `is_sync` is set, marked code is removed.
#[proc_macro_attribute]
pub fn async_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let send = match args.to_string().replace(" ", "").as_str() {
        "" | "Send" => true,
        "?Send" => false,
        _ => {
            return syn::Error::new(Span::call_site(), "Only accepts `Send` or `?Send`")
                .to_compile_error()
                .into();
        }
    };

    let token = if cfg!(feature = "is_async") {
        let item = parse_macro_input!(input as Item);
        convert_async(item, send, false)
    } else {
        quote!()
    };
    token.into()
}

macro_rules! match_nested_meta_to_str_lit {
    ($t:expr) => {
        match $t {
            NestedMeta::Lit(lit) => {
                match lit {
                    Lit::Str(s) => {
                        s.value().parse::<TokenStream2>().unwrap()
                    }
                    _ => {
                        return syn::Error::new(lit.span(), "expected meta or string literal").to_compile_error().into();
                    }
                }
            }
            NestedMeta::Meta(meta) => quote!(#meta)
        }
    };
}

/// Handy macro to unify test code of sync and async code
///
/// Since the API of both sync and async code are the same,
/// with only difference that async functions must be awaited.
/// So it's tedious to write unit sync and async respectively.
///
/// This macro helps unify the sync and async unit test code.
/// Pass the condition to treat test code as sync as the first
/// argument. And specify the condition when to treat test code
/// as async and the lib to run async test, e.x. `async-std::test`,
/// `tokio::test`, or any valid attribute macro.
///
/// **ATTENTION**: do not write await inside a assert macro
///
/// - Examples
///
/// ```rust
/// #[maybe_async::maybe_async]
/// async fn async_fn() -> bool {
///     true
/// }
///
/// #[maybe_async::test(
///     // when to treat the test code as sync version
///     feature="is_sync",
///     // when to run async test
///     async(all(not(feature="is_sync"), feature="async_std"), async_std::test),
///     // you can specify multiple conditions for different async runtime
///     async(all(not(feature="is_sync"), feature="tokio"), tokio::test)
/// )]
/// async fn test_async_fn() {
///     let res = async_fn().await;
///     assert_eq!(res, true);
/// }
///
/// // Only run test in sync version
/// #[maybe_async::test(feature = "is_sync")]
/// async fn test_sync_fn() {
///     let res = async_fn().await;
///     assert_eq!(res, true);
/// }
/// ```
///
/// The above code is transcripted to the following code:
///
/// ```rust
/// # use maybe_async::{must_be_async, must_be_sync, sync_impl};
/// # #[maybe_async::maybe_async]
/// # async fn async_fn() -> bool { true }
///
/// // convert to sync version when sync condition is met, keep in async version when corresponding
/// // condition is met
/// #[cfg_attr(feature = "is_sync", must_be_sync, test)]
/// #[cfg_attr(
///     all(not(feature = "is_sync"), feature = "async_std"),
///     must_be_async,
///     async_std::test
/// )]
/// #[cfg_attr(
///     all(not(feature = "is_sync"), feature = "tokio"),
///     must_be_async,
///     tokio::test
/// )]
/// async fn test_async_fn() {
///     let res = async_fn().await;
///     assert_eq!(res, true);
/// }
///
/// // force converted to sync function, and only compile on sync condition
/// #[cfg(feature = "is_sync")]
/// #[test]
/// fn test_sync_fn() {
///     let res = async_fn();
///     assert_eq!(res, true);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let input = TokenStream2::from(input);
    if attr_args.len() < 1 {
        return syn::Error::new(
            Span::call_site(),
            "Arguments cannot be empty, at least specify the condition for sync code",
        )
        .to_compile_error()
        .into();
    }

    // The first attributes indicates sync condition
    let sync_cond = match_nested_meta_to_str_lit!(attr_args.first().unwrap());
    let mut ts = quote!(#[cfg_attr(#sync_cond, maybe_async::must_be_sync, test)]);

    // The rest attributes indicates async condition and async test macro
    // only accepts in the forms of `async(cond, test_macro)`, but `cond` and
    // `test_macro` can be either meta attributes or string literal
    let mut async_token = Vec::new();
    let mut async_conditions = Vec::new();
    for async_meta in attr_args.into_iter().skip(1) {
        match async_meta {
            NestedMeta::Meta(meta) => match meta {
                Meta::List(list) => {
                    let name = list.path.segments[0].ident.to_string();
                    if name.ne("async") {
                        return syn::Error::new(
                            list.path.span(),
                            format!("Unknown path: `{}`, must be `async`", name),
                        )
                        .to_compile_error()
                        .into();
                    }
                    if list.nested.len() == 2 {
                        let async_cond =
                            match_nested_meta_to_str_lit!(list.nested.first().unwrap());
                        let async_test = match_nested_meta_to_str_lit!(list.nested.last().unwrap());
                        let attr = quote!(
                            #[cfg_attr(#async_cond, maybe_async::must_be_async, #async_test)]
                        );
                        async_conditions.push(async_cond);
                        async_token.push(attr);
                    } else {
                        let msg = format!(
                            "Must pass two metas or string literals like `async(condition, \
                             async_test_macro)`, you passed {} metas.",
                            list.nested.len()
                        );
                        return syn::Error::new(list.span(), msg).to_compile_error().into();
                    }
                }
                _ => {
                    return syn::Error::new(
                        meta.span(),
                        "Must be list of metas like: `async(condition, async_test_macro)`",
                    )
                    .to_compile_error()
                    .into();
                }
            },
            NestedMeta::Lit(lit) => {
                return syn::Error::new(
                    lit.span(),
                    "Must be list of metas like: `async(condition, async_test_macro)`",
                )
                .to_compile_error()
                .into();
            }
        };
    }

    async_token.into_iter().for_each(|t| ts.extend(t));
    ts.extend(quote!( #input ));
    if !async_conditions.is_empty() {
        quote! {
            #[cfg(any(#sync_cond, #(#async_conditions),*))]
            #ts
        }
    } else {
        quote! {
            #[cfg(#sync_cond)]
            #ts
        }
    }
    .into()
}
