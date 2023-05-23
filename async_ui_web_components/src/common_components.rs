use std::{
    borrow::Cow,
    future::{pending, Future, Pending},
    ops::Deref,
};

use async_ui_web_core::{window::DOCUMENT, ContainerNodeFuture};
use wasm_bindgen::prelude::{JsCast, UnwrapThrowExt};

macro_rules! component_impl {
    ($ty:ident, $tag_name:literal, $elem_ty:ty) => {
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
        impl Deref for $ty {
            type Target = $elem_ty;
            fn deref(&self) -> &Self::Target {
                &self.element
            }
        }
    };
    ($ty:ident, $tag_name:literal, $elem_ty:ty, childed) => {
        component_impl!($ty, $tag_name, $elem_ty);
        impl $ty {
            pub fn render<F: Future>(&self, c: F) -> ContainerNodeFuture<F> {
                ContainerNodeFuture::new(c, AsRef::<web_sys::Node>::as_ref(&self.element).clone())
            }
        }
    };
    ($ty:ident, $tag_name:literal, $elem_ty:ty, childless) => {
        component_impl!($ty, $tag_name, $elem_ty);
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

component_impl!(Anchor, "a", web_sys::HtmlAnchorElement, childed);
component_impl!(Area, "area", web_sys::HtmlAreaElement, childless);
component_impl!(Audio, "audio", web_sys::HtmlAudioElement, childed);
component_impl!(Br, "br", web_sys::HtmlBrElement, childless);
component_impl!(Base, "base", web_sys::HtmlBaseElement, childless);
component_impl!(Button, "button", web_sys::HtmlButtonElement, childed);
component_impl!(Canvas, "canvas", web_sys::HtmlCanvasElement, childed);
component_impl!(Dl, "dl", web_sys::HtmlDListElement, childed);
component_impl!(Data, "data", web_sys::HtmlDataElement, childed);
component_impl!(DataList, "datalist", web_sys::HtmlDataListElement, childed);
component_impl!(Dialog, "dialog", web_sys::HtmlDialogElement, childed);
component_impl!(Div, "div", web_sys::HtmlDivElement, childed);
component_impl!(Embed, "embed", web_sys::HtmlEmbedElement, childless);
component_impl!(FieldSet, "fieldset", web_sys::HtmlFieldSetElement, childed);
component_impl!(Form, "form", web_sys::HtmlFormElement, childed);
component_impl!(FrameSet, "frameset", web_sys::HtmlFrameSetElement, childed);
component_impl!(Hr, "hr", web_sys::HtmlHrElement, childless);
component_impl!(H1, "h1", web_sys::HtmlHeadingElement, childed);
component_impl!(H2, "h2", web_sys::HtmlHeadingElement, childed);
component_impl!(H3, "h3", web_sys::HtmlHeadingElement, childed);
component_impl!(H4, "h4", web_sys::HtmlHeadingElement, childed);
component_impl!(H5, "h5", web_sys::HtmlHeadingElement, childed);
component_impl!(H6, "h6", web_sys::HtmlHeadingElement, childed);
component_impl!(IFrame, "iframe", web_sys::HtmlIFrameElement, childed);
component_impl!(Img, "img", web_sys::HtmlImageElement, childed);
component_impl!(Input, "input", web_sys::HtmlInputElement, childless);
component_impl!(Li, "li", web_sys::HtmlLiElement, childed);
component_impl!(Label, "label", web_sys::HtmlLabelElement, childed);
component_impl!(Legend, "legend", web_sys::HtmlLegendElement, childed);
component_impl!(Link, "link", web_sys::HtmlLinkElement, childless);
component_impl!(Map, "map", web_sys::HtmlMapElement, childed);
component_impl!(Meta, "meta", web_sys::HtmlMetaElement, childless);
component_impl!(Meter, "meter", web_sys::HtmlMeterElement, childed);
component_impl!(Ol, "ol", web_sys::HtmlOListElement, childed);
component_impl!(Object, "object", web_sys::HtmlObjectElement, childed);
component_impl!(OptGroup, "optgroup", web_sys::HtmlOptGroupElement, childed);
component_impl!(Option, "option", web_sys::HtmlOptionElement, childed);
component_impl!(Output, "output", web_sys::HtmlOutputElement, childed);
component_impl!(Paragraph, "p", web_sys::HtmlParagraphElement, childed);
component_impl!(Picture, "picture", web_sys::HtmlPictureElement, childed);
component_impl!(Pre, "pre", web_sys::HtmlPreElement, childed);
component_impl!(Progress, "progress", web_sys::HtmlProgressElement, childed);
component_impl!(Quote, "q", web_sys::HtmlQuoteElement, childed);
component_impl!(Select, "select", web_sys::HtmlSelectElement, childed);
component_impl!(Source, "source", web_sys::HtmlSourceElement, childless);
component_impl!(Span, "span", web_sys::HtmlSpanElement, childed);
component_impl!(Style, "style", web_sys::HtmlStyleElement, childed);
component_impl!(Th, "th", web_sys::HtmlTableCellElement, childed);
component_impl!(Td, "td", web_sys::HtmlTableCellElement, childed);
component_impl!(Col, "col", web_sys::HtmlTableColElement, childed);
component_impl!(ColGroup, "colgroup", web_sys::HtmlTableColElement, childed);
component_impl!(Table, "table", web_sys::HtmlTableElement, childed);
component_impl!(Tr, "tr", web_sys::HtmlTableRowElement, childed);
component_impl!(THead, "thead", web_sys::HtmlTableSectionElement, childed);
component_impl!(TFoot, "tfoot", web_sys::HtmlTableSectionElement, childed);
component_impl!(TBody, "tbody", web_sys::HtmlTableSectionElement, childed);
component_impl!(Template, "template", web_sys::HtmlTemplateElement, childed);
component_impl!(TextArea, "textarea", web_sys::HtmlTextAreaElement, childed);
component_impl!(Time, "time", web_sys::HtmlTimeElement, childed);
component_impl!(Track, "track", web_sys::HtmlTrackElement, childless);
component_impl!(Ul, "ul", web_sys::HtmlUListElement, childed);
component_impl!(Video, "video", web_sys::HtmlVideoElement, childed);

pub struct CustomElement {
    pub element: web_sys::HtmlElement,
}
impl CustomElement {
    pub fn new(tag_name: Cow<'static, str>) -> Self {
        Self {
            element: create_element(&*tag_name),
        }
    }
}

fn create_element<E: JsCast>(tag_name: &str) -> E {
    DOCUMENT
        .with(|doc| doc.create_element(&*tag_name).unwrap_throw())
        .unchecked_into()
}
