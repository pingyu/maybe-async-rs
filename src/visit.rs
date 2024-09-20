use std::iter::FromIterator;

use crate::{ident_add_suffix, ident_try_remove_suffix};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    visit_mut::{self, visit_item_mut, visit_path_segment_mut, VisitMut},
    Expr, ExprBlock, File, GenericArgument, GenericParam, Item, PathArguments, PathSegment, Type,
    TypeParamBound, WherePredicate,
};

pub struct ReplaceGenericType<'a> {
    generic_type: &'a str,
    arg_type: &'a PathSegment,
}

impl<'a> ReplaceGenericType<'a> {
    pub fn new(generic_type: &'a str, arg_type: &'a PathSegment) -> Self {
        Self {
            generic_type,
            arg_type,
        }
    }

    pub fn replace_generic_type(item: &mut Item, generic_type: &'a str, arg_type: &'a PathSegment) {
        let mut s = Self::new(generic_type, arg_type);
        s.visit_item_mut(item);
    }
}

impl<'a> VisitMut for ReplaceGenericType<'a> {
    fn visit_item_mut(&mut self, i: &mut Item) {
        if let Item::Fn(item_fn) = i {
            // remove generic type from generics <T, F>
            let args = item_fn
                .sig
                .generics
                .params
                .iter()
                .filter_map(|param| {
                    if let GenericParam::Type(type_param) = &param {
                        if type_param.ident.to_string().eq(self.generic_type) {
                            None
                        } else {
                            Some(param)
                        }
                    } else {
                        Some(param)
                    }
                })
                .collect::<Vec<_>>();
            item_fn.sig.generics.params =
                Punctuated::from_iter(args.into_iter().map(|p| p.clone()).collect::<Vec<_>>());

            // remove generic type from where clause
            if let Some(where_clause) = &mut item_fn.sig.generics.where_clause {
                let new_where_clause = where_clause
                    .predicates
                    .iter()
                    .filter_map(|predicate| {
                        if let WherePredicate::Type(predicate_type) = predicate {
                            if let Type::Path(p) = &predicate_type.bounded_ty {
                                if p.path.segments[0].ident.to_string().eq(self.generic_type) {
                                    None
                                } else {
                                    Some(predicate)
                                }
                            } else {
                                Some(predicate)
                            }
                        } else {
                            Some(predicate)
                        }
                    })
                    .collect::<Vec<_>>();

                where_clause.predicates = Punctuated::from_iter(
                    new_where_clause
                        .into_iter()
                        .map(|c| c.clone())
                        .collect::<Vec<_>>(),
                );
            };
        }
        visit_item_mut(self, i)
    }
    fn visit_path_segment_mut(&mut self, i: &mut PathSegment) {
        // replace generic type with target type
        if i.ident.to_string().eq(&self.generic_type) {
            *i = self.arg_type.clone();
        }
        visit_path_segment_mut(self, i);
    }
}

pub struct AsyncAwaitRemoval;

impl AsyncAwaitRemoval {
    pub fn remove_async_await(&mut self, item: TokenStream) -> TokenStream {
        let mut syntax_tree: File = syn::parse(item.into()).unwrap();
        self.visit_file_mut(&mut syntax_tree);
        quote!(#syntax_tree)
    }
}

impl VisitMut for AsyncAwaitRemoval {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_mut(self, node);

        match node {
            Expr::Await(expr) => *node = (*expr.base).clone(),

            Expr::Async(expr) => {
                let inner = &expr.block;
                let sync_expr = if inner.stmts.len() == 1 {
                    // remove useless braces when there is only one statement
                    let stmt = &inner.stmts.get(0).unwrap();
                    // convert statement to Expr
                    parse_quote!(#stmt)
                } else {
                    Expr::Block(ExprBlock {
                        attrs: expr.attrs.clone(),
                        block: inner.clone(),
                        label: None,
                    })
                };
                *node = sync_expr;
            }

            Expr::MethodCall(expr) => {
                // TODO: Remove suffix n async & await context only.
                if let Some(new_ident) = ident_try_remove_suffix(&expr.method, "_async") {
                    expr.method = new_ident;
                }
            }

            _ => {}
        }
    }

    fn visit_item_mut(&mut self, i: &mut Item) {
        // find generic parameter of Future and replace it with its Output type
        if let Item::Fn(item_fn) = i {
            let mut inputs: Vec<(String, PathSegment)> = vec![];

            // generic params: <T:Future<Output=()>, F>
            for param in &item_fn.sig.generics.params {
                // generic param: T:Future<Output=()>
                if let GenericParam::Type(type_param) = param {
                    let generic_type_name = type_param.ident.to_string();

                    // bound: Future<Output=()>
                    for bound in &type_param.bounds {
                        inputs.extend(search_trait_bound(&generic_type_name, bound));
                    }
                }
            }

            if let Some(where_clause) = &item_fn.sig.generics.where_clause {
                for predicate in &where_clause.predicates {
                    if let WherePredicate::Type(predicate_type) = predicate {
                        let generic_type_name = if let Type::Path(p) = &predicate_type.bounded_ty {
                            p.path.segments[0].ident.to_string()
                        } else {
                            panic!("Please submit an issue");
                        };

                        for bound in &predicate_type.bounds {
                            inputs.extend(search_trait_bound(&generic_type_name, bound));
                        }
                    }
                }
            }

            for (generic_type_name, path_seg) in &inputs {
                ReplaceGenericType::replace_generic_type(i, generic_type_name, path_seg);
            }
        }
        visit_item_mut(self, i);
    }
}

fn search_trait_bound(
    generic_type_name: &str,
    bound: &TypeParamBound,
) -> Vec<(String, PathSegment)> {
    let mut inputs = vec![];

    if let TypeParamBound::Trait(trait_bound) = bound {
        let segment = &trait_bound.path.segments[trait_bound.path.segments.len() - 1];
        let name = segment.ident.to_string();
        if name.eq("Future") {
            // match Future<Output=Type>
            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                // binding: Output=Type
                if let GenericArgument::Binding(binding) = &args.args[0] {
                    if let Type::Path(p) = &binding.ty {
                        inputs.push((generic_type_name.to_owned(), p.path.segments[0].clone()));
                    }
                }
            }
        }
    }
    inputs
}

pub struct AsyncIdentAdder;

impl AsyncIdentAdder {
    pub fn add_async_ident(&mut self, item: TokenStream) -> TokenStream {
        let mut syntax_tree: File = syn::parse(item.into()).unwrap();
        self.visit_file_mut(&mut syntax_tree);
        quote!(#syntax_tree)
    }
}

impl VisitMut for AsyncIdentAdder {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_mut(self, node);

        match node {
            Expr::Await(expr) => match expr.base.as_ref() {
                Expr::MethodCall(base_expr) => {
                    if !base_expr.method.to_string().ends_with("_async") {
                        let mut base_expr = base_expr.clone();
                        base_expr.method = ident_add_suffix(&base_expr.method, "_async");
                        expr.base = Box::new(Expr::MethodCall(base_expr));
                    }
                }

                Expr::Call(call_expr) => {
                    if let Expr::Path(path_expr) = &*call_expr.func {
                        let last_seg = path_expr.path.segments.last().unwrap();
                        if !last_seg.ident.to_string().ends_with("_async") {
                            let mut call_expr = call_expr.clone();
                            let mut func_expr = call_expr.func.as_ref().clone();
                            let Expr::Path(path_expr) = &mut func_expr else {
                                unreachable!()
                            };
                            path_expr.path.segments.last_mut().unwrap().ident =
                                ident_add_suffix(&last_seg.ident, "_async");
                            call_expr.func = Box::new(func_expr);
                            expr.base = Box::new(Expr::Call(call_expr));
                        }
                    }
                }

                _ => {}
            },

            _ => {}
        }
    }
}
