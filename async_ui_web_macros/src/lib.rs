mod css;
use css::css_macro;
mod select;
use select::select_macro;

/// Register CSS to be bundled and generate postfixed classnames.
// TODO: Incompatible with SSR, should there be some `let _style: StyleGuard = use_style(css_mod::STYLE)`, where `StyleGuard` maintains refcnt
// of used styles, and inserts used styles to used css style list context?
#[proc_macro]
pub fn css(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    css_macro(input)
}

/// See explanation of `select!` [here](https://rust-lang.github.io/async-book/06_multiple_futures/03_select.html).
/// Note that our variant does not support `complete` or `default` branch.
#[proc_macro]
pub fn select(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    select_macro(input)
}
