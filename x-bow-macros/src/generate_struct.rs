use proc_macro2::Ident;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::DeriveInput;
use syn::Field;

use proc_macro2::TokenStream;

use syn::Lifetime;
use syn::Path;

use syn;

use syn::parse_quote;
use syn::Token;
use syn::Variant;

use crate::attributes::TrackMode;
use crate::utils::add_trailing_punct;
use crate::utils::get_field_member;

enum ToGenField<'a> {
    Struct {
        field: &'a Field,
        index: usize,
        deep: bool,
    },
    Enum {
        variant: &'a Variant,
        field: &'a Field,
        index: usize,
        deep: bool,
    },
}
impl<'a> ToGenField<'a> {
    fn to_name(&self) -> Ident {
        match self {
            Self::Struct { field, index, .. } => match &field.ident {
                Some(ident) => ident.to_owned(),
                None => format_ident!("t{index}"),
            },
            Self::Enum {
                variant,
                field,
                index,
                ..
            } => match &field.ident {
                Some(ident) => format_ident!("{}_{}", variant.ident, ident),
                None => format_ident!("{}_{}", variant.ident, index),
            },
        }
    }
    fn is_deep(&self) -> bool {
        *match self {
            ToGenField::Struct { deep, .. } => deep,
            ToGenField::Enum { deep, .. } => deep,
        }
    }
    fn field(&self) -> &Field {
        match self {
            ToGenField::Struct { field, .. } => field,
            ToGenField::Enum { field, .. } => field,
        }
    }
    fn access(
        &self,
        input: &Ident,
        mut_token: Option<Token![mut]>,
        type_name: &Path,
    ) -> TokenStream {
        match self {
            ToGenField::Struct { field, index, .. } => {
                let member = get_field_member(&field.ident, *index);
                quote! {
                    Some(& #mut_token #input . #member)
                }
            }
            ToGenField::Enum {
                variant,
                field,
                index,
                ..
            } => {
                let variant_name = &variant.ident;
                let match_inner = if let Some(field_name) = &field.ident {
                    quote! {
                        #type_name :: #variant_name { #field_name, .. } => ::std::option::Option::Some(#field_name)
                    }
                } else {
                    let underscores = (0..*index).map(|_| -> Token![_] { Default::default() });
                    quote! {
                        #type_name :: #variant_name (#(#underscores,)* data, ..) => ::std::option::Option::Some(data)
                    }
                };
                quote! {
                    match #input {
                        #match_inner,
                        _ => ::std::option::Option::None
                    }
                }
            }
        }
    }
}

pub(crate) fn generate_struct(
    input: &DeriveInput,
    remote: &Path,
    prefix: &Path,
) -> syn::Result<TokenStream> {
    let DeriveInput {
        attrs,
        vis,
        ident: name,
        generics,
        data,
    } = input;
    let (impl_params, type_params, where_clause) = generics.split_for_impl();
    let mut all_params_unbracketed = generics.params.clone();
    add_trailing_punct(&mut all_params_unbracketed);
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    let (impl_params_unbracketed, type_params_unbracketed) =
        crate::utils::unbracket_generics(generics);
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut generic_params = Vec::new();
    let mut mapper_names = Vec::new();
    let mut mappers = Vec::new();
    let mut trackable_down_trait = Vec::<Path>::new();
    let remote_name = &remote.segments.last().unwrap().ident;
    let default_mode = TrackMode::from_attributes(&*attrs, &TrackMode::Shallow)?;
    let mut filtered_fields = Vec::new();
    let is_guaranteed;
    match data {
        syn::Data::Struct(data) => {
            is_guaranteed = true;
            for (index, field) in data.fields.iter().enumerate() {
                match TrackMode::from_attributes(&field.attrs, &default_mode)? {
                    x @ TrackMode::Deep | x @ TrackMode::Shallow => {
                        filtered_fields.push(ToGenField::Struct {
                            field,
                            index,
                            deep: matches!(x, TrackMode::Deep),
                        });
                    }
                    TrackMode::Skip => {}
                }
            }
        }
        syn::Data::Enum(data) => {
            is_guaranteed = false;
            for variant in data.variants.iter() {
                for (index, field) in variant.fields.iter().enumerate() {
                    match TrackMode::from_attributes(&*field.attrs, &default_mode)? {
                        x @ TrackMode::Deep | x @ TrackMode::Shallow => {
                            filtered_fields.push(ToGenField::Enum {
                                variant,
                                field,
                                index,
                                deep: matches!(x, TrackMode::Deep),
                            });
                        }
                        TrackMode::Skip => {}
                    }
                }
            }
        }
        syn::Data::Union(data) => {
            return Err(syn::Error::new_spanned(
                &data.union_token,
                "union not supported",
            ));
        }
    }

    for field in filtered_fields.iter() {
        let field_name = field.to_name();
        let param_name = format_ident!("FieldType_{field_name}");
        let mapper_name = format_ident!("Mapper_{remote_name}_{field_name}");
        let field_type = &field.field().ty;
        if field.is_deep() {
            where_clause
                .predicates
                .push(parse_quote! (#field_type: #prefix :: Trackable));
        }
        let input_ident = Ident::new("input", Span::mixed_site());
        let access = field.access(&input_ident, None, remote);
        let access_mut = field.access(&input_ident, Some(Token![mut](Span::mixed_site())), remote);
        mappers.push(quote! {
            #[doc(hidden)]
            #vis struct #mapper_name #generics (pub ::std::marker::PhantomData<(#type_params_unbracketed)>);
            impl #impl_params #prefix :: Mapper for #mapper_name #type_params #where_clause {
                type In = #remote #type_params;
                type Out = #field_type;
                fn map <'s, 'd>(&'s self, #input_ident: &'d Self::In) -> Option<&'d Self::Out> {
                    #access
                }
                fn map_mut <'s, 'd>(&'s self, #input_ident: &'d mut Self::In) -> Option<&'d mut Self::Out> {
                    #access_mut
                }
            }
        });
        generic_params.push(param_name);
        field_names.push(field_name);
        field_types.push(field_type);
        mapper_names.push(mapper_name);
        trackable_down_trait.push(if field.is_deep() {
            parse_quote!(#prefix :: Trackable)
        } else {
            parse_quote!(#prefix :: TrackableLeaf)
        });
    }
    let mut where_clause_for_tracker = where_clause.clone();
    let store_lifetime = Lifetime::new("'xbow", Span::mixed_site());
    for field in filtered_fields.iter() {
        let field_type = &field.field().ty;
        where_clause_for_tracker
            .predicates
            .push(parse_quote! (#field_type: #store_lifetime));
    }
    let tracker_name = format_ident!("Tracked_{name}", span = Span::mixed_site());
    let up_node_name = format_ident!("__tracking_internals", span = Span::mixed_site());
    let guaranteed_name = format_ident!("GUARANTEED", span = Span::mixed_site());
    let shared_name = format_ident!("shared", span = Span::mixed_site());
    let up_generic_name = format_ident!("Up", span = Span::mixed_site());
    let guaranteed_children: syn::Expr = if is_guaranteed {
        parse_quote!(#guaranteed_name)
    } else {
        parse_quote!(false)
    };
    let out = quote! {
        #(#mappers)*

        #[doc(hidden)]
        #[allow(non_snake_case)]
        #vis struct #tracker_name <#store_lifetime, #all_params_unbracketed const #guaranteed_name: #prefix :: bool> #where_clause_for_tracker {
            #(
                pub #field_names: <#field_types as #trackable_down_trait> :: NodeDown <#store_lifetime, #guaranteed_children>,
            )*
            #[doc(hidden)]
            #up_node_name: & #store_lifetime (dyn #prefix :: NodeUpTrait<Data = #remote #type_params> + #store_lifetime)
        }
        impl <#store_lifetime, #impl_params_unbracketed const #guaranteed_name: #prefix :: bool> #prefix :: IsGuaranteed<#guaranteed_name> for #tracker_name <#store_lifetime, #type_params_unbracketed #guaranteed_name> #where_clause_for_tracker {}

        impl <#store_lifetime, #impl_params_unbracketed const #guaranteed_name: #prefix :: bool> #prefix :: NodeDownTrait<#store_lifetime, #remote #type_params> for #tracker_name <#store_lifetime, #type_params_unbracketed #guaranteed_name> #where_clause_for_tracker {
            fn invalidate_downward(&self) {
                #(
                    self . #field_names . node_up() . invalidate_downward();
                    self . #field_names . invalidate_downward();
                )*
            }
            fn node_up(&self) -> & #store_lifetime (dyn #prefix :: NodeUpTrait <Data = #remote #type_params> + #store_lifetime) {
                self . #up_node_name
            }
        }

        impl #impl_params #prefix :: Trackable for #remote #type_params #where_clause {
            type NodeDown<#store_lifetime, const #guaranteed_name: #prefix :: bool> = #tracker_name <#store_lifetime, #type_params_unbracketed #guaranteed_name> where Self: #store_lifetime;

            fn new_node<#store_lifetime, #up_generic_name: #prefix :: NodeUpTrait<Data = Self>, const #guaranteed_name: #prefix :: bool>(#shared_name: & #store_lifetime #prefix :: Shared, #up_node_name: & #store_lifetime #up_generic_name) -> Self::NodeDown<#store_lifetime, #guaranteed_name> where Self: #store_lifetime {
                #tracker_name {
                    #(
                        #field_names: <#field_types as #trackable_down_trait> :: new_node(#shared_name, #shared_name . allocator . alloc(#prefix :: NodeUp :: new(#shared_name, #up_node_name.clone(), #mapper_names (::std::marker::PhantomData)))),
                    )*
                    #up_node_name,
                }
            }
        }
    };
    Ok(out)
}
