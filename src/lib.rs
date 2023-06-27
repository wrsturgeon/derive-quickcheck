/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Automatically derive `quickcheck::Arbitrary` with `#[derive(QuickCheck)]`.

#![deny(warnings)]
#![warn(
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    clippy::restriction,
    clippy::cargo,
    missing_docs,
    rustdoc::all
)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::expect_used,
    clippy::implicit_return,
    clippy::needless_borrowed_reference,
    clippy::panic,
    clippy::question_mark_used,
    clippy::string_add
)]
#![allow(clippy::needless_pass_by_value)] // TODO: REMOVE

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

/// Immediately exit with an error associated with a span of source code.
macro_rules! bail {
    ($span:expr, $msg:expr) => {
        return Err(syn::Error::new($span, $msg))
    };
}

/// Make a delimiting token.
macro_rules! delim_token {
    (Paren) => {
        syn::token::Paren {
            span: proc_macro2::Group::new(
                proc_macro2::Delimiter::Parenthesis,
                proc_macro2::TokenStream::new(),
            )
            .delim_span(),
        }
    };
    ($name:ident) => {
        syn::token::$name {
            span: proc_macro2::Group::new(
                proc_macro2::Delimiter::$name,
                proc_macro2::TokenStream::new(),
            )
            .delim_span(),
        }
    };
}

/// Make a single token.
macro_rules! single_token {
    ($n:ident) => {
        syn::token::$n {
            spans: [Span::call_site()],
        }
    };
}

/// Make a punctuated list from elements.
macro_rules! punctuate {
    ($($tt:tt)*) => {{
        let mut _punctuated = syn::punctuated::Punctuated::new();
        for _element in [$($tt)*] {
            _punctuated.push(_element);
        }
        _punctuated
    }};
}

/// Make a `syn` identifier from a raw identifier.
macro_rules! ident {
    ($i:ident) => {
        syn::Ident::new(stringify!($i), Span::call_site())
    };
}

/// Automatically derive `quickcheck::Arbitrary`.
#[proc_macro_derive(QuickCheck)]
pub fn arbitrary(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(ts as syn::DeriveInput);
    from_derive_input(input)
        .map_or_else(
            syn::Error::into_compile_error,
            syn::ItemMod::into_token_stream,
        )
        .into()
}

/// Test that `Arbitrary::arbitrary` doesn't panic by making a `prop_` that takes an argument then discards it and returns true.
fn make_trivial_prop(ident: &syn::Ident, generics: &syn::Generics) -> syn::Result<syn::Item> {
    Ok(syn::Item::Macro(syn::ItemMacro {
        attrs: vec![],
        ident: None,
        mac: syn::Macro {
            path: syn::parse2(quote! { ::quickcheck::quickcheck })?,
            bang_token: single_token!(Not),
            delimiter: syn::MacroDelimiter::Brace(delim_token!(Brace)),
            tokens: syn::ItemFn {
                attrs: vec![],
                vis: syn::Visibility::Inherited,
                sig: syn::Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: syn::parse2(quote! { fn })?,
                    ident: ident!(prop_doesnt_panic),
                    generics: syn::Generics {
                        lt_token: None,
                        params: syn::punctuated::Punctuated::new(),
                        gt_token: None,
                        where_clause: None,
                    },
                    paren_token: delim_token!(Paren),
                    inputs: punctuate!(syn::FnArg::Typed(syn::PatType {
                        attrs: vec![],
                        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: ident!(_unused),
                            subpat: None
                        })),
                        colon_token: single_token!(Colon),
                        ty: Box::new(syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: punctuate!(syn::PathSegment {
                                    ident: ident.clone(),
                                    arguments: syn::PathArguments::AngleBracketed(
                                        syn::AngleBracketedGenericArguments {
                                            colon2_token: None,
                                            lt_token: single_token!(Lt),
                                            args: generics
                                                .params
                                                .iter()
                                                .map(move |p| match *p {
                                                    syn::GenericParam::Type(_) =>
                                                        syn::GenericArgument::Type(
                                                            syn::Type::Tuple(syn::TypeTuple {
                                                                paren_token: delim_token!(Paren),
                                                                elems: punctuate!()
                                                            })
                                                        ),
                                                    syn::GenericParam::Lifetime(_) =>
                                                        syn::GenericArgument::Lifetime(
                                                            syn::parse2(quote! { 'static })
                                                                .expect("`derive-quickcheck`-internal: couldn't parse `'static`")
                                                        ),
                                                    syn::GenericParam::Const(_) =>
                                                        syn::GenericArgument::Const(
                                                            syn::Expr::Block(
                                                                syn::parse2(
                                                                    quote! { { 0 } } // TODO: no way to do this in general
                                                                )
                                                                .expect("`derive-quickcheck`-internal: couldn't parse `{ 0 }`")
                                                            )
                                                        ),
                                                })
                                                .collect(),
                                            gt_token: single_token!(Gt)
                                        }
                                    )
                                })
                            }
                        }))
                    })),
                    variadic: None,
                    output: syn::parse2(quote! { -> bool })?,
                },
                block: Box::new(syn::parse2(quote! { { true } })?),
            }
            .into_token_stream(),
        },
        semi_token: None,
    }))
}

/// Potentially fail with a compilation error.
fn from_derive_input(i: syn::DeriveInput) -> syn::Result<syn::ItemMod> {
    use heck::ToSnakeCase;
    let mod_name = &(i.ident.to_string().to_snake_case() + "_derive_quickcheck");
    Ok(syn::ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        unsafety: None,
        mod_token: syn::parse2(quote! { mod })?,
        ident: syn::Ident::new(mod_name, Span::call_site()),
        content: Some((
            delim_token!(Brace),
            vec![
                syn::Item::Use(syn::parse2(quote! { use super::*; })?),
                make_trivial_prop(&i.ident, &i.generics)?,
                syn::Item::Impl(match i.data {
                    syn::Data::Enum(d) => from_enum(i.attrs, i.ident, i.generics, d),
                    syn::Data::Struct(d) => from_struct(i.attrs, i.ident, i.generics, d),
                    syn::Data::Union(d) => from_union(i.attrs, i.ident, i.generics, d),
                }?),
            ],
        )),
        semi: None,
    })
}

/// Call a type's static `arbitrary` function.
fn static_arbitrary(ty: syn::Type) -> syn::Expr {
    let mut e: syn::ExprCall = syn::parse2(
                        quote! { <A as ::quickcheck::Arbitrary>::arbitrary(&mut smaller_gen) },
                    ).expect("`derive-quickcheck`-internal: Expected to be able to parse our internal implementation but couldn't");
    let &mut syn::Expr::Path(ref mut p) = e.func.as_mut() else { panic!("`derive-quickcheck`-internal: Expected a path") }; // <A as ::quickcheck::Arbitrary>::arbitrary
    let Some(qself) = p.qself.as_mut() else { panic!("`derive-quickcheck`-internal: Expected a qself (i.e. `<A as T>::...`)") };
    *qself.ty.as_mut() = ty;
    syn::Expr::Call(e)
}

/// Call `arbitrary` on all these fields and wrap it in the appropriate `{...}` or `(...)`.
#[allow(clippy::too_many_lines)]
fn all_of(path: syn::Path, fields: syn::Fields) -> syn::Expr {
    #[allow(clippy::expect_used, clippy::panic)]
    let decr_size_by = fields.len();
    match fields {
        syn::Fields::Unit => syn::Expr::Path(syn::ExprPath {
            attrs: vec![],
            qself: None,
            path,
        }),
        syn::Fields::Unnamed(members) => syn::Expr::Block(syn::ExprBlock {
            attrs: vec![],
            label: None,
            block: syn::Block {
                brace_token: delim_token!(Brace),
                stmts: vec![
                    syn::Stmt::Local(syn::Local {
                        attrs: vec![],
                        let_token: syn::token::Let {
                            span: Span::call_site(),
                        },
                        pat: syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: Some(syn::token::Mut {
                                span: Span::call_site(),
                            }),
                            ident: ident!(smaller_gen),
                            subpat: None,
                        }),
                        init: Some(syn::LocalInit {
                            eq_token: single_token!(Eq),
                            expr: Box::new(syn::Expr::Call(syn::ExprCall {
                                attrs: vec![],
                                func: Box::new(syn::Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: Some(syn::token::PathSep {
                                            spans: [Span::call_site(), Span::call_site()],
                                        }),
                                        segments: punctuate!(
                                            syn::PathSegment {
                                                ident: ident!(quickcheck),
                                                arguments: syn::PathArguments::None
                                            },
                                            syn::PathSegment {
                                                ident: ident!(Gen),
                                                arguments: syn::PathArguments::None
                                            },
                                            syn::PathSegment {
                                                ident: ident!(new),
                                                arguments: syn::PathArguments::None
                                            },
                                        ),
                                    },
                                })),
                                paren_token: delim_token!(Paren),
                                args: punctuate!(syn::Expr::MethodCall(syn::ExprMethodCall {
                                    attrs: vec![],
                                    receiver: Box::new(syn::Expr::MethodCall(
                                        syn::ExprMethodCall {
                                            attrs: vec![],
                                            receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                                                attrs: vec![],
                                                qself: None,
                                                path: syn::Path {
                                                    leading_colon: None,
                                                    segments: punctuate!(syn::PathSegment {
                                                        ident: ident!(g),
                                                        arguments: syn::PathArguments::None
                                                    })
                                                }
                                            })),
                                            dot_token: single_token!(Dot),
                                            method: ident!(size),
                                            turbofish: None,
                                            paren_token: delim_token!(Paren),
                                            args: punctuate!(),
                                        }
                                    )),
                                    dot_token: single_token!(Dot),
                                    method: ident!(saturating_sub),
                                    turbofish: None,
                                    paren_token: delim_token!(Paren),
                                    args: punctuate![syn::Expr::Lit(syn::ExprLit {
                                        attrs: vec![],
                                        lit: syn::Lit::Verbatim(
                                            proc_macro2::Literal::usize_unsuffixed(decr_size_by)
                                        )
                                    })]
                                })),
                            })),
                            diverge: None,
                        }),
                        semi_token: single_token!(Semi),
                    }),
                    syn::Stmt::Expr(
                        syn::Expr::Call(syn::ExprCall {
                            attrs: vec![],
                            func: Box::new(syn::Expr::Path(syn::ExprPath {
                                attrs: vec![],
                                qself: None,
                                path,
                            })),
                            paren_token: delim_token!(Paren),
                            args: members
                                .unnamed
                                .into_iter()
                                .map(move |f| static_arbitrary(f.ty))
                                .collect(),
                        }),
                        None,
                    ),
                ],
            },
        }),
        syn::Fields::Named(members) => syn::Expr::Block(syn::ExprBlock {
            attrs: vec![],
            label: None,
            block: syn::Block {
                brace_token: delim_token!(Brace),
                stmts: vec![
                    syn::Stmt::Local(syn::Local {
                        attrs: vec![],
                        let_token: syn::token::Let {
                            span: Span::call_site(),
                        },
                        pat: syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: Some(syn::token::Mut {
                                span: Span::call_site(),
                            }),
                            ident: ident!(smaller_gen),
                            subpat: None,
                        }),
                        init: Some(syn::LocalInit {
                            eq_token: single_token!(Eq),
                            expr: Box::new(syn::Expr::Call(syn::ExprCall {
                                attrs: vec![],
                                func: Box::new(syn::Expr::Path(syn::ExprPath {
                                    attrs: vec![],
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: Some(syn::token::PathSep {
                                            spans: [Span::call_site(), Span::call_site()],
                                        }),
                                        segments: punctuate!(
                                            syn::PathSegment {
                                                ident: ident!(quickcheck),
                                                arguments: syn::PathArguments::None
                                            },
                                            syn::PathSegment {
                                                ident: ident!(Gen),
                                                arguments: syn::PathArguments::None
                                            },
                                            syn::PathSegment {
                                                ident: ident!(new),
                                                arguments: syn::PathArguments::None
                                            },
                                        ),
                                    },
                                })),
                                paren_token: delim_token!(Paren),
                                args: punctuate!(syn::Expr::MethodCall(syn::ExprMethodCall {
                                    attrs: vec![],
                                    receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                                        attrs: vec![],
                                        qself: None,
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: punctuate!(syn::PathSegment {
                                                ident: ident!(g),
                                                arguments: syn::PathArguments::None
                                            })
                                        }
                                    })),
                                    dot_token: single_token!(Dot),
                                    method: ident!(size),
                                    turbofish: None,
                                    paren_token: delim_token!(Paren),
                                    args: punctuate!(),
                                })),
                            })),
                            diverge: None,
                        }),
                        semi_token: single_token!(Semi),
                    }),
                    syn::Stmt::Expr(
                        syn::Expr::Struct(syn::ExprStruct {
                            attrs: vec![],
                            qself: None,
                            path,
                            brace_token: delim_token!(Brace),
                            fields: members
                                .named
                                .into_iter()
                                .map(move |f| syn::FieldValue {
                                    attrs: f.attrs,
                                    member: f.ident.map_or_else(
                                        || {
                                            syn::Member::Unnamed(syn::Index {
                                                index: u32::MAX,
                                                span: f.ty.span(),
                                            })
                                        },
                                        syn::Member::Named,
                                    ),
                                    colon_token: f.colon_token,
                                    expr: static_arbitrary(f.ty),
                                })
                                .collect(),
                            dot2_token: None,
                            rest: None,
                        }),
                        None,
                    ),
                ],
            },
        }),
    }
}

/// Choose one of many variants and call `arbitrary` on all its members.
#[allow(clippy::too_many_lines)]
fn one_of(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    max_len: usize,
) -> syn::Result<syn::Expr> {
    if variants.is_empty() {
        bail!(
            variants.span(),
            "Need at least one variant to instantiate the value"
        )
    }
    let fn_type = syn::Type::BareFn(syn::TypeBareFn {
        lifetimes: None,
        unsafety: None,
        abi: None,
        fn_token: syn::token::Fn {
            span: Span::call_site(),
        },
        paren_token: delim_token!(Paren),
        inputs: punctuate!(syn::BareFnArg {
            attrs: vec![],
            name: None,
            ty: syn::Type::Reference(syn::TypeReference {
                and_token: single_token!(And),
                lifetime: None,
                mutability: Some(syn::token::Mut {
                    span: Span::call_site()
                }),
                elem: Box::new(syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: Some(syn::token::PathSep {
                            spans: [Span::call_site(), Span::call_site()]
                        }),
                        segments: punctuate!(
                            syn::PathSegment {
                                ident: ident!(quickcheck),
                                arguments: syn::PathArguments::None
                            },
                            syn::PathSegment {
                                ident: ident!(Gen),
                                arguments: syn::PathArguments::None
                            }
                        ),
                    }
                }))
            }),
        }),
        variadic: None,
        output: syn::ReturnType::Type(
            syn::token::RArrow {
                spans: [Span::call_site(), Span::call_site()],
            },
            Box::new(syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: punctuate!(syn::PathSegment {
                        ident: ident!(Self),
                        arguments: syn::PathArguments::None,
                    }),
                },
            })),
        ),
    });
    let elems = variants
        .into_iter()
        .filter(|v| v.fields.len() <= max_len)
        .map(|v| {
            syn::Expr::Cast(syn::ExprCast {
                attrs: vec![],
                expr: Box::new(syn::Expr::Paren(syn::ExprParen {
                    attrs: vec![],
                    paren_token: delim_token!(Paren),
                    expr: Box::new(syn::Expr::Closure(syn::ExprClosure {
                        attrs: vec![],
                        lifetimes: None,
                        constness: None,
                        movability: None,
                        asyncness: None,
                        capture: Some(syn::token::Move {
                            span: Span::call_site(),
                        }),
                        or1_token: single_token!(Or),
                        inputs: punctuate!(syn::Pat::Ident(syn::PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: ident!(g),
                            subpat: None
                        })),
                        or2_token: single_token!(Or),
                        output: syn::ReturnType::Default,
                        body: Box::new(all_of(
                            syn::Path {
                                leading_colon: None,
                                segments: punctuate!(
                                    syn::PathSegment {
                                        ident: ident!(Self),
                                        arguments: syn::PathArguments::None
                                    },
                                    syn::PathSegment {
                                        ident: v.ident.clone(),
                                        arguments: syn::PathArguments::None
                                    }
                                ),
                            },
                            v.fields.clone(),
                        )),
                    })),
                })),
                as_token: syn::token::As {
                    span: Span::call_site(),
                },
                ty: Box::new(fn_type.clone()),
            })
        })
        .collect();
    Ok(syn::Expr::Call(syn::ExprCall {
        attrs: vec![],
        func: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
            attrs: vec![],
            receiver: Box::new(syn::Expr::MethodCall(syn::ExprMethodCall {
                attrs: vec![],
                receiver: Box::new(syn::Expr::Path(syn::ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments: punctuate!(syn::PathSegment {
                            ident: ident!(g),
                            arguments: syn::PathArguments::None
                        }),
                    },
                })),
                dot_token: single_token!(Dot),
                method: ident!(choose),
                turbofish: Some(syn::AngleBracketedGenericArguments {
                    colon2_token: Some(syn::token::PathSep {
                        spans: [Span::call_site(), Span::call_site()],
                    }),
                    lt_token: single_token!(Lt),
                    args: punctuate!(syn::GenericArgument::Type(fn_type)),
                    gt_token: single_token!(Gt),
                }),
                paren_token: delim_token!(Paren),
                args: punctuate!(syn::Expr::Reference(syn::ExprReference {
                    attrs: vec![],
                    and_token: single_token!(And),
                    mutability: None,
                    expr: Box::new(syn::Expr::Array(syn::ExprArray {
                        attrs: vec![],
                        bracket_token: delim_token!(Bracket),
                        elems
                    }))
                })),
            })),
            dot_token: single_token!(Dot),
            method: ident!(unwrap),
            turbofish: None,
            paren_token: delim_token!(Paren),
            args: punctuate!(),
        })),
        paren_token: delim_token!(Paren),
        args: punctuate!(syn::Expr::Path(syn::ExprPath {
            attrs: vec![],
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: punctuate!(syn::PathSegment {
                    ident: ident!(g),
                    arguments: syn::PathArguments::None
                })
            }
        })),
    }))
}

/// `GenericParam` to `GenericArgument`.
fn param2arg(p: syn::GenericParam) -> syn::GenericArgument {
    match p {
        syn::GenericParam::Lifetime(lt) => syn::GenericArgument::Lifetime(lt.lifetime),
        syn::GenericParam::Type(t) => syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: punctuate!(syn::PathSegment {
                    ident: t.ident,
                    arguments: syn::PathArguments::None
                }),
            },
        })),
        syn::GenericParam::Const(c) => {
            syn::GenericArgument::Const(syn::Expr::Path(syn::ExprPath {
                attrs: c.attrs,
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: punctuate!(syn::PathSegment {
                        ident: c.ident,
                        arguments: syn::PathArguments::None
                    }),
                },
            }))
        }
    }
}

/// Add `: ::quickcheck::Arbitrary` to each type parameter.
fn constrain_generics(generics: &syn::Generics) -> syn::Generics {
    syn::Generics {
        params: generics
            .params
            .iter()
            .map(move |p| match p {
                &syn::GenericParam::Type(ref t) => syn::GenericParam::Type(syn::TypeParam {
                    bounds: {
                        let mut b = t.bounds.clone();
                        b.push(syn::TypeParamBound::Trait(
                            syn::parse2(quote! { ::quickcheck::Arbitrary }).expect("`derive-quickcheck`-internal: Expected to be able to parse `::quickcheck::Arbitrary` but couldn't."),
                        ));
                        b
                    },
                    eq_token: None,
                    default: None,
                    ..t.clone()
                }),
                &syn::GenericParam::Lifetime(_) | &syn::GenericParam::Const(_) => p.clone(),
            })
            .collect(),
        ..generics.clone()
    }
}

/// Write `Self<A, ...>` after `impl<A: ...>`
fn make_self_ty(ident: syn::Ident, generics: syn::Generics) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path {
            leading_colon: None,
            segments: punctuate!(syn::PathSegment {
                ident,
                arguments: syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: single_token!(Lt),
                        args: generics.params.into_iter().map(param2arg).collect(),
                        gt_token: single_token!(Gt)
                    }
                ),
            }),
        },
    })
}

/// Write `fn arbitrary(...` as part of the `impl`.
fn make_arbitrary_fn(stmts: Vec<syn::Stmt>) -> syn::Result<syn::ImplItem> {
    Ok(syn::ImplItem::Fn(syn::ImplItemFn {
        attrs: vec![syn::Attribute {
            pound_token: single_token!(Pound),
            bracket_token: delim_token!(Bracket),
            style: syn::AttrStyle::Outer,
            meta: syn::Meta::Path(syn::parse2(quote! { inline })?),
        }],
        vis: syn::Visibility::Inherited,
        defaultness: None,
        sig: syn::parse2(quote! { fn arbitrary(g: &mut ::quickcheck::Gen) -> Self })?,
        block: syn::Block {
            brace_token: delim_token!(Brace),
            stmts,
        },
    }))
}

/// Implement for an `enum`.
fn from_enum(
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    generics: syn::Generics,
    d: syn::DataEnum,
) -> syn::Result<syn::ItemImpl> {
    Ok(syn::ItemImpl {
        attrs,
        defaultness: None,
        unsafety: None,
        impl_token: syn::parse2(quote! { impl })?,
        generics: constrain_generics(&generics),
        trait_: Some((
            None,
            syn::parse2(quote! { ::quickcheck::Arbitrary })?,
            syn::parse2(quote! { for })?,
        )),
        self_ty: Box::new(make_self_ty(ident, generics)),
        brace_token: delim_token!(Brace),
        items: vec![make_arbitrary_fn(vec![syn::Stmt::Expr(
            syn::Expr::Match(syn::ExprMatch {
                attrs: vec![],
                match_token: syn::parse2(quote! { match })?,
                expr: Box::new(syn::parse2(quote! { g.size() })?),
                brace_token: delim_token!(Brace),
                arms: {
                    let most_fields = d.variants.iter().fold(0, |acc, v| acc.max(v.fields.len()));
                    let mut arms = vec![];
                    if most_fields > 0 {
                        arms.push(syn::Arm {
                            attrs: vec![],
                            pat: syn::Pat::Lit(syn::ExprLit {
                                attrs: vec![],
                                lit: syn::Lit::Verbatim(proc_macro2::Literal::usize_unsuffixed(0)),
                            }),
                            guard: None,
                            fat_arrow_token: syn::parse2(quote! { => })?,
                            body: Box::new(one_of(&d.variants, 0)?),
                            comma: Some(syn::parse2(quote! { , })?),
                        });
                        for i in 0..most_fields {
                            arms.push(syn::Arm {
                                attrs: vec![],
                                pat: syn::Pat::Lit(syn::ExprLit {
                                    attrs: vec![],
                                    lit: syn::Lit::Verbatim(
                                        proc_macro2::Literal::usize_unsuffixed(
                                            i.checked_add(1).ok_or_else(|| {
                                                syn::Error::new(
                                                d.variants.span(),
                                                "Ridiculously huge number of fields in a variant",
                                            )
                                            })?,
                                        ),
                                    ),
                                }),
                                guard: None,
                                fat_arrow_token: syn::parse2(quote! { => })?,
                                body: Box::new(one_of(&d.variants, i)?),
                                comma: Some(syn::parse2(quote! { , })?),
                            });
                        }
                    }
                    arms.push(syn::Arm {
                        attrs: vec![],
                        pat: syn::Pat::Wild(syn::PatWild {
                            attrs: vec![],
                            underscore_token: syn::parse2(quote! { _ })?,
                        }),
                        guard: None,
                        fat_arrow_token: syn::parse2(quote! { => })?,
                        body: Box::new(one_of(&d.variants, usize::MAX)?),
                        comma: Some(syn::parse2(quote! { , })?),
                    });
                    arms
                },
            }),
            None,
        )])?],
    })
}

/// Implement for a `struct`.
fn from_struct(
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    generics: syn::Generics,
    d: syn::DataStruct,
) -> syn::Result<syn::ItemImpl> {
    Ok(syn::ItemImpl {
        attrs,
        defaultness: None,
        unsafety: None,
        impl_token: syn::parse2(quote! { impl })?,
        generics: constrain_generics(&generics),
        trait_: Some((
            None,
            syn::parse2(quote! { ::quickcheck::Arbitrary })?,
            syn::parse2(quote! { for })?,
        )),
        self_ty: Box::new(make_self_ty(ident, generics)),
        brace_token: delim_token!(Brace),
        items: vec![make_arbitrary_fn(vec![syn::Stmt::Expr(
            all_of(
                syn::Path {
                    leading_colon: None,
                    segments: punctuate!(syn::PathSegment {
                        ident: ident!(Self),
                        arguments: syn::PathArguments::None
                    }),
                },
                d.fields,
            ),
            None,
        )])?],
    })
}

/// Implement for a `union`.
fn from_union(
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    generics: syn::Generics,
    _d: syn::DataUnion,
) -> syn::Result<syn::ItemImpl> {
    Ok(syn::ItemImpl {
        attrs,
        defaultness: None,
        unsafety: None,
        impl_token: syn::parse2(quote! { impl })?,
        generics: constrain_generics(&generics),
        trait_: Some((
            None,
            syn::parse2(quote! { ::quickcheck::Arbitrary })?,
            syn::parse2(quote! { for })?,
        )),
        self_ty: Box::new(make_self_ty(ident, generics)),
        brace_token: delim_token!(Brace),
        items: vec![make_arbitrary_fn(vec![syn::Stmt::Macro(syn::StmtMacro {
            attrs: vec![],
            mac: syn::parse2(
                quote! { todo!("`union`s not yet implemented in ``derive-quickcheck`") },
            )?,
            semi_token: None,
        })])?],
    })
}
