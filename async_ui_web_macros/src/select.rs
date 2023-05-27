use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{braced, parse::Parse, parse_macro_input, Expr, Pat, Token};

pub fn select_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as MacroInput).generate().into()
}

struct MacroInput {
    branches: Vec<Branch>,
}

#[derive(Default)]
struct Branch {
    pattern: Option<Pat>,
    future: Option<Expr>,
    expr: Option<TokenStream>,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        fn parse_one_branch(
            input: syn::parse::ParseStream,
            branch: &mut Branch,
        ) -> syn::Result<()> {
            branch.pattern = Some(Pat::parse_single(input)?);
            input.parse::<Token![=]>()?;
            branch.future = Some(input.parse()?);
            input.parse::<Token![=>]>()?;
            let is_block = input.peek(syn::token::Brace);
            branch.expr = Some(if is_block {
                let (tt, _next) = input.cursor().token_tree().unwrap();
                let content;
                braced!(content in input);
                let _ = syn::Block::parse_within(&content);
                tt.into_token_stream()
            } else {
                input.parse::<Expr>()?.into_token_stream()
            });
            if (is_block || input.peek(Token![,])) && !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
            Ok(())
        }
        let mut branches = Vec::new();
        while !input.is_empty() {
            let mut branch = Branch::default();
            parse_one_branch(input, &mut branch).ok();
            if branch.pattern.is_none() {
                break;
            }
            branches.push(branch);
        }
        Ok(Self { branches })
    }
}
impl MacroInput {
    fn generate(self) -> TokenStream {
        let Self { branches } = self;
        let enum_name = Ident::new("SelectResult", Span::mixed_site());
        let enum_variants = (0..branches.len())
            .map(|i| format_ident!("{enum_name}{i}"))
            .collect::<Vec<_>>();
        let exprs = branches.iter().map(|br| &br.expr);
        let futures = branches.iter().map(|br| &br.future);
        let patterns = branches.iter().map(|br| &br.pattern);
        quote!(
            {
                enum #enum_name <#(#enum_variants),*> {
                    #(#enum_variants (#enum_variants)),*
                }
                match ::async_ui_web::race((
                    #( async { #enum_name :: #enum_variants (#futures.await) },)*
                )).await {
                    #(
                        #enum_name :: #enum_variants ( #patterns ) => #exprs,
                    )*
                }
            }

        )
    }
}
