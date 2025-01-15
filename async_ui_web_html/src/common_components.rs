use std::{
    borrow::Cow,
    future::{pending, Future, Pending},
    ops::Deref,
};

use async_ui_web_core::{
    dom::{self, elements, Node},
    ContainerNodeFuture,
};
#[cfg(feature = "csr")]
use wasm_bindgen::prelude::{JsCast, UnwrapThrowExt};

macro_rules! component_impl {
    ($ty:ident, $tag_name:literal, $elem_ty:ident, $link:tt) => {
        #[doc = "The HTML `"]
        #[doc = $tag_name]
        #[doc = "` tag."]
        #[doc = "See"]
        #[doc = $link]
        #[doc = "."]
        pub struct $ty {
            pub element: elements::$elem_ty,
        }
        impl $ty {
            #[doc = "Create a new instance of this type."]
            #[doc = ""]
            #[doc = "This creates the HTML node, but doesn't put it on the screen yet."]
            #[doc = "Use the `.render(_)` method to do that."]
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
            type Target = elements::$elem_ty;
            fn deref(&self) -> &Self::Target {
                &self.element
            }
        }
        impl<X> AsRef<X> for $ty
        where
            elements::$elem_ty: AsRef<X>,
        {
            fn as_ref(&self) -> &X {
                self.element.as_ref()
            }
        }
    };
    ($ty:ident, $tag_name:literal, $elem_ty:ident, $link:tt, childed) => {
        component_impl!($ty, $tag_name, $elem_ty, $link);
        impl $ty {
            #[doc = "Put this HTML element on the screen."]
            #[doc = ""]
            #[doc = "The return Future completes when the given argument Future finishes."]
            #[doc = "Anything the argument Future renders will in the HTML tag of this component."]
            #[doc = ""]
            #[doc = "When the returned Future is dropped, the HTML element will be removed."]
            #[doc = ""]
            #[doc = "This method should only be called once. It may misbehave otherwise."]
            pub fn render<F: Future>(&self, c: F) -> ContainerNodeFuture<F> {
                ContainerNodeFuture::new(c, AsRef::<Node>::as_ref(&self.element).clone())
            }
        }
    };
    ($ty:ident, $tag_name:literal, $elem_ty:ident, $link:tt, childless) => {
        component_impl!($ty, $tag_name, $elem_ty, $link);
        impl $ty {
            #[doc = "Put this HTML element on the screen."]
            #[doc = ""]
            #[doc = "This method returns a Future that never finishes."]
            #[doc = ""]
            #[doc = "When the returned Future is dropped, the HTML element will be removed."]
            #[doc = ""]
            #[doc = "This method should only be called once. It may misbehave otherwise."]
            pub fn render(&self) -> ContainerNodeFuture<Pending<()>> {
                ContainerNodeFuture::new(pending(), AsRef::<Node>::as_ref(&self.element).clone())
            }
        }
    };
}

#[rustfmt::skip]
mod impls {
    use super::*;
    component_impl!(Anchor, "a", HtmlAnchorElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)", childed);
    component_impl!(Area, "area", HtmlAreaElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area)", childless);
    component_impl!(Audio, "audio", HtmlAudioElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio)", childed);
    // component_impl!(Bold, "b", HtmlBElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b)", childed);
    component_impl!(Br, "br", HtmlBrElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br)", childless);
    component_impl!(Base, "base", HtmlBaseElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base)", childless);
    component_impl!(Button, "button", HtmlButtonElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button)", childed);
    component_impl!(Canvas, "canvas", HtmlCanvasElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas)", childed);
    component_impl!(Dl, "dl", HtmlDListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl)", childed);
    component_impl!(Data, "data", HtmlDataElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/data)", childed);
    component_impl!(DataList, "datalist", HtmlDataListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist)", childed);
    component_impl!(Dialog, "dialog", HtmlDialogElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dialog)", childed);
    component_impl!(Div, "div", HtmlDivElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div)", childed);
    component_impl!(Embed, "embed", HtmlEmbedElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/embed)", childless);
    component_impl!(FieldSet, "fieldset", HtmlFieldSetElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset)", childed);
    component_impl!(Form, "form", HtmlFormElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form)", childed);
    component_impl!(FrameSet, "frameset", HtmlFrameSetElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/frameset)", childed);
    component_impl!(Hr, "hr", HtmlHrElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)", childless);
    component_impl!(H1, "h1", HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1)", childed);
    component_impl!(H2, "h2", HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h2)", childed);
    component_impl!(H3, "h3", HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h3)", childed);
    component_impl!(H4, "h4", HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h4)", childed);
    component_impl!(H5, "h5", HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h5)", childed);
    component_impl!(H6, "h6", HtmlHeadingElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h6)", childed);
    component_impl!(Italic, "i", HtmlImageElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i)", childed);
    component_impl!(IFrame, "iframe", HtmlIFrameElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe)", childed);
    component_impl!(Img, "img", HtmlImageElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img)", childed);
    component_impl!(Input, "input", HtmlInputElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input)", childless);
    component_impl!(Li, "li", HtmlLiElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)", childed);
    component_impl!(Label, "label", HtmlLabelElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label)", childed);
    component_impl!(Legend, "legend", HtmlLegendElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/legend)", childed);
    component_impl!(Link, "link", HtmlLinkElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link)", childless);
    component_impl!(Map, "map", HtmlMapElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map)", childed);
    component_impl!(Meta, "meta", HtmlMetaElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta)", childless);
    component_impl!(Meter, "meter", HtmlMeterElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meter)", childed);
    component_impl!(Ol, "ol", HtmlOListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol)", childed);
    component_impl!(Object, "object", HtmlObjectElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object)", childed);
    component_impl!(OptGroup, "optgroup", HtmlOptGroupElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup)", childed);
    component_impl!(Option, "option", HtmlOptionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option)", childed);
    component_impl!(Output, "output", HtmlOutputElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output)", childed);
    component_impl!(Paragraph, "p", HtmlParagraphElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)", childed);
    component_impl!(Picture, "picture", HtmlPictureElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture)", childed);
    component_impl!(Pre, "pre", HtmlPreElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)", childed);
    component_impl!(Progress, "progress", HtmlProgressElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/progress)", childed);
    component_impl!(Quote, "q", HtmlQuoteElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)", childed);
    component_impl!(Select, "select", HtmlSelectElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select)", childed);
    component_impl!(Source, "source", HtmlSourceElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source)", childless);
    component_impl!(Span, "span", HtmlSpanElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)", childed);
    component_impl!(Style, "style", HtmlStyleElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style)", childed);
    component_impl!(Th, "th", HtmlTableCellElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th)", childed);
    component_impl!(Td, "td", HtmlTableCellElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)", childed);
    component_impl!(Col, "col", HtmlTableColElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/col)", childed);
    component_impl!(ColGroup, "colgroup", HtmlTableColElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup)", childed);
    component_impl!(Table, "table", HtmlTableElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table)", childed);
    component_impl!(Tr, "tr", HtmlTableRowElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr)", childed);
    component_impl!(THead, "thead", HtmlTableSectionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead)", childed);
    component_impl!(TFoot, "tfoot", HtmlTableSectionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tfoot)", childed);
    component_impl!(TBody, "tbody", HtmlTableSectionElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody)", childed);
    component_impl!(Template, "template", HtmlTemplateElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template)", childed);
    component_impl!(TextArea, "textarea", HtmlTextAreaElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/textarea)", childed);
    component_impl!(Time, "time", HtmlTimeElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time)", childed);
    component_impl!(Track, "track", HtmlTrackElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/track)", childless);
    component_impl!(Ul, "ul", HtmlUListElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul)", childed);
    component_impl!(Video, "video", HtmlVideoElement, "[the documentation on MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video)", childed);
}
pub use impls::*;

/// Make an HTML element with your own tag.
pub struct CustomElement {
    pub element: dom::HtmlElement,
}

impl CustomElement {
    pub fn new(tag_name: Cow<'static, str>) -> Self {
        Self {
            element: create_element(&tag_name),
        }
    }

    pub fn render<F: Future>(&self, c: F) -> ContainerNodeFuture<F> {
        ContainerNodeFuture::new(c, AsRef::<Node>::as_ref(&self.element).clone())
    }
}

#[cfg(feature = "csr")]
fn create_element<E: JsCast>(tag_name: &str) -> E {
    use async_ui_web_core::window::DOCUMENT;
    DOCUMENT
        .with(|doc| doc.create_element(tag_name).unwrap_throw())
        .unchecked_into()
}

#[cfg(any(feature = "ssr", not(feature = "csr")))]
fn create_element<E: From<dom::Element>>(tag_name: &str) -> E {
    E::from(dom::create_ssr_element(tag_name))
}
