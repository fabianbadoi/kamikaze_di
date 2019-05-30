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

#[proc_macro_derive(Inject)]
pub fn derive_resolve(input: TokenStream) -> TokenStream {
    derive_code(input, "kamikaze_di::Inject")
}

#[proc_macro_derive(InjectAsRc)]
pub fn derive_resolve_to_rc(input: TokenStream) -> TokenStream {
    derive_code(input, "kamikaze_di::InjectAsRc")
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
    let quoted_name = quote!(#name).to_string();

    let resolve_fields = fields.named.iter().map(|field| {
        let name = &field.ident;
        let ty = quote!(#field).to_string();

        quote_spanned! {field.span()=>
            #name: {
                debug!("resolving {}::{}", #quoted_name, #ty);

                kamikaze_di::Injector::inject(container).map_err(|s| {
                    warn!("could not resolve {}::{}", #quoted_name, #ty);

                    format!("could not resolve {}::{}: {}", #quoted_name, #ty, s)
                })?
            },
        }
    });

    let quote = quote! {
        impl #resolve_type for #name {
            fn resolve(container: &kamikaze_di::Container) -> kamikaze_di::Result<Self> {
                debug!("injecting {}", #quoted_name);

                Ok(#name {
                    #(#resolve_fields)*
                })
            }
        }
    };

    TokenStream::from(quote)
}

fn derive_for_unnamed(name: Ident, fields: FieldsUnnamed, resolve_type: Path) -> TokenStream {
    let quoted_name = quote!(#name).to_string();

    let resolve_fields = fields.unnamed.iter().enumerate().map(|(index, field)| {
        let ty = quote!(#field).to_string();

        quote_spanned! {field.span()=>
            {
                debug!("resolving {}::{}::{}", #quoted_name, #index, #ty);

                kamikaze_di::Injector::inject(container).map_err(|s| {
                    warn!("could not resolve {}::{}", #quoted_name, #ty);

                    format!("could not resolve {}::{}: {}", #quoted_name, #ty, s)
                })?
            },
        }
    });

    TokenStream::from(quote! {
        impl #resolve_type for #name {
            fn resolve(container: &kamikaze_di::Container) -> kamikaze_di::Result<Self> {
                debug!("injecting {}", #quoted_name);

                Ok(#name (
                    #(#resolve_fields)*
                ))
            }
        }
    })
}
