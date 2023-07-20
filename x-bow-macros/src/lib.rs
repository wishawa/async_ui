mod attributes;
mod generate;
mod helpers;
mod utils;

use attributes::{ATTRIBUTE_MODULE_PREFIX, ATTRIBUTE_PATH, ATTRIBUTE_REMOTE_TYPE};
use proc_macro2::TokenStream;
use syn::{parse_macro_input, parse_quote, DeriveInput, Expr, Path};

#[proc_macro_derive(IntoPath, attributes(into_path))]
pub fn derive_into_path(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut prefix = None;
    const INTO_PATH: &str = "into_path";
    const PREFIX: &str = "prefix";
    input.attrs.iter().for_each(|attr| {
        if attr.path().is_ident(INTO_PATH) {
            if let Ok(syn::ExprAssign { left, right, .. }) = attr.parse_args() {
                if let (Expr::Path(left), Expr::Path(right)) = (&*left, &*right) {
                    if left.path.is_ident(PREFIX) {
                        prefix = Some(right.path.clone());
                    }
                }
            }
        }
    });
    match helpers::into_path(
        input,
        prefix.unwrap_or_else(|| parse_quote!(::x_bow::__private_macro_only)),
    ) {
        Ok(r) => r,
        Err(e) => e.to_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Trackable, attributes(x_bow, track))]
pub fn derive_trackable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive(input) {
        Ok(r) => r,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
fn derive(ast: DeriveInput) -> syn::Result<TokenStream> {
    let mut remote_path = None;
    let mut prefix_path = None;
    ast.attrs.iter().for_each(|attr| {
        if attr.path().is_ident(ATTRIBUTE_PATH) {
            if let Ok(syn::ExprAssign { left, right, .. }) = attr.parse_args() {
                if let (Expr::Path(left), Expr::Path(right)) = (&*left, &*right) {
                    if left.path.is_ident(ATTRIBUTE_MODULE_PREFIX) {
                        prefix_path = Some(right.path.clone());
                    }
                    if left.path.is_ident(ATTRIBUTE_REMOTE_TYPE) {
                        remote_path = Some(right.path.clone());
                    }
                }
            }
        }
    });
    let remote_path = remote_path.unwrap_or_else(|| Path::from(ast.ident.clone()));
    let prefix_path = prefix_path.unwrap_or_else(|| parse_quote!(::x_bow::__private_macro_only));
    generate::generate(&ast, &remote_path, &prefix_path)
}
