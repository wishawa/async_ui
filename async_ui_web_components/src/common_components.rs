use std::{
    borrow::Cow,
    future::{pending, Future, Pending},
    ops::Deref,
};

use async_ui_web_core::{window::DOCUMENT, ContainerNodeFuture};
use wasm_bindgen::prelude::{JsCast, UnwrapThrowExt};

macro_rules! component_impl {
    ($ty:ident, $tag_name:literal, $elem_ty:ty, $link:tt) => {
        #[doc = "The HTML `"]
        #[doc = $tag_name]
        #[doc = "` tag."]
        #[doc = "See"]
        #[doc = $link]
        #[doc = "."]
        pub struct $ty {
            pub element: $elem_ty,
        }
        impl $ty {
            pub fn new() -> Self {
                Self {
                    element: create_element($tag_name),
                }
            }
        }
        impl Default for $ty {
            fn default() -> Self {
                Self::new()
            }
        }
        impl Deref for $ty {
            type Target = $elem_ty;
            fn deref(&self) -> &Self::Target {
                &self.element
            }
        }
    };
    ($ty:ident, $tag_name:literal, $elem_ty:ty, $link:tt, childed) => {
        component_impl!($ty, $tag_name, $elem_ty, $link);
        impl $ty {
            pub fn render<F: Future>(&self, c: F) -> ContainerNodeFuture<F> {
                ContainerNodeFuture::new(c, AsRef::<web_sys::Node>::as_ref(&self.element).clone())
            }
        }
    };
    ($ty:ident, $tag_name:literal, $elem_ty:ty, $link:tt, childless) => {
        component_impl!($ty, $tag_name, $elem_ty, $link);
        impl $ty {
            pub fn render(&self) -> ContainerNodeFuture<Pending<()>> {
                ContainerNodeFuture::new(
                    pending(),
                    AsRef::<web_sys::Node>::as_ref(&self.element).clone(),
                )
            }
        }
    };
}

#[rustfmt::skip]
mod impls {
    use super::*;
    component_impl!(Anchor, "a", web_sys::HtmlAnchorElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)", childed);
    component_impl!(Area, "area", web_sys::HtmlAreaElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area)", childless);
    component_impl!(Audio, "audio", web_sys::HtmlAudioElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio)", childed);
    component_impl!(Bold, "b", web_sys::HtmlBrElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b)", childless);
    component_impl!(Br, "br", web_sys::HtmlBrElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br)", childless);
    component_impl!(Base, "base", web_sys::HtmlBaseElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base)", childless);
    component_impl!(Button, "button", web_sys::HtmlButtonElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button)", childed);
    component_impl!(Canvas, "canvas", web_sys::HtmlCanvasElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas)", childed);
    component_impl!(Dl, "dl", web_sys::HtmlDListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl)", childed);
    component_impl!(Data, "data", web_sys::HtmlDataElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/data)", childed);
    component_impl!(DataList, "datalist", web_sys::HtmlDataListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist)", childed);
    component_impl!(Dialog, "dialog", web_sys::HtmlDialogElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dialog)", childed);
    component_impl!(Div, "div", web_sys::HtmlDivElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div)", childed);
    component_impl!(Embed, "embed", web_sys::HtmlEmbedElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/embed)", childless);
    component_impl!(FieldSet, "fieldset", web_sys::HtmlFieldSetElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset)", childed);
    component_impl!(Form, "form", web_sys::HtmlFormElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form)", childed);
    component_impl!(FrameSet, "frameset", web_sys::HtmlFrameSetElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/frameset)", childed);
    component_impl!(Hr, "hr", web_sys::HtmlHrElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)", childless);
    component_impl!(H1, "h1", web_sys::HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1)", childed);
    component_impl!(H2, "h2", web_sys::HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h2)", childed);
    component_impl!(H3, "h3", web_sys::HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h3)", childed);
    component_impl!(H4, "h4", web_sys::HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h4)", childed);
    component_impl!(H5, "h5", web_sys::HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h5)", childed);
    component_impl!(H6, "h6", web_sys::HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h6)", childed);
    component_impl!(Italic, "i", web_sys::HtmlImageElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i)", childed);
    component_impl!(IFrame, "iframe", web_sys::HtmlIFrameElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe)", childed);
    component_impl!(Img, "img", web_sys::HtmlImageElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img)", childed);
    component_impl!(Input, "input", web_sys::HtmlInputElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input)", childless);
    component_impl!(Li, "li", web_sys::HtmlLiElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)", childed);
    component_impl!(Label, "label", web_sys::HtmlLabelElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label)", childed);
    component_impl!(Legend, "legend", web_sys::HtmlLegendElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/legend)", childed);
    component_impl!(Link, "link", web_sys::HtmlLinkElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link)", childless);
    component_impl!(Map, "map", web_sys::HtmlMapElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map)", childed);
    component_impl!(Meta, "meta", web_sys::HtmlMetaElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta)", childless);
    component_impl!(Meter, "meter", web_sys::HtmlMeterElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meter)", childed);
    component_impl!(Ol, "ol", web_sys::HtmlOListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol)", childed);
    component_impl!(Object, "object", web_sys::HtmlObjectElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object)", childed);
    component_impl!(OptGroup, "optgroup", web_sys::HtmlOptGroupElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup)", childed);
    component_impl!(Option, "option", web_sys::HtmlOptionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option)", childed);
    component_impl!(Output, "output", web_sys::HtmlOutputElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output)", childed);
    component_impl!(Paragraph, "p", web_sys::HtmlParagraphElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)", childed);
    component_impl!(Picture, "picture", web_sys::HtmlPictureElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture)", childed);
    component_impl!(Pre, "pre", web_sys::HtmlPreElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)", childed);
    component_impl!(Progress, "progress", web_sys::HtmlProgressElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/progress)", childed);
    component_impl!(Quote, "q", web_sys::HtmlQuoteElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)", childed);
    component_impl!(Select, "select", web_sys::HtmlSelectElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select)", childed);
    component_impl!(Source, "source", web_sys::HtmlSourceElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source)", childless);
    component_impl!(Span, "span", web_sys::HtmlSpanElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)", childed);
    component_impl!(Style, "style", web_sys::HtmlStyleElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style)", childed);
    component_impl!(Th, "th", web_sys::HtmlTableCellElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th)", childed);
    component_impl!(Td, "td", web_sys::HtmlTableCellElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)", childed);
    component_impl!(Col, "col", web_sys::HtmlTableColElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/col)", childed);
    component_impl!(ColGroup, "colgroup", web_sys::HtmlTableColElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup)", childed);
    component_impl!(Table, "table", web_sys::HtmlTableElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table)", childed);
    component_impl!(Tr, "tr", web_sys::HtmlTableRowElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr)", childed);
    component_impl!(THead, "thead", web_sys::HtmlTableSectionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead)", childed);
    component_impl!(TFoot, "tfoot", web_sys::HtmlTableSectionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tfoot)", childed);
    component_impl!(TBody, "tbody", web_sys::HtmlTableSectionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody)", childed);
    component_impl!(Template, "template", web_sys::HtmlTemplateElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template)", childed);
    component_impl!(TextArea, "textarea", web_sys::HtmlTextAreaElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/textarea)", childed);
    component_impl!(Time, "time", web_sys::HtmlTimeElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time)", childed);
    component_impl!(Track, "track", web_sys::HtmlTrackElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/track)", childless);
    component_impl!(Ul, "ul", web_sys::HtmlUListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul)", childed);
    component_impl!(Video, "video", web_sys::HtmlVideoElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video)", childed);
}
pub use impls::*;

/// Make an HTML element with your own tag.
pub struct CustomElement {
    pub element: web_sys::HtmlElement,
}
impl CustomElement {
    pub fn new(tag_name: Cow<'static, str>) -> Self {
        Self {
            element: create_element(&tag_name),
        }
    }
}

fn create_element<E: JsCast>(tag_name: &str) -> E {
    DOCUMENT
        .with(|doc| doc.create_element(tag_name).unwrap_throw())
        .unchecked_into()
}
