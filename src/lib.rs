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
    clippy::implicit_return,
    clippy::question_mark_used
)]

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
            syn::ItemImpl::into_token_stream,
        )
        .into()
}

/// Potentially fail with a compilation error.
fn from_derive_input(i: syn::DeriveInput) -> syn::Result<syn::ItemImpl> {
    match i.data {
        syn::Data::Enum(d) => from_enum(i.attrs, i.ident, i.generics, d),
        syn::Data::Struct(d) => from_struct(i.attrs, i.ident, i.generics, d),
        syn::Data::Union(d) => from_union(i.attrs, i.ident, i.generics, d),
    }
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

/// Implement for an `enum`.
#[allow(clippy::needless_pass_by_value, unused_variables)] // TODO: REMOVE
fn from_enum(
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    generics: syn::Generics,
    d: syn::DataEnum,
) -> syn::Result<syn::ItemImpl> {
    bail!(
        d.enum_token.span(),
        "#[derive(QuickCheck)] not yet implemented for `struct`s"
    )
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

/// Implement for a `struct`.
fn from_struct(
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    generics: syn::Generics,
    d: syn::DataStruct,
) -> syn::Result<syn::ItemImpl> {
    #![allow(clippy::wildcard_enum_match_arm, clippy::needless_borrowed_reference)]
    #![allow(clippy::unwrap_used, clippy::panic)] // TODO: replace with unreachables

    Ok(syn::ItemImpl {
        attrs,
        defaultness: None,
        unsafety: None,
        impl_token: syn::parse2(quote! { impl })?,
        generics: syn::Generics {
            lt_token: generics.lt_token,
            params: generics
                .params
                .iter()
                .map(move |p| match p {
                    &syn::GenericParam::Type(ref t) => syn::GenericParam::Type(syn::TypeParam {
                        bounds: {
                            let mut b = t.bounds.clone();
                            b.push(syn::TypeParamBound::Trait(
                                syn::parse2(quote! { ::quickcheck::Arbitrary }).unwrap(),
                            ));
                            b
                        },
                        ..t.clone()
                    }),
                    other => other.clone(),
                })
                .collect(),
            gt_token: generics.gt_token,
            where_clause: generics.where_clause,
        },
        trait_: Some((
            None,
            syn::parse2(quote! { ::quickcheck::Arbitrary })?,
            syn::parse2(quote! { for })?,
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
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
        })),
        brace_token: delim_token!(Brace),
        items: vec![syn::ImplItem::Fn(syn::ImplItemFn {
            attrs: vec![syn::Attribute {
                pound_token: single_token!(Pound),
                bracket_token: delim_token!(Bracket),
                style: syn::AttrStyle::Outer,
                meta: syn::Meta::List(syn::parse2(quote! { inline(always) })?),
            }],
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: syn::parse2(quote! { fn arbitrary(g: &mut ::quickcheck::Gen) -> Self })?,
            block: syn::Block {
                brace_token: delim_token!(Brace),
                stmts: vec![syn::Stmt::Expr(all_of(ident!(Self), d.fields), None)],
            },
        })],
    })
}

/// Implement for a `union`.
#[allow(clippy::needless_pass_by_value, unused_variables)] // TODO: REMOVE
fn from_union(
    attrs: Vec<syn::Attribute>,
    ident: syn::Ident,
    generics: syn::Generics,
    d: syn::DataUnion,
) -> syn::Result<syn::ItemImpl> {
    bail!(
        d.union_token.span(),
        "#[derive(QuickCheck)] not yet implemented for `union`s"
    )
}
