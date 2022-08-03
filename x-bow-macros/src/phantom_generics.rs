use syn::{parse_quote, GenericParam, Generics, Type, TypePath};

pub fn generic_phantom_data(generics: &Generics) -> Type {
    let args = generics.params.iter().filter_map(|param| match param {
        GenericParam::Lifetime(gen_lt) => Some(GenericParam::Type(parse_quote!(
            & #gen_lt ()
        ))),
        GenericParam::Type(gen_ty) => {
            let mut gen_ty = gen_ty.to_owned();
            gen_ty.attrs = Default::default();
            gen_ty.bounds = Default::default();
            Some(GenericParam::Type(gen_ty))
        }
        _ => None,
    });
    let path = parse_quote! (
        ::std::marker::PhantomData<(#(#args),*)>
    );
    let path = TypePath { qself: None, path };
    Type::Path(path)
}
