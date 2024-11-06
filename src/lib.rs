//!
//! # Maybe-Async Procedure Macro
//!
//! THANKS to the great projects of <https://github.com/fMeow/maybe-async-rs> and <https://github.com/marioortizmanero/maybe-async-rs>.
//!
//! **Why bother writing similar code twice for blocking and async code?**
//!
//! [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
//!
//! When implementing both sync and async versions of API in a crate, most API
//! of the two version are almost the same except for some async/await keyword.
//!
//! `maybe-async` help unifying async and sync implementation by **procedural
//! macro**.
//! - Write async code with normal `async`, `await`, and let `maybe_async`
//!   handles those `async` and `await` when you need a blocking code.
//! - Switch between sync and async by toggling `is_sync` feature gate in
//!   `Cargo.toml`.
//!
//! # License
//! MIT

extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, AttributeArgs, Ident, ImplItem, Lit, Meta,
    NestedMeta, TraitItem,
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

/// Generate Async Codes.
///
/// ## Example
///
/// ```rust
/// struct Foo {}
///
/// impl Foo {
///     #[maybe_async::both]
///     async fn fn1(&self) -> bool {
///         self.fn2().await;
///         false
///     }
///     #[maybe_async::both]
///     async fn fn2(&self) {}
/// }
///
/// #[maybe_async::both(Recursion)]
/// pub async fn fn3() {
///     let foo = Foo {};
///     let ok = foo.fn1().await;
///     //...
///     if ok {
///         fn3().await;
///     }
///     //...
/// }
/// ```
///
/// Will generate:
///
/// ```rust
/// struct Foo {}
///
/// impl Foo {
///     async fn fn1_async(&self) -> bool {
///         self.fn2_async().await;
///         false
///     }
///     async fn fn2_async(&self) {}
/// }
///
/// #[async_recursion::async_recursion]
/// pub async fn fn3_async() {
///     let foo = Foo {};
///     let ok = foo.fn1_async().await;
///     //...
///     if ok {
///         fn3_async().await;
///     }
///     //...
/// }
/// ```
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
            // Currently useless.
            // Use `#[maybe_async::async_trait]` on traits impls instead.
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
}

/// Generate Sync Codes.
///
/// ## Example
///
/// ```rust
/// struct Foo {}
///
/// impl Foo {
///     #[maybe_async::both]
///     async fn fn1(&self) -> bool {
///         self.fn2().await;
///         false
///     }
///     #[maybe_async::both]
///     async fn fn2(&self) {}
/// }
///
/// #[maybe_async::both(Recursion)]
/// pub async fn fn3() {
///     let foo = Foo {};
///     let ok = foo.fn1().await;
///     //...
///     if ok {
///         fn3().await;
///     }
///     //...
/// }
/// ```
///
/// Will generate:
///
/// ```rust
/// struct Foo {}
///
/// impl Foo {
///     fn fn1(&self) -> bool {
///         self.fn2();
///         false
///     }
///     fn fn2(&self) {}
/// }
///
/// pub fn fn3() {
///     let foo = Foo {};
///     let ok = foo.fn1();
///     //...
///     if ok {
///         fn3();
///     }
///     //...
/// }
/// ```
fn convert_sync(mut input: Item) -> TokenStream2 {
    match &mut input {
        Item::Impl(item) => {
            // Currently useless.
            // Use `#[maybe_async::async_trait]` on traits impls instead.
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
            quote!(#item)
        }
        Item::Enum(item) => {
            quote!(#item)
        }
        Item::Trait(item) => {
            // Currently useless.
            // Use `#[maybe_async::async_trait]` on traits instead.
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
}

/// Generate Sync & Async Codes for trait and trait impls.
///
/// ## Example
///
/// ```rust
/// #[maybe_async::async_trait]
/// trait A {
///     #[maybe_async]
///     async fn fn1(&self);
///     #[maybe_async]
///     async fn fn2(&self);
/// }
///
/// struct Foo {}
///
/// #[maybe_async::async_trait]
/// impl A for Foo {
///     #[maybe_async]
///     async fn fn1(&self) {
///         self.fn2().await;
///     }
///
///     #[maybe_async]
///     async fn fn2(&self) {}
/// }
/// ```
///
/// Will generate:
///
/// ```rust
/// #[async_trait::async_trait]
/// trait A {
///     fn fn1(&self);
///     async fn fn1_async(&self);
///     fn fn2(&self);
///     async fn fn2_async(&self);
/// }
///
/// struct Foo {}
///
/// #[async_trait::async_trait]
/// impl A for Foo {
///     fn fn1(&self) {
///         self.fn2();
///     }
///     async fn fn1_async(&self) {
///         self.fn2_async().await;
///     }
///
///     fn fn2(&self) {}
///     async fn fn2_async(&self) {}
/// }
/// ```
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
                    if let Some(pos) = method
                        .attrs
                        .iter()
                        .position(|attr| attr.path.is_ident("maybe_async"))
                    {
                        method.attrs.remove(pos);

                        if cfg!(feature = "is_async") {
                            let mut method = method.clone();
                            method.sig.ident = ident_add_suffix(&method.sig.ident, "_async");
                            let expanded = AsyncIdentAdder.add_async_ident(quote!(#method));
                            let method = parse_quote! { #expanded };
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
                                    unimplemented!();
                                }));
                                method.attrs.push(parse_quote!(#[allow(unused)]));
                                method
                                    .attrs
                                    .push(parse_quote!(#[allow(clippy::diverging_sub_expression)]));
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
            .to_compile_error(),
    }
}

/// `maybe_async::both` attribute macro
///
/// Can be applied to functions and items in impls.
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

/// Convert marked *async* codes to async.
///
/// Currently only used for testing.
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

/// Convert marked *async* codes to sync.
///
/// Currently only used for testing.
#[proc_macro_attribute]
pub fn must_be_sync(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    convert_sync(item).into()
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
/// #[maybe_async::both]
/// async fn async_fn() -> bool {
///     true
/// }
///
/// #[maybe_async::test(
///     // when to treat the test code as sync version
///     feature="is_sync",
///     // when to run async test
///     async(all(feature="is_async", feature="async_std"), async_std::test),
///     // you can specify multiple conditions for different async runtime
///     async(all(feature="is_async", feature="tokio"), tokio::test)
/// )]
/// async fn test_async_fn() {
///     let res = async_fn_async().await;
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
/// # use maybe_async::{must_be_async, must_be_sync};
/// # #[maybe_async::both]
/// # async fn async_fn() -> bool { true }
///
/// // convert to sync version when sync condition is met, keep in async version when corresponding
/// // condition is met
/// #[cfg_attr(feature = "is_sync", must_be_sync, test)]
/// #[cfg_attr(
///     all(feature = "is_async", feature = "async_std"),
///     must_be_async,
///     async_std::test
/// )]
/// #[cfg_attr(
///     all(feature = "is_async", feature = "tokio"),
///     must_be_async,
///     tokio::test
/// )]
/// async fn test_async_fn() {
///     let res = async_fn_async().await;
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
    if attr_args.is_empty() {
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
