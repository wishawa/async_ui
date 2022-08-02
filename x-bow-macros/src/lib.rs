mod phantom_generics;
use phantom_generics::generic_phantom_data;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::SelfType, Attribute, DataEnum, DataStruct,
    DeriveInput, Expr, ExprCall, ExprPath, ExprStruct, Field, FieldValue, Fields, FieldsNamed,
    FieldsUnnamed, GenericParam, ItemStruct, Member, Meta, NestedMeta, Path, PathSegment, Stmt,
    Token, Type, TypeGenerics, TypeParam,
};

const ATTRIBUTE_PATH: &str = "x_bow";
const ATTRIBUTE_SKIP: &str = "no_project";
#[proc_macro_derive(XBowProject, attributes(x_bow))]
pub fn derive_project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let res = match &ast.data {
        syn::Data::Struct(data) => derive_for_struct(&ast, data),
        syn::Data::Enum(data) => todo!(),
        _ => panic!("XBowProject: only structs and enums are supported"),
    };
    res.into()
}
fn get_projection_ident(input_ident: &Ident) -> Ident {
    Ident::new(
        &format!("XBowProjection_{}", input_ident.to_string()),
        Span::mixed_site(),
    )
}
fn get_edge_generic_param(my_ident: Ident, my_generics: &TypeGenerics) -> TypeParam {
    let ident = Ident::new("XBowProjectionEdge", Span::mixed_site());
    parse_quote!(
        #ident: ::x_bow::__for_macro::EdgeTrait<Data = #my_ident #my_generics>
    )
}
fn get_incoming_edge_ident() -> Ident {
    Ident::new("x_bow_projection_incoming_edge", Span::mixed_site())
}
fn derive_for_struct(ast: &DeriveInput, data: &DataStruct) -> TokenStream {
    let num_fields = data.fields.len();
    let mut field_types = Punctuated::<Field, Token![,]>::new();
    let mut field_constructors = Punctuated::<FieldValue, Token![,]>::new();
    let mut field_mappers = Vec::with_capacity(num_fields);
    let mut field_invalidates = Vec::new();

    let inp_ident = &ast.ident;
    let (inp_impl_params, inp_type_params, inp_where_clause) = ast.generics.split_for_impl();
    let mapper_phantom_data = generic_phantom_data(&ast.generics);

    let edge_generic = get_edge_generic_param(ast.ident.clone(), &inp_type_params);
    let edge_generic_ident = edge_generic.ident.clone();
    let incoming_edge = get_incoming_edge_ident();
    let is_tuple = match data.fields {
        syn::Fields::Unnamed(_) => true,
        _ => false,
    };
    for (idx, field) in data.fields.iter().enumerate() {
        let Field { vis, ty, .. } = field;
        let project_wrap_name = Expr::Path(if has_skip(&field.attrs) {
            parse_quote!(::x_bow::__for_macro::ProjectLeaf)
        } else {
            parse_quote!(::x_bow::__for_macro::ProjectPart)
        });

        let field_member = field.ident.as_ref().map_or_else(
            || Member::Unnamed(idx.into()),
            |ident| Member::Named(ident.to_owned()),
        );
        let mapper_name = Ident::new(
            &format!(
                "XBowMapper_{}_{}",
                inp_ident,
                field
                    .ident
                    .as_ref()
                    .map_or_else(|| idx.to_string(), |ident| ident.to_string())
            ),
            Span::mixed_site(),
        );

        field_invalidates.push({
            Stmt::Semi(
                Expr::Call(parse_quote! {
                    ::x_bow::__for_macro::Projection::invalidate_here_down(&self . #field_member)
                }),
                Default::default(),
            )
        });
        field_types.push({
            let mut field = field.to_owned();
            field.attrs = Vec::new();
            field.ty = Type::Path(parse_quote!(
                #project_wrap_name <#ty #inp_type_params, ::x_bow::__for_macro::Edge<#edge_generic_ident, #mapper_name #inp_type_params, #edge_generic_ident :: InEnum>>
            ));
            field
        });
        field_constructors.push({
            let member = if let Some(idt) = field.ident.as_ref() {
                Member::Named(idt.to_owned())
            } else {
                Member::Unnamed(idx.into())
            };
            FieldValue {
                attrs: Vec::new(),
                member,
                colon_token: field.colon_token,
                expr: parse_quote!(
                    ::x_bow::__for_macro::Projection::new(
                        ::std::rc::Rc::new(
                            ::x_bow::__for_macro::Edge::new(
                                ::std::clone::Clone::clone(& #incoming_edge),
                                #mapper_name (::std::marker::PhantomData)
                            )
                        )
                    )
                ),
            }
        });
        field_mappers.push(quote! {
            #vis struct #mapper_name #inp_type_params (#mapper_phantom_data);
            impl #inp_type_params ::std::clone::Clone for #mapper_name #inp_type_params {
                fn clone(&self) -> Self {
                    Self(::std::marker::PhantomData)
                }
            }
            impl #inp_impl_params ::x_bow::__for_macro::Mapper for #mapper_name #inp_type_params
            #inp_where_clause
            {
                type In = #inp_ident #inp_type_params;
                type Out = #ty;
                #[inline]
                    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> ::std::option::Option<&'d Self::Out> {
                    ::std::option::Option::Some(&input. #field_member)
                }
                #[inline]
                fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> ::std::option::Option<&'d mut Self::Out> {
                    ::std::option::Option::Some(&mut input. #field_member)
                }
            }
        });
    }
    let incoming_edge_ident = get_incoming_edge_ident();
    let (incoming_edge_field, incoming_edge_colon, incoming_edge_member) = if is_tuple {
        (None, None, Member::Unnamed(num_fields.into()))
    } else {
        (
            Some(incoming_edge_ident.clone()),
            Some(<Token![:]>::default()),
            Member::Named(incoming_edge_ident.clone()),
        )
    };
    field_types.push({
        let ty = parse_quote! (
            ::std::rc::Rc<#edge_generic_ident>
        );
        Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            ident: incoming_edge_field.clone(),
            colon_token: incoming_edge_colon,
            ty,
        }
    });
    field_constructors.push({
        let expr = Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path {
                leading_colon: None,
                segments: [PathSegment {
                    ident: incoming_edge_ident.clone(),
                    arguments: syn::PathArguments::None,
                }]
                .into_iter()
                .collect(),
            },
        });
        FieldValue {
            attrs: Vec::new(),
            member: incoming_edge_member.clone(),
            colon_token: incoming_edge_colon,
            expr,
        }
    });

    let mut modified_generics = ast.generics.clone();
    modified_generics
        .params
        .push(GenericParam::Type(edge_generic));
    let proj_ident = get_projection_ident(&ast.ident);
    let ty_out = ItemStruct {
        attrs: Vec::new(),
        vis: ast.vis.to_owned(),
        struct_token: Default::default(),
        ident: proj_ident,
        generics: modified_generics.clone(),
        semi_token: is_tuple.then_some(Default::default()),
        fields: {
            if is_tuple {
                Fields::Unnamed(FieldsUnnamed {
                    paren_token: Default::default(),
                    unnamed: field_types,
                })
            } else {
                Fields::Named(FieldsNamed {
                    brace_token: Default::default(),
                    named: field_types,
                })
            }
        },
    };

    let constructor_struct_path = Path {
        leading_colon: None,
        segments: [PathSegment {
            ident: SelfType::default().into(),
            arguments: syn::PathArguments::None,
        }]
        .into_iter()
        .collect(),
    };
    let constructor = if !is_tuple {
        Expr::Struct(ExprStruct {
            attrs: Vec::new(),
            brace_token: Default::default(),
            dot2_token: None,
            fields: field_constructors,
            rest: None,
            path: constructor_struct_path,
        })
    } else {
        Expr::Call(ExprCall {
            attrs: Vec::new(),
            paren_token: Default::default(),
            args: field_constructors.into_iter().map(|fie| fie.expr).collect(),
            func: Box::new(Expr::Path(ExprPath {
                attrs: Vec::new(),
                path: constructor_struct_path,
                qself: None,
            })),
        })
    };
    let (impl_params, type_params, where_clause) = modified_generics.split_for_impl();
    let projection_ident = get_projection_ident(inp_ident);
    quote! {
        #ty_out
        impl #impl_params ::x_bow::__for_macro::Projection for #projection_ident #type_params
        #where_clause
        {
            type Edge = #edge_generic_ident;
            fn new(#incoming_edge_ident: ::std::rc::Rc<#edge_generic_ident>) -> Self {
                #constructor
            }
            fn edge(&self) -> &::std::rc::Rc<Self::Edge> {
                &self. #incoming_edge_member
            }
            fn invalidate_here_down(&self) {
                ::x_bow::__for_macro::EdgeTrait::invalidate_here(::x_bow::__for_macro::Projection::edge(self));
                #(#field_invalidates)*
            }
        }
        impl #impl_params ::x_bow::__for_macro::Projectable<#edge_generic_ident> for #inp_ident #inp_type_params
        #where_clause
        {
            type Projection = #projection_ident #type_params;
        }
        #(#field_mappers)*
    }
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
