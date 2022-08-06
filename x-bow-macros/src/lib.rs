mod phantom_generics;
use phantom_generics::generic_phantom_data;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::SelfType, Attribute, Data, DeriveInput, Expr,
    ExprAssign, ExprCall, ExprField, ExprPath, ExprStruct, Field, FieldPat, FieldValue, Fields,
    FieldsNamed, FieldsUnnamed, GenericParam, ItemStruct, Member, Meta, NestedMeta, Pat, PatIdent,
    PatRest, PatStruct, PatTuple, PatTupleStruct, PatWild, Path, PathSegment, PredicateType, Stmt,
    Token, TraitBound, TraitBoundModifier, Type, TypeGenerics, TypeParam, TypeParamBound, Variant,
    VisPublic, WhereClause, WherePredicate,
};

const ATTRIBUTE_PATH: &str = "x_bow";
const ATTRIBUTE_SKIP: &str = "no_track";
const ATTRIBUTE_MODULE_PREFIX: &str = "module_prefix";
const ATTRIBUTE_REMOTE_TYPE: &str = "remote_type";
#[proc_macro_derive(Track, attributes(x_bow))]
pub fn derive_project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let mut remote_type = None;
    let mut prefix_path = None;
    ast.attrs.iter().for_each(|attr| {
        if let Ok(ExprAssign { left, right, .. }) = attr.parse_args() {
            if let (Expr::Path(left), Expr::Path(right)) = (&*left, &*right) {
                if left.path.is_ident(ATTRIBUTE_MODULE_PREFIX) {
                    prefix_path = Some(right.path.clone());
                }
                if left.path.is_ident(ATTRIBUTE_REMOTE_TYPE) {
                    remote_type = right.path.segments.last().map(|seg| seg.ident.clone());
                }
            }
        }
    });
    let prefix_path = prefix_path.unwrap_or_else(|| parse_quote!(::x_bow::__private_macro_only));
    let res = derive_main(&ast, &prefix_path, remote_type);
    res.into()
}
fn get_projection_ident(input_ident: &Ident) -> Ident {
    Ident::new(
        &format!("XBowTracked_{}", input_ident.to_string()),
        Span::mixed_site(),
    )
}
fn get_edge_generic_param(
    my_ident: Ident,
    my_generics: &TypeGenerics,
    module_prefix: &Path,
) -> TypeParam {
    let ident = Ident::new("XBowTrackedEdge", Span::mixed_site());
    parse_quote!(
        #ident: #module_prefix::EdgeTrait<Data = #my_ident #my_generics>
    )
}
fn get_incoming_edge_ident() -> Ident {
    Ident::new("x_bow_tracked_incoming_edge", Span::mixed_site())
}
fn derive_main(
    ast: &DeriveInput,
    module_prefix: &Path,
    on_remote_type: Option<Ident>,
) -> TokenStream {
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

    let target_ident = on_remote_type.as_ref().unwrap_or(&ast.ident);
    let (inp_impl_params, inp_type_params, inp_where_clause) = ast.generics.split_for_impl();
    let mapper_phantom_data = generic_phantom_data(&ast.generics);

    let edge_generic =
        get_edge_generic_param(target_ident.clone(), &inp_type_params, module_prefix);
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
    let mut proj_constraints: Vec<WherePredicate> = Vec::new();
    let mut for_each_field =
        |idx: usize, field: &Field, variant_info: Option<(&Variant, &Field, usize)>| {
            let Field { vis, ty, .. } = field;
            let skip = has_skip(&field.attrs);
            let field_member = field.ident.as_ref().map_or_else(
                || Member::Unnamed(idx.into()),
                |ident| Member::Named(ident.to_owned()),
            );
            let mapper_name = Ident::new(
                &format!(
                    "XBowMapper_{}_{}",
                    target_ident,
                    field
                        .ident
                        .as_ref()
                        .map_or_else(|| idx.to_string(), |ident| ident.to_string())
                ),
                Span::mixed_site(),
            );

            field_invalidates.push(Stmt::Semi(
                Expr::Call(parse_quote! {
                    #module_prefix::TrackedNode::invalidate_outside_down(&* self . #field_member)
                }),
                Default::default(),
            ));
            {
                let mut field = field.to_owned();
                field.attrs = Vec::new();
                let optional_path: Path = if is_enum {
                    parse_quote! {
                        #module_prefix::OptionalYes
                    }
                } else {
                    parse_quote! {
                        #edge_generic_ident :: Optional
                    }
                };
                let add_edge: Path = parse_quote! (
                    #module_prefix::Edge<#edge_generic_ident, #mapper_name #inp_type_params, #optional_path>
                );
                let tracked_node_ty: Type = if skip {
                    parse_quote! (
                        #module_prefix::XBowLeaf<#ty, #add_edge>
                    )
                } else {
                    parse_quote! (
                        #module_prefix::TrackedNodeAlias<#ty, #add_edge>
                    )
                };
                field.ty = Type::Path(parse_quote!(
                    #module_prefix::Tracked<#tracked_node_ty>
                ));
                field_types.push(field);
                if !skip {
                    proj_constraints.push(WherePredicate::Type(PredicateType {
                        bounded_ty: ty.clone(),
                        bounds: [TypeParamBound::Trait(TraitBound {
                            lifetimes: None,
                            paren_token: None,
                            modifier: TraitBoundModifier::None,
                            path: parse_quote! (
                                #module_prefix::Trackable<#add_edge>
                            ),
                        })]
                        .into_iter()
                        .collect(),
                        colon_token: Default::default(),
                        lifetimes: None,
                    }));
                }
            }
            field_constructors.push({
                FieldValue {
                    attrs: Vec::new(),
                    member: field_member.clone(),
                    colon_token: field.colon_token,
                    expr: parse_quote!(
                        #module_prefix::Tracked::new(
                            ::std::rc::Rc::new(
                                #module_prefix::Edge::new(
                                    ::std::clone::Clone::clone(& #incoming_edge),
                                    #mapper_name (::std::marker::PhantomData)
                                )
                            )
                        )
                    ),
                }
            });
            {
                let input_ident = Ident::new("map_input", Span::mixed_site());
                let (map_expr, map_mut_expr): (Expr, Expr) =
                    if let Some((variant, vf, vf_idx)) = variant_info {
                        let variant_name = &variant.ident;
                        let value_ident = Ident::new("map_variant_value", Span::mixed_site());
                        let pat_ident = Pat::Ident(PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            subpat: None,
                            ident: value_ident.clone(),
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
                                        pat: Box::new(pat_ident),
                                    }]
                                    .into_iter()
                                    .collect(),
                                })
                            }
                            Fields::Unnamed(_) => {
                                let mut receiver: Punctuated<Pat, Token![,]> = (0..vf_idx)
                                    .map(|_| {
                                        Pat::Wild(PatWild {
                                            attrs: Vec::new(),
                                            underscore_token: Default::default(),
                                        })
                                    })
                                    .collect();
                                receiver.push(pat_ident);
                                receiver.push(Pat::Rest(PatRest {
                                    attrs: Vec::new(),
                                    dot2_token: Default::default(),
                                }));
                                Pat::TupleStruct(PatTupleStruct {
                                    attrs: Vec::new(),
                                    path: variant_path,
                                    pat: PatTuple {
                                        attrs: Vec::new(),
                                        elems: receiver,
                                        paren_token: Default::default(),
                                    },
                                })
                            }
                            _ => unreachable!(),
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
                field_mappers.push(quote! {
                    #vis struct #mapper_name #inp_type_params (#mapper_phantom_data);
                    impl #inp_type_params ::std::clone::Clone for #mapper_name #inp_type_params {
                        #[inline]
                        fn clone(&self) -> Self {
                            Self(::std::marker::PhantomData)
                        }
                    }
                    impl #inp_impl_params #module_prefix::Mapper for #mapper_name #inp_type_params
                    #inp_where_clause
                    {
                        type In = #target_ident #inp_type_params;
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
                });
            }
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
                let fields = match &variant.fields {
                    Fields::Named(fields) => Some(&fields.named),
                    Fields::Unnamed(fields) => Some(&fields.unnamed),
                    Fields::Unit => None,
                };
                if let Some(fields) = fields {
                    let total_fields = fields.len();
                    fields
                        .iter()
                        .enumerate()
                        .for_each(|(variant_field_idx, variant_field)| {
                            let variant_field_member = variant_field.ident.as_ref().map_or_else(
                                || {
                                    if total_fields > 1 {
                                        Ident::new(
                                            &format!("{variant_name}_{variant_field_idx}"),
                                            Span::mixed_site(),
                                        )
                                    } else {
                                        variant_name.clone()
                                    }
                                },
                                |n| Ident::new(&format!("{variant_name}_{n}"), Span::mixed_site()),
                            );
                            let field = Field {
                                attrs: variant_field.attrs.clone(),
                                colon_token: Some(Default::default()),
                                ident: Some(variant_field_member),
                                ty: variant_field.ty.clone(),
                                vis: syn::Visibility::Public(VisPublic {
                                    pub_token: Default::default(),
                                }),
                            };
                            for_each_field(
                                idx,
                                &field,
                                Some((variant, &variant_field, variant_field_idx)),
                            );
                        });
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
    let wc = modified_generics
        .where_clause
        .get_or_insert_with(|| WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
    wc.predicates.extend(proj_constraints);

    let proj_ident = get_projection_ident(target_ident);
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
    let projection_ident = get_projection_ident(target_ident);
    quote! {
        #[allow(non_snake_case)]
        #ty_out
        impl #impl_params #module_prefix::TrackedNode for #projection_ident #type_params
        #where_clause
        {
            type Edge = #edge_generic_ident;
            fn new(#incoming_edge_ident: ::std::rc::Rc<#edge_generic_ident>) -> Self {
                #constructor
            }
            fn edge(&self) -> &::std::rc::Rc<Self::Edge> {
                &self. #incoming_edge_member
            }
            fn invalidate_outside_down(&self) {
                #module_prefix::EdgeTrait::invalidate_outside_here(#module_prefix::TrackedNode::edge(self));
                #(#field_invalidates)*
            }
        }
        impl #impl_params #module_prefix::Trackable<#edge_generic_ident> for #target_ident #inp_type_params
        #where_clause
        {
            type TrackedNode = #projection_ident #type_params;
        }
        #(#field_mappers)*
    }
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
