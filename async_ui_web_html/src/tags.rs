use crate::elem::{Elem, HtmlTag};

macro_rules! impl_tag {
    ($elem_type:ty, $($tag:literal = $name:ident),+) => {
        paste::paste! {
            impl HtmlTag for web_sys::$elem_type {}
			$(
				pub fn [<$name:snake>]<'a>() -> Elem<'a, web_sys::$elem_type> {
					Elem::create($tag)
				}
			)+
        }
    };
}
impl_tag!(HtmlDivElement, "div" = Div);
impl_tag!(HtmlButtonElement, "button" = Button);
impl_tag!(HtmlInputElement, "input" = Input);
impl_tag!(HtmlSpanElement, "span" = Span);
impl_tag!(HtmlParagraphElement, "p" = Paragraph);
impl_tag!(HtmlAnchorElement, "a" = Anchor);
impl_tag!(HtmlElement, "main" = Main, "nav" = Nav, "section" = Section);
mod anchor;
mod input;
