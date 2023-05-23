mod css;
use css::css_macro;
mod select;
use select::select_macro;

#[proc_macro]
pub fn css(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    css_macro(input)
}

#[proc_macro]
pub fn select(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    select_macro(input)
}
