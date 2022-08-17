use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use observables::{NextChangeFuture, Observable, ObservableExt};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};

use crate::window::DOCUMENT;

use super::{event_handler::EventHandler, ElementFuture};

pub struct TextInput<'c> {
    pub text: &'c (dyn Observable<str> + 'c),
    pub on_input: &'c (dyn Fn(InputEvent) + 'c),
}

fn dummy_handler_fn(_e: InputEvent) {}
impl<'c> Default for TextInput<'c> {
    fn default() -> Self {
        Self {
            text: &"",
            on_input: &dummy_handler_fn,
        }
    }
}

pub struct TextInputFuture<'c> {
    obs: &'c (dyn Observable<str> + 'c),
    change_fut: NextChangeFuture<dyn Observable<str> + 'c, &'c (dyn Observable<str> + 'c)>,
    node: HtmlInputElement,
    set: bool,
    on_input: Option<EventHandler<'c, InputEvent>>,
}
impl<'c> Future for TextInputFuture<'c> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.get_mut();
        let reset = match Pin::new(&mut this.change_fut).poll(cx) {
            Poll::Ready(_) => {
                this.change_fut = this.obs.until_change();
                let _ = Pin::new(&mut this.change_fut).poll(cx);
                true
            }
            Poll::Pending => false,
        };
        if reset || !this.set {
            this.set = true;
            let txt = this.obs.observable_borrow();
            this.node.set_value(&*txt);
        }
        if let Some(on_input) = &mut this.on_input {
            let _ = Pin::new(on_input).poll(cx);
        }
        Poll::Pending
    }
}

impl<'c> IntoFuture for TextInput<'c> {
    type Output = ();
    type IntoFuture = ElementFuture<TextInputFuture<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        let input = DOCUMENT.with(|doc| {
            let elem = doc.create_element("input").expect("create element failed");
            let elem: HtmlInputElement = elem.unchecked_into();
            elem
        });
        let on_input = (!std::ptr::eq(self.on_input as *const _, &dummy_handler_fn as *const _))
            .then(|| {
                let handler = EventHandler::new(self.on_input);
                input.set_oninput(Some(handler.get_function()));
                handler
            });

        ElementFuture::new(
            TextInputFuture {
                obs: self.text,
                change_fut: self.text.until_change(),
                node: input.clone().into(),
                set: false,
                on_input,
            },
            input.into(),
        )
    }
}
