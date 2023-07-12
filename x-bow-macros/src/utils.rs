use proc_macro2::Ident;
use quote::quote;
use syn::{punctuated::Punctuated, AngleBracketedGenericArguments, Generics, Member, Token};

pub(crate) fn unbracket_generics(
    generics: &Generics,
) -> (
    Punctuated<syn::GenericArgument, Token![,]>,
    Punctuated<syn::GenericArgument, Token![,]>,
) {
    let (impl_gnr, type_gnr, _) = generics.split_for_impl();
    let [p1, p2]: [_; 2] = [quote!(#impl_gnr), quote!(#type_gnr)]
        .map(syn::parse2)
        .map(|res| {
            res.map_or_else(
                |_| Default::default(),
                |args: AngleBracketedGenericArguments| args.args,
            )
        })
        .map(|mut res| {
            add_trailing_punct(&mut res);
            res
        });
    (p1, p2)
}

pub(crate) fn get_field_member(ident: &Option<Ident>, index: usize) -> Member {
    match ident {
        Some(ident) => Member::from(ident.to_owned()),
        None => Member::from(index),
    }
}

pub(crate) fn add_trailing_punct<T, P: Default>(input: &mut Punctuated<T, P>) {
    if !input.empty_or_trailing() {
        input.push_punct(Default::default())
    }
}
