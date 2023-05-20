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

use quote::{quote, ToTokens};
use syn::spanned::Spanned;

/*
/// Immediately exit with an error associated with a span of source code.
macro_rules! bail {
    ($span:expr, $msg:expr) => {
        return Err(syn::Error::new($span, $msg))
    };
}
*/

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
            spans: [proc_macro2::Span::call_site()],
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
        syn::Ident::new(stringify!($i), proc_macro2::Span::call_site())
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
fn make_trivial_prop(
    mod_name: &str,
    ident: &syn::Ident,
    generics: &syn::Generics,
) -> syn::Result<syn::Item> {
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
                    ident: syn::Ident::new(
                        &("prop_".to_owned() + mod_name),
                        proc_macro2::Span::call_site(),
                    ),
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
                                                                .expect("qcderive-internal: couldn't parse `'static`")
                                                        ),
                                                    syn::GenericParam::Const(_) =>
                                                        syn::GenericArgument::Const(
                                                            syn::Expr::Block(
                                                                syn::parse2(
                                                                    quote! { { 0 } } // TODO: no way to do this in general
                                                                )
                                                                .expect("qcderive-internal: couldn't parse `{ 0 }`")
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
    let mod_name = &(i.ident.to_string().to_snake_case() + "_qcderive");
    Ok(syn::ItemMod {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        unsafety: None,
        mod_token: syn::parse2(quote! { mod })?,
        ident: syn::Ident::new(mod_name, proc_macro2::Span::call_site()),
        content: Some((
            delim_token!(Brace),
            vec![
                syn::Item::Use(syn::parse2(quote! { use super::*; })?),
                make_trivial_prop(mod_name, &i.ident, &i.generics)?,
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
                        quote! { <A as ::quickcheck::Arbitrary>::arbitrary(g) },
                    ).expect("qcderive-internal: Expected to be able to parse our internal implementation but couldn't");
    let &mut syn::Expr::Path(ref mut p) = e.func.as_mut() else { panic!("qcderive-internal: Expected a path") }; // <A as ::quickcheck::Arbitrary>::arbitrary
    let Some(qself) = p.qself.as_mut() else { panic!("qcderive-internal: Expected a qself (i.e. `<A as T>::...`)") };
    *qself.ty.as_mut() = ty;
    syn::Expr::Call(e)
}

/// Call `arbitrary` on all these fields and wrap it in the appropriate `{...}` or `(...)`.
fn all_of(ident: syn::Ident, fields: syn::Fields) -> syn::Expr {
    #[allow(clippy::expect_used, clippy::panic)]
    match fields {
        syn::Fields::Unit => syn::Expr::Path(syn::ExprPath {
            attrs: vec![],
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: punctuate!(syn::PathSegment {
                    ident: ident!(Self),
                    arguments: syn::PathArguments::None,
                }),
            },
        }),
        syn::Fields::Unnamed(members) => syn::Expr::Call(syn::ExprCall {
            attrs: vec![],
            func: Box::new(syn::Expr::Path(syn::ExprPath {
                attrs: vec![],
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: punctuate!(syn::PathSegment {
                        ident,
                        arguments: syn::PathArguments::None,
                    }),
                },
            })),
            paren_token: delim_token!(Paren),
            args: members
                .unnamed
                .into_iter()
                .map(move |f| static_arbitrary(f.ty))
                .collect(),
        }),
        syn::Fields::Named(members) => syn::Expr::Struct(syn::ExprStruct {
            attrs: vec![],
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: punctuate!(syn::PathSegment {
                    ident,
                    arguments: syn::PathArguments::None,
                }),
            },
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
    }
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
                            syn::parse2(quote! { ::quickcheck::Arbitrary }).expect("qcderive-internal: Expected to be able to parse `::quickcheck::Arbitrary` but couldn't."),
                        ));
                        b
                    },
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
    _d: syn::DataEnum,
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
            mac: syn::parse2(quote! { todo!("`enum`s not yet implemented in `qcderive`") })?,
            semi_token: None,
        })])?],
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
            all_of(ident!(Self), d.fields),
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
            mac: syn::parse2(quote! { todo!("`union`s not yet implemented in `qcderive`") })?,
            semi_token: None,
        })])?],
    })
}
