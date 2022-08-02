mod phantom_generics;
use phantom_generics::generic_phantom_data;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, DataEnum, DataStruct, DeriveInput, Field, GenericArgument, GenericParam, Meta,
    NestedMeta, TypeParam,
};

const ATTRIBUTE_PATH: &str = "x_bow";
const ATTRIBUTE_SKIP: &str = "no_project";
#[proc_macro_derive(XBowProject, attributes(x_bow))]
pub fn derive_project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = &ast;

    let projection_struct_ident = Ident::new(
        &format!("XBowProjection_{}", ident.to_string()),
        Span::mixed_site(),
    );
    match data {
        syn::Data::Struct(st) => {}
        syn::Data::Enum(en) => todo!(),
        _ => panic!("XBowProject: only structs and enums are supported"),
    }

    let proj_type_def = quote! {
        #vis struct #projection_struct_ident {

        }
    };
    todo!()
}
fn get_struct_fields(input: &DeriveInput, data: &DataStruct, gener_edge: TypeParam) -> TokenStream {
    let num_fields = data.fields.len();
    let mut field_types = Vec::with_capacity(num_fields);
    let mut field_constructors = Vec::with_capacity(num_fields);
    let mut field_mappers = Vec::with_capacity(num_fields);

    let inp_ident = &input.ident;
    let (impl_geners, ty_geners, where_clause) = input.generics.split_for_impl();

    for (idx, field) in data.fields.iter().enumerate() {
        let Field {
            attrs,
            vis,
            ident,
            colon_token,
            ty,
        } = field;
        let project_wrap_name = if has_skip(attrs) {
            "ProjectedLeaf"
        } else {
            "ProjectedPart"
        };

        let field_ident = ident
                    .as_ref()
                    .map_or_else(|| idx.to_string(), |ident| ident.to_string());
        let mapper_name = Ident::new(
            &format!(
                "XBowMapper_{}_{}",
                inp_ident,
                field_ident
            ),
            Span::mixed_site(),
        );

        field_types.push(
            quote! {
                #vis #ident #colon_token #project_wrap_name <#ty #ty_geners, Edge<#gener_edge, #mapper_name #ty_geners, #gener_edge :: InEnum>>,
            }
        );
        field_constructors.push(
            quote! {
                #ident #colon_token ::x_bow::__for_macro::Projection::new(::std::rc::Rc::new(::x_bow::__for_macro::Edge::new(::std::clone::Clone::clone(incoming_edge), #mapper_name (::std::marker::PhantomData)))),
            }
        );
        let phantom_data = generic_phantom_data(&input.generics);
        field_mappers.push(quote! {
            #vis struct #mapper_name #ty_geners (#phantom_data);
                impl #ty_geners ::std::clone::Clone for #mapper_name #ty_geners {
                    fn clone(&self) -> Self {
                        Self(::std::marker::PhantomData)
                    }
                }
                impl #impl_geners ::x_bow::__for_macro::Mapper for #mapper_name #ty_geners
                #where_clause 
                {
                    type In = #inp_ident #ty_geners;
                    type Out = #ty;
                    #[inline]
                        fn map<'s, 'd>(&'s self, input: &'d Self::In) -> ::std::option::Option<&'d Self::Out> {
                        &input. #field_ident
                    }
                    #[inline]
                    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> ::std::option::Option<&'d mut Self::Out> {
                        &mut input. #field_ident
                    }
                }
            });
    }
    todo!()
}

fn get_enum_fields(input: &DataEnum) -> TokenStream {
    todo!()
}
fn has_skip(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.is_ident(ATTRIBUTE_PATH) {
                for item in meta_list.nested.iter() {
                    if let NestedMeta::Meta(Meta::Path(path)) = item {
                        if path.is_ident(ATTRIBUTE_SKIP) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
