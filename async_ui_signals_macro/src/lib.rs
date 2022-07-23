use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Track)]
pub fn trackable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let ast: DeriveInput = syn::parse(input).unwrap();
    let (vis, ty, gr) = (&ast.vis, &ast.ident, &ast.generics);
	let (gr_impl, gr_ty, gr_where) = gr.split_for_impl();
	match &ast.data {
		syn::Data::Struct(st) => {
			match st.fields {
				syn::Fields::Named(named) => {
					for field in named.named.iter() {
						field.
					}
				},
				syn::Fields::Unnamed(_) => todo!(),
				syn::Fields::Unit => todo!(),
			}
		},
		syn::Data::Enum(en) => {
			for variant in en.variants.iter() {
				
			}
		},
		syn::Data::Union(_) => panic!("union not supported")
	}
}