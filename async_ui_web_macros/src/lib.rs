mod css;
use css::css_macro;
mod select;
use select::select_macro;

/// Register CSS to be bundled and generate postfixed classnames.
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
