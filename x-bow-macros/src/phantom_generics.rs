use syn::{Generics, Type, TypePath, parse_quote};

pub fn generic_phantom_data(generics: &Generics) -> Type {
    let args = generics.params.iter().map(|param| {
		match param {
        syn::GenericParam::Lifetime(gen_lt) => syn::GenericParam::Type(parse_quote!(
			& #gen_lt ()
		)),
        _ => param.to_owned()
    }});
    let path = parse_quote! (
        ::std::marker::PhantomData<(#(#args),*)>
    );
    let path = TypePath { qself: None, path };
    Type::Path(path)
}
