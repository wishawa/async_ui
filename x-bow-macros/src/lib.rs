mod phantom_generics;
use phantom_generics::generic_phantom_data;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::SelfType, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, Expr, ExprCall, ExprField, ExprPath, ExprStruct, Field, FieldPat, FieldValue,
    Fields, FieldsNamed, FieldsUnnamed, GenericParam, ItemStruct, Member, Meta, NestedMeta, Pat,
    PatIdent, PatRest, PatStruct, PatTuple, PatTupleStruct, PatWild, Path, PathSegment, Stmt,
    Token, Type, TypeGenerics, TypeParam, Variant, VisPublic,
};

const ATTRIBUTE_PATH: &str = "x_bow";
const ATTRIBUTE_SKIP: &str = "no_track";
#[proc_macro_derive(Track, attributes(x_bow))]
pub fn derive_project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let res = derive_main(&ast);
    res.into()
}
fn get_projection_ident(input_ident: &Ident) -> Ident {
    Ident::new(
        &format!("XBowTracked_{}", input_ident.to_string()),
        Span::mixed_site(),
    )
}
fn get_edge_generic_param(my_ident: Ident, my_generics: &TypeGenerics) -> TypeParam {
    let ident = Ident::new("XBowTrackedEdge", Span::mixed_site());
    parse_quote!(
        #ident: ::x_bow::__for_macro::EdgeTrait<Data = #my_ident #my_generics>
    )
}
fn get_incoming_edge_ident() -> Ident {
    Ident::new("x_bow_tracked_incoming_edge", Span::mixed_site())
}
fn derive_main(ast: &DeriveInput) -> TokenStream {
    let data = &ast.data;
    let num_fields = match data {
        Data::Struct(data) => data.fields.len(),
        Data::Enum(data) => data.variants.len(),
        _ => panic!("x-bow: Track: only structs and enums are supported"),
    };
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
    let (is_enum, is_tuple) = match data {
        Data::Struct(data) => (
            false,
            match data.fields {
                syn::Fields::Unnamed(_) => true,
                _ => false,
            },
        ),
        _ => (true, false),
    };
    // for (idx, field) in data.fields.iter().enumerate() {
    let mut for_each_field =
        |idx: usize, field: &Field, variant_info: Option<(&Variant, &Field, usize)>| {
            let Field { vis, ty, .. } = field;
            let project_wrap_name = Expr::Path(if has_skip(&field.attrs) {
                parse_quote!(::x_bow::__for_macro::TrackedLeaf)
            } else {
                parse_quote!(::x_bow::__for_macro::TrackedPart)
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
                        ::x_bow::__for_macro::Tracked::invalidate_here_down(&self . #field_member)
                    }),
                    Default::default(),
                )
            });
            field_types.push({
            let mut field = field.to_owned();
            field.attrs = Vec::new();
            let optional_path: Path = if is_enum {
                parse_quote! {
                    ::x_bow::__for_macro::OptionalYes
                }
            }
            else {
                parse_quote! {
                    #edge_generic_ident :: Optional
                }
            };
            field.ty = Type::Path(parse_quote!(
                #project_wrap_name <#ty, ::x_bow::__for_macro::Edge<#edge_generic_ident, #mapper_name #inp_type_params, #optional_path>>
            ));
            field
        });
            field_constructors.push({
                FieldValue {
                    attrs: Vec::new(),
                    member: field_member.clone(),
                    colon_token: field.colon_token,
                    expr: parse_quote!(
                        ::x_bow::__for_macro::Tracked::new(
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
            field_mappers.push({
            let input_ident = Ident::new("map_input", Span::mixed_site());
            let (map_expr, map_mut_expr): (Expr, Expr) = if let Some((variant, vf, vf_idx)) = variant_info {
                let variant_name = &variant.ident;
                let value_ident = Ident::new("map_variant_value", Span::mixed_site());
                let pat_ident = Pat::Ident(PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            subpat: None,
                            ident: value_ident.clone()
                        });
                        let variant_path = parse_quote! (Self::In::#variant_name);
                let pattern: Pat = match &variant.fields {
                    Fields::Named(fields) => {
                        let vf_name = vf.ident.as_ref().unwrap();
                        Pat::Struct(PatStruct {
                            attrs: Vec::new(),
                            brace_token: fields.brace_token.to_owned(),
                            dot2_token: Some(Default::default()),
                            path: variant_path,
                            fields: [FieldPat {
                                attrs: Vec::new(),
                                colon_token: Some(Default::default()),
                                member: Member::Named(vf_name.to_owned()),
                                pat: Box::new(pat_ident)
                            }].into_iter().collect()
                        })
                    },
                    Fields::Unnamed(_) => {
                        let mut receiver: Punctuated::<Pat, Token![,]> = (0..vf_idx).map(|_| Pat::Wild(PatWild {
                            attrs: Vec::new(),
                            underscore_token: Default::default()
                        })).collect();
                        receiver.push(pat_ident);
                        receiver.push(Pat::Rest(PatRest {
                            attrs: Vec::new(),
                            dot2_token: Default::default()
                        }));
                        Pat::TupleStruct(PatTupleStruct {
                            attrs: Vec::new(),
                            path: variant_path,
                            pat: PatTuple {
                                attrs: Vec::new(),
                                elems: receiver,
                                paren_token: Default::default()
                            }
                        })
                    },
                    _ => unreachable!()
                };
                let out: Expr = parse_quote! (
                    match #input_ident {
                        #pattern => ::std::option::Option::Some(#value_ident),
                        _ => None
                    }
                );
                (out.clone(), out)
            } else {
                let access: ExprField = parse_quote! {
                    #input_ident. #field_member
                };
                (
                    parse_quote! (
                        ::std::option::Option::Some(& #access)
                    ),
                    parse_quote! (
                        ::std::option::Option::Some(&mut #access)
                    ),
                )
            };
            quote! {
                #vis struct #mapper_name #inp_type_params (#mapper_phantom_data);
                impl #inp_type_params ::std::clone::Clone for #mapper_name #inp_type_params {
                    #[inline]
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
                    fn map<'s, 'd>(&'s self, #input_ident: &'d Self::In) -> ::std::option::Option<&'d Self::Out> {
                        #map_expr
                    }
                    #[inline]
                    fn map_mut<'s, 'd>(&'s self, #input_ident: &'d mut Self::In) -> ::std::option::Option<&'d mut Self::Out> {
                        #map_mut_expr
                    }
                }
            }
        });
        };
    match data {
        Data::Struct(data) => {
            data.fields
                .iter()
                .enumerate()
                .for_each(|(idx, field)| for_each_field(idx, field, None));
        }
        Data::Enum(data) => {
            data.variants.iter().enumerate().for_each(|(idx, variant)| {
                let variant_name = variant.ident.clone();
                let for_each_variant_field = |(variant_idx, variant_field): (usize, &Field)| {
                    let variant_field_member = variant_field
                        .ident
                        .as_ref()
                        .map_or_else(|| variant_idx.to_string(), |n| n.to_string());
                    let field = Field {
                        attrs: variant_field.attrs.clone(),
                        colon_token: Some(Default::default()),
                        ident: Some(Ident::new(
                            &format!("{variant_name}_{variant_field_member}"),
                            Span::mixed_site(),
                        )),
                        ty: variant_field.ty.clone(),
                        vis: syn::Visibility::Public(VisPublic {
                            pub_token: Default::default(),
                        }),
                    };
                    for_each_field(idx, &field, Some((variant, &variant_field, variant_idx)));
                };
                match &variant.fields {
                    Fields::Named(fields) => {
                        fields
                            .named
                            .iter()
                            .enumerate()
                            .for_each(for_each_variant_field);
                    }
                    Fields::Unnamed(fields) => {
                        fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .for_each(for_each_variant_field);
                    }
                    Fields::Unit => {}
                }
            });
        }
        _ => unreachable!(),
    };
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
        impl #impl_params ::x_bow::__for_macro::Tracked for #projection_ident #type_params
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
                ::x_bow::__for_macro::EdgeTrait::invalidate_here(::x_bow::__for_macro::Tracked::edge(self));
                #(#field_invalidates)*
            }
        }
        impl #impl_params ::x_bow::__for_macro::Trackable<#edge_generic_ident> for #inp_ident #inp_type_params
        #where_clause
        {
            type Tracked = #projection_ident #type_params;
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
