use std::result;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_quote, Attribute, Generics, Type};

pub(super) type Result<T> = result::Result<T, TokenStream>;

#[inline(never)]
pub(super) fn compile_err(msg: &str) -> TokenStream {
    TokenStream::from(quote!(compile_error!(#msg);))
}

#[inline(never)]
pub(super) fn parse_failed<T>(name: &str, msg: &str) -> Result<T> {
    Err(compile_err(&format!(
        "`{}` cannot be implemented for {}",
        name, msg
    )))
}

pub(super) fn pin() -> TokenStream2 {
    quote!(::core::pin::Pin)
}

pub(super) fn find_remove(attrs: &mut Vec<Attribute>, ident: &str) -> Option<Attribute> {
    attrs
        .iter()
        .position(|Attribute { path, tts, .. }| path.is_ident(ident) && tts.is_empty())
        .map(|i| remove(attrs, i))
}

pub(super) fn remove<T>(v: &mut Vec<T>, index: usize) -> T {
    match v.len() {
        1 => v.pop().unwrap(),
        2 => v.swap_remove(index),
        _ => v.remove(index),
    }
}

pub(super) fn proj_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("__{}Projection", ident), Span::call_site())
}

pub(super) fn proj_generics(generics: &Generics) -> Generics {
    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!('__a));
    generics
}

pub(super) struct ImplUnpin {
    generics: Option<Generics>,
}

impl ImplUnpin {
    pub(super) fn parse(args: TokenStream, generics: &Generics, name: &str) -> Result<Self> {
        match &*args.to_string() {
            "" => Ok(Self { generics: None }),
            "Unpin" => Ok(Self {
                generics: Some(generics.clone()),
            }),
            _ => Err(compile_err(&format!(
                "`{}` an invalid argument was passed",
                name
            ))),
        }
    }

    pub(super) fn push(&mut self, ty: &Type) {
        if let Some(generics) = &mut self.generics {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::marker::Unpin));
        }
    }

    pub(super) fn build(self, ident: &Ident) -> TokenStream2 {
        self.generics
            .map(|generics| {
                let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
                quote! {
                    impl #impl_generics ::core::marker::Unpin for #ident #ty_generics #where_clause {}
                }
            })
            .unwrap_or_default()
    }
}