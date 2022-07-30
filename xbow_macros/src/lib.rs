use proc_macro2::{Ident, Span};
use quote::quote;
use syn::DeriveInput;
#[proc_macro_derive(Project, attributes(project))]
pub fn derive_project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, generics) = (&ast.vis, &ast.ident, &ast.generics);
    let projection_struct_ident = Ident::new(&format!("🏹⁅{}⁆", ty.to_string()), Span::call_site());
    todo!()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
