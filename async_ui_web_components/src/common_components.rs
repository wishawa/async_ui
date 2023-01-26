use crate::{IsHtmlElement, WrappedHtmlElement};
macro_rules! make_component_impl {
    ($ty:ident, $tag_name:literal, $elem_ty:ty) => {
        impl IsHtmlElement for $elem_ty {
            const TAG_NAME: &'static str = $tag_name;
        }
        pub type $ty = WrappedHtmlElement<$elem_ty>;
    };
}
make_component_impl!(Div, "div", web_sys::HtmlDivElement);
make_component_impl!(Button, "button", web_sys::HtmlButtonElement);
