extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_str, Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident,
    Path,
};

#[proc_macro_derive(Resolve)]
pub fn derive_resolve(input: TokenStream) -> TokenStream {
    derive_code(input, "kamikaze_di::Resolve")
}

#[proc_macro_derive(ResolveToRc)]
pub fn derive_resolve_to_rc(input: TokenStream) -> TokenStream {
    derive_code(input, "kamikaze_di::ResolveToRc")
}

fn derive_code(input: TokenStream, trait_path: &str) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let resolve_type = parse_str::<Path>(trait_path).unwrap();

    if let Data::Struct(structure) = input.data {
        return match structure.fields {
            Fields::Named(fields) => derive_for_named(name, fields, resolve_type),
            Fields::Unnamed(fields) => derive_for_unnamed(name, fields, resolve_type),
            _ => unimplemented!(),
        };
    };

    unimplemented!()
}

fn derive_for_named(name: Ident, fields: FieldsNamed, resolve_type: Path) -> TokenStream {
    let resolve_fields = fields.named.iter().map(|field| {
        let name = &field.ident;

        quote_spanned! {field.span()=>
            #name: kamikaze_di::AutoResolver::resolve(container)?,
        }
    });

    let quote = quote! {
        impl #resolve_type for #name {
            fn resolve(container: &kamikaze_di::Container) -> kamikaze_di::Result<Self> {
                Ok(#name {
                    #(#resolve_fields)*
                })
            }
        }
    };

    TokenStream::from(quote)
}

fn derive_for_unnamed(name: Ident, fields: FieldsUnnamed, resolve_type: Path) -> TokenStream {
    let resolve_fields = fields.unnamed.iter().enumerate().map(|(_index, field)| {
        quote_spanned! {field.span()=>
            kamikaze_di::AutoResolver::resolve(container)?,
        }
    });

    TokenStream::from(quote! {
        impl #resolve_type for #name {
            fn resolve(container: &kamikaze_di::Container) -> kamikaze_di::Result<Self> {
                Ok(#name (
                    #(#resolve_fields)*
                ))
            }
        }
    })
}
