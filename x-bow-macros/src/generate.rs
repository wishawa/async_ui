use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, DeriveInput, Field, Path, Token, Variant};

use crate::{
    attributes::TrackMode,
    utils::{add_trailing_punct, get_field_member, unbracket_generics},
};

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
        ref_or_refmut: &Path,
        type_name: &Path,
    ) -> TokenStream {
        match self {
            ToGenField::Struct { field, index, .. } => {
                let member = get_field_member(&field.ident, *index);
                quote! {
                    ::core::option::Option::map(
                        #input,
                        |#input| {
                            #ref_or_refmut :: map(
                                #input,
                                |#input| & #mut_token #input . #member
                            )
                        }
                    )
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
                        #type_name :: #variant_name { #field_name, .. } => ::core::option::Option::Some(#field_name)
                    }
                } else {
                    let underscores = (0..*index).map(|_| -> Token![_] { Default::default() });
                    quote! {
                        #type_name :: #variant_name (#(#underscores,)* data, ..) => ::core::option::Option::Some(data)
                    }
                };
                quote! {
                    ::core::option::Option::and_then(
                        #input,
                        |#input| {
                            #ref_or_refmut ::filter_map(
                                #input,
                                |#input| {
                                    match #input {
                                        #match_inner,
                                        _ => ::std::option::Option::None
                                    }
                                }
                            ).ok()
                        }
                    )
                }
            }
        }
    }
}

pub(crate) fn generate(
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
    let (impl_params_unbracketed, type_params_unbracketed) = unbracket_generics(generics);

    let default_mode = TrackMode::from_attributes(attrs, &TrackMode::Shallow)?;
    let remote_name = &remote.segments.last().unwrap().ident;

    let is_struct;
    let mut filtered_fields = Vec::new();
    match data {
        syn::Data::Struct(data) => {
            is_struct = true;
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
            is_struct = false;
            for variant in data.variants.iter() {
                for (index, field) in variant.fields.iter().enumerate() {
                    match TrackMode::from_attributes(&field.attrs, &default_mode)? {
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
                data.union_token,
                "union not supported",
            ));
        }
    }
    let mut mappers = Vec::new();
    let mut mapper_build_methods = Vec::new();

    let parent_generic_param = Ident::new("TrackParent", Span::mixed_site());
    let parent_value_name = Ident::new("parent", Span::mixed_site());
    let inner_path_name = format_ident!("inner_path", span = Span::mixed_site());

    for (field_index, field) in filtered_fields.iter().enumerate() {
        let field_name = field.to_name();
        let mapper_name = format_ident!("Mapper_{remote_name}_{field_name}");
        let field_type = &field.field().ty;
        let deep = field.is_deep();
        if deep {
            where_clause
                .predicates
                .push(parse_quote! (#field_type: #prefix :: Trackable));
        }
        let input_ident = Ident::new("input", Span::mixed_site());
        let access = field.access(&input_ident, None, &parse_quote!(::std::cell::Ref), remote);
        let access_mut = field.access(
            &input_ident,
            Some(Token![mut](Span::mixed_site())),
            &parse_quote!(::std::cell::RefMut),
            remote,
        );

        let guarantee_code = if is_struct {
            Some(quote! {
                impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params> + #prefix::PathExtGuaranteed> #prefix::PathExtGuaranteed for #mapper_name <#type_params_unbracketed #parent_generic_param> #where_clause {}
            })
        } else {
            None
        };
        let field_vis = if is_struct {
            field.field().vis.clone()
        } else {
            syn::Visibility::Public(Default::default())
        };
        mappers.push(quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #mapper_name <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params>> #where_clause {
                #parent_value_name: #parent_generic_param,
            }
            impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params> + ::core::clone::Clone> ::core::clone::Clone for #mapper_name <#type_params_unbracketed #parent_generic_param> #where_clause {
                fn clone(&self) -> Self {
                    Self {
                        #parent_value_name: ::core::clone::Clone::clone(&self.#parent_value_name)
                    }
                }
            }
            impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params> + ::core::marker::Copy> ::core::marker::Copy for #mapper_name <#type_params_unbracketed #parent_generic_param> #where_clause {}
            impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params>> #prefix::Path for #mapper_name <#type_params_unbracketed #parent_generic_param> #where_clause {
                type Out = #field_type;

                fn path_borrow(&self) -> ::core::option::Option<::std::cell::Ref<'_, Self::Out>>
                {
                    let #input_ident = #prefix::Path::path_borrow(&self.#parent_value_name);
                    #access
                }

                fn path_borrow_mut(&self) -> ::core::option::Option<::std::cell::RefMut<'_, Self::Out>>
                {
                    let #input_ident = #prefix::Path::path_borrow_mut(&self.#parent_value_name);
                    #access_mut
                }

                fn visit_hashes(&self, visitor: &mut #prefix::HashVisitor) {
                    #prefix::Path::visit_hashes(&self.#parent_value_name, visitor);
                    ::std::hash::Hasher::write_usize(&mut **visitor, #field_index);
                    visitor.finish_one();
                }

                fn store_wakers(&self) -> &std::cell::RefCell<#prefix::StoreWakers> {
                    self.#parent_value_name.store_wakers()
                }
            }
            #guarantee_code
        });
        let trackable_trait: Path = if deep {
            parse_quote!(#prefix::Trackable)
        } else {
            parse_quote!(#prefix::TrackableLeaf)
        };
        mapper_build_methods.push(quote! {
            #field_vis fn #field_name (self) -> <#field_type as #trackable_trait>::PathBuilder<#mapper_name <#type_params_unbracketed #parent_generic_param>> {
                #trackable_trait::new_path_builder(#mapper_name {
                    #parent_value_name: self.#inner_path_name
                })
            }
        });
    }

    let path_builder_name = format_ident!("{name}PathBuilder", span = Span::mixed_site());

    Ok(quote! {
        #(#mappers)*

        #[derive(#prefix :: IntoInnerPath)]
        #[into_inner_path(prefix = #prefix)]
        #vis struct #path_builder_name <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params>>
        #where_clause
        {
            #inner_path_name: #parent_generic_param
        }

        impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params> + ::core::clone::Clone> ::core::clone::Clone for #path_builder_name <#type_params_unbracketed #parent_generic_param> #where_clause {
            fn clone(&self) -> Self {
                Self {
                    #inner_path_name: ::core::clone::Clone::clone(&self.#inner_path_name)
                }
            }
        }
        impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params> + ::core::marker::Copy> ::core::marker::Copy for #path_builder_name <#type_params_unbracketed #parent_generic_param> #where_clause {}

        impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params>> ::core::ops::Deref for #path_builder_name <#type_params_unbracketed #parent_generic_param> #where_clause {
            type Target = #parent_generic_param;
            fn deref(&self) -> &Self::Target {
                &self.#inner_path_name
            }
        }

        #[allow(non_snake_case)]
        impl <#impl_params_unbracketed #parent_generic_param: #prefix::Path<Out = #remote_name #type_params>> #path_builder_name <#type_params_unbracketed #parent_generic_param> #where_clause {
            #(#mapper_build_methods)*
        }

        impl #impl_params #prefix::Trackable for #remote_name #type_params #where_clause {
            type PathBuilder<#parent_generic_param: #prefix::Path<Out = Self>> = #path_builder_name <#type_params_unbracketed #parent_generic_param>;

            fn new_path_builder<#parent_generic_param: #prefix::Path<Out = Self>>(#parent_value_name: #parent_generic_param) -> Self::PathBuilder<#parent_generic_param> {
                #path_builder_name {
                    #inner_path_name: #parent_value_name,
                }
            }
        }
    })
}
