use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Path};

pub fn into_inner_path(input: DeriveInput, prefix: Path) -> syn::Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(s) => {
            let name = &input.ident;
            let first_field = s
                .fields
                .iter()
                .next()
                .ok_or_else(|| syn::Error::new_spanned(&s.fields, "no field"))?;
            let first_field_name = first_field.ident.as_ref().ok_or_else(|| {
                syn::Error::new_spanned(first_field, "tuple struct not supported")
            })?;
            let (impl_gnr, type_gnr, where_clause) = input.generics.split_for_impl();
            let Some(syn::GenericParam::Type(last_gnr_type)) = input.generics.params.last() else {
				Err(syn::Error::new_spanned(&input.generics.params, "need path generic"))?
			};
            let last_gnr_type = &last_gnr_type.ident;
            Ok(quote! {
                impl #impl_gnr #prefix::IntoInnerPath<#last_gnr_type> for #name #type_gnr #where_clause {
                    fn into_inner_path(self) -> #last_gnr_type {
                        self.#first_field_name
                    }
                }
            })
        }
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "only struct supported",
        )),
    }
}
