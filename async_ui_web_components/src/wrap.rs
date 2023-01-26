use std::{cell::RefCell, future::Future, ops::Deref, rc::Rc};

use super::event_handling::EventsHandler;
use async_ui_web_core::{window::DOCUMENT, ContainerNodeFuture};
use wasm_bindgen::JsCast;

pub trait IsHtmlElement: AsRef<web_sys::HtmlElement> {
    const TAG_NAME: &'static str;
}
pub struct WrappedHtmlElement<E> {
    pub(crate) element: E,
    pub(crate) handlers: RefCell<Option<Rc<RefCell<EventsHandler>>>>,
}

impl<E: IsHtmlElement> WrappedHtmlElement<E> {
    pub fn new() -> Self
    where
        E: JsCast,
    {
        let elem = DOCUMENT.with(|doc| {
            doc.create_element(E::TAG_NAME)
                .expect("create element failed")
        });
        Self::from_element(elem.unchecked_into())
    }

    pub fn from_element(element: E) -> Self {
        Self {
            element,
            handlers: RefCell::new(None),
        }
    }
}

impl<E: IsHtmlElement> WrappedHtmlElement<E> {
    pub fn render<F: Future>(&self, child: F) -> ContainerNodeFuture<F> {
        ContainerNodeFuture::new(child, self.element.as_ref().deref().deref().to_owned())
    }
}

impl<E: IsHtmlElement> Deref for WrappedHtmlElement<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl<E: IsHtmlElement> AsRef<web_sys::HtmlElement> for WrappedHtmlElement<E> {
    fn as_ref(&self) -> &web_sys::HtmlElement {
        self.element.as_ref()
    }
}
