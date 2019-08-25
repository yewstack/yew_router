extern crate proc_macro;
use proc_macro::{TokenStream};
use yew_router_route_parser::{OptimizedToken, CaptureVariants};
use quote::{quote, ToTokens};
use syn::export::TokenStream2;
use proc_macro_hack::proc_macro_hack;
use syn::{Error, Type};

use syn::parse::{Parse, ParseBuffer};
use syn::parse_macro_input;
use syn::Token;
use syn::Expr;
use syn::spanned::Spanned;

enum Either<T, U> {
    Left(T),
    Right(U)
}

struct S {
    s: String,
    target: Option<Either<Type, Expr>>
}
impl Parse for S {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let s = input.parse::<syn::LitStr>()?;
        // The specification of a type must be preceded by a "=>"
        let lookahead = input.lookahead1();
        let target: Option<Either<Type, Expr>> = if lookahead.peek(Token![=>]) {
            input.parse::<syn::token::FatArrow>()
                .ok()
                .map(|_| {
                    let lookahead = input.lookahead1();
                    input.parse::<Type>()
                        .map(Either::Left)
                })
                .transpose()?
        } else if lookahead.peek(Token![,]){
            input.parse::<syn::token::Comma>()
                .ok()
                .map(|_| {
//                    input.parse::<syn::Lit>()
                    input.parse::<syn::Expr>()
                        .and_then(|expr| {
                            match &expr {
                                Expr::Closure(_) | Expr::Block(_) | Expr::MethodCall(_) | Expr::Call(_) | Expr::Lit(_) => Ok(expr),
                                Expr::Group(_) => Err(Error::new(expr.span(), "Erroneous error")),
                                Expr::Path(_) => panic!("path"), // TODO this is broken.
                                Expr::__Nonexhaustive => panic!("nonexhaustive"),
                                _ => Err(Error::new(expr.span(), "Must be a Component's Type, a closure returning Option<Html<_>>, or expression that can resolve to such a closure."))
                            }
                        })
                        .map(Either::Right)
                })
                .transpose()?
        } else {
            None
        };

        let expr = input.parse::<syn::Expr>()
            .and_then(|expr| {
                match &expr {
                    Expr::Closure(_) | Expr::Block(_) | Expr::Call(_) | Expr::Lit(_) => Ok(expr),
                    _ => Err(Error::new(expr.span(), "Must be closure, or structure that can resolve to a closure."))
                }
            });

        Ok(
            S {
                s: s.value(),
                target
            }
        )
    }
}

/// Expected to be used like: route!("/route/to/thing" => Component)
#[proc_macro_hack]
pub fn route(input: TokenStream) -> TokenStream {
    let s = parse_macro_input!(input as S);
    let target = s.target;
    let s: String = s.s;

    // Do the parsing at compile time so the user knows if their matcher is malformed.
    // It will still be their responsibility to know that the corresponding Props can be acquired from a path matcher.
    let t = yew_router_route_parser::parse_str_and_optimize_tokens(s.as_str())
        .expect("Invalid Path Matcher")
        .into_iter()
        .map(ShadowOptimizedToken::from);


    let render_fn = match target {
        Some(target) => {
            match target {
                Either::Left(ty) => {
                    quote! {
                        use std::marker::PhantomData as __PhantomData;
                        use yew_router::Router as __Router;
                        let phantom: __PhantomData<#ty> = __PhantomData;
                        let render_fn = Some(__PathMatcher::<__Router>::create_render_fn::<#ty>(phantom));
                    }
                }
                Either::Right(expr) => {
                    quote! {
                        let x: Box<Fn(&std::collections::HashMap<String,String>) -> Option<Html<_>> > = Box::new(#expr);
                        let render_fn = Some(x);
                    }
                }
            }
        },
        None => quote!{
            let render_fn = None;
        }
    };

    let expanded = quote!{
        {
            use yew_router::path_matcher::PathMatcher as __PathMatcher;
            use yew_router::path_matcher::CaptureVariants as __CaptureVariants;
            use yew_router::path_matcher::OptimizedToken as __OptimizedToken;

            #render_fn

            __PathMatcher {
                tokens : vec![#(#t),*],
                render_fn
            }
        }
    };
    TokenStream::from(expanded)
}

impl ToTokens for ShadowOptimizedToken {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        use ShadowOptimizedToken as SOT;
        let t: TokenStream2 = match self {
            SOT::Match(s) => {
                TokenStream2::from(quote!{__OptimizedToken::Match(#s.to_string())})
            }
            SOT::Capture ( variant ) => {
                TokenStream2::from(quote!{
                    __OptimizedToken::Capture(#variant)
                })
            }
            SOT::QueryCapture { ident, value } => {
                let ident = ident.clone();
                let value = value.clone();
                TokenStream2::from(quote!{
                    __OptimizedToken::QueryCapture{ident: #ident.to_string(), value: #value.to_string()}
                })
            }
        };
        ts.extend(t)
    }
}

/// A shadow of the OptimizedToken type.
/// It should match it exactly so that this macro can expand to the original.
enum ShadowOptimizedToken {
    Match(String),
    Capture(ShadowCaptureVariant),
    QueryCapture {
        ident: String,
        value: String
    }
}

enum ShadowCaptureVariant {
    Unnamed, // {} - matches anything
    ManyUnnamed, // {*} - matches over multiple sections
    NumberedUnnamed{sections: usize}, // {4} - matches 4 sections
    Named(String), // {name} - captures a section and adds it to the map with a given name
    ManyNamed(String), // {*:name} - captures over many sections and adds it to the map with a given name.
    NumberedNamed{sections: usize, name: String} // {2:name} - captures a fixed number of sections with a given name.
}

impl ToTokens for ShadowCaptureVariant {

    fn to_tokens(&self, ts: &mut TokenStream2) {
        let t = match self {
            ShadowCaptureVariant::Unnamed => TokenStream2::from(quote!{__CaptureVariants::Unnamed}),
            ShadowCaptureVariant::ManyUnnamed => TokenStream2::from(quote!{__CaptureVariants::ManyUnnamed}),
            ShadowCaptureVariant::NumberedUnnamed { sections } => TokenStream2::from(quote!{__CaptureVariants::NumberedUnnamed{#sections}}),
            ShadowCaptureVariant::Named(name) => TokenStream2::from(quote!{__CaptureVariants::Named(#name.to_string())}),
            ShadowCaptureVariant::ManyNamed(name) => TokenStream2::from(quote!{__CaptureVariants::ManyNamed(#name.to_string())}),
            ShadowCaptureVariant::NumberedNamed { sections, name } => TokenStream2::from(quote!{__CaptureVariants::NumberedNamed{#sections, #name.to_string()}}),
        };
        ts.extend(t)

    }
}

impl From<OptimizedToken> for ShadowOptimizedToken {
    fn from(ot: OptimizedToken) -> Self {
        use OptimizedToken as OT;
        use ShadowOptimizedToken as SOT;
        match ot {
            OT::Match(s) => SOT::Match(s),
            OT::Capture(variant) => SOT::Capture(variant.into()),
            OT::QueryCapture { ident, value } => SOT::QueryCapture {ident, value}
        }
    }
}

impl From<CaptureVariants> for ShadowCaptureVariant {

    fn from(cv: CaptureVariants) -> Self {
        use CaptureVariants as CV;
        use ShadowCaptureVariant as SCV;
        match cv {
            CV::Unnamed => SCV::Unnamed,
            CaptureVariants::ManyUnnamed => SCV::ManyUnnamed,
            CaptureVariants::NumberedUnnamed { sections } => SCV::NumberedUnnamed {sections},
            CaptureVariants::Named(name) => SCV::Named(name),
            CaptureVariants::ManyNamed(name) => SCV::ManyNamed(name),
            CaptureVariants::NumberedNamed { sections, name } => SCV::NumberedNamed {sections, name}
        }

    }
}
