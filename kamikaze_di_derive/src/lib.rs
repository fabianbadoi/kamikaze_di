extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident};

#[proc_macro_derive(Resolve)]
pub fn derive_resolve(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    if let Data::Struct(structure) = input.data {
        return match structure.fields {
            Fields::Named(fields) => derive_for_named(name, fields),
            Fields::Unnamed(fields) => derive_for_unnamed(name, fields),
            _ => unimplemented!(),
        };
    };

    unimplemented!()
}

fn derive_for_named(name: Ident, fields: FieldsNamed) -> TokenStream {
    let resolve_fields = fields.named.iter().map(|field| {
        let name = &field.ident;

        quote_spanned! {field.span()=>
            #name: kamikaze_di::AutoResolver::resolve(container)?,
        }
    });

    TokenStream::from(quote! {
        impl kamikaze_di::Resolve for #name {
            fn resolve(container: &kamikaze_di::Container) -> kamikaze_di::Result<Self> {
                Ok(#name {
                    #(#resolve_fields)*
                })
            }
        }
    })
}

fn derive_for_unnamed(name: Ident, fields: FieldsUnnamed) -> TokenStream {
    let resolve_fields = fields.unnamed.iter().enumerate().map(|(_index, field)| {
        quote_spanned! {field.span()=>
            kamikaze_di::AutoResolver::resolve(container)?,
        }
    });

    TokenStream::from(quote! {
        impl kamikaze_di::Resolve for #name {
            fn resolve(container: &kamikaze_di::Container) -> kamikaze_di::Result<Self> {
                Ok(#name (
                    #(#resolve_fields)*
                ))
            }
        }
    })
}
