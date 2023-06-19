mod attributes;
mod generate_struct;
mod utils;

use attributes::{
    ATTRIBUTE_MODULE_PREFIX, ATTRIBUTE_PATH, ATTRIBUTE_REMOTE_TYPE, ATTRIBUTE_TRACK_ALL,
};
use proc_macro2::TokenStream;
use syn::{parse_macro_input, parse_quote, DeriveInput, Expr, Path};

#[proc_macro_derive(Trackable, attributes(x_bow, track_all, track))]
pub fn derive_trackable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match generate(input) {
        Ok(r) => r,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
fn generate(ast: DeriveInput) -> syn::Result<TokenStream> {
    let mut remote_path = None;
    let mut prefix_path = None;
    let mut track_all = false;
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
        track_all |= attr.path().is_ident(ATTRIBUTE_TRACK_ALL);
    });
    let remote_path = remote_path.unwrap_or_else(|| Path::from(ast.ident.clone()));
    let prefix_path = prefix_path.unwrap_or_else(|| parse_quote!(::x_bow::__private_macro_only));

    generate_struct::generate_struct(&ast, &remote_path, &prefix_path)
}
