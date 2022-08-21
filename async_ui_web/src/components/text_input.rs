use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use observables::{NextChangeFuture, ObservableAs, ObservableAsExt};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, InputEvent};

use crate::window::DOCUMENT;

use super::{
    dummy::{create_dummy, is_dummy},
    event_handler::EventHandler,
    ElementFuture,
};

pub struct TextInput<'c> {
    pub text: &'c (dyn ObservableAs<str> + 'c),
    pub on_change_text: &'c mut (dyn FnMut(String) + 'c),
}

impl<'c> Default for TextInput<'c> {
    fn default() -> Self {
        Self {
            text: &"",
            on_change_text: create_dummy(),
        }
    }
}

pub struct TextInputFuture<'c> {
    obs: &'c (dyn ObservableAs<str> + 'c),
    change_fut: NextChangeFuture<dyn ObservableAs<str> + 'c, &'c (dyn ObservableAs<str> + 'c)>,
    node: HtmlInputElement,
    set: bool,
    on_input: Option<(
        EventHandler<'c, InputEvent>,
        &'c mut (dyn FnMut(String) + 'c),
    )>,
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
            let txt = this.obs.borrow_observable_as();
            this.node.set_value(&*txt);
        }
        if let Some((on_input_listener, on_input_handler)) = &mut this.on_input {
            match Pin::new(on_input_listener).poll_next(cx) {
                Poll::Ready(Some(_ev)) => on_input_handler(this.node.value()),
                _ => (),
            }
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
        let on_input = (!is_dummy(self.on_change_text)).then(|| {
            let listener = EventHandler::new();
            input.set_oninput(Some(listener.get_function()));
            (listener, self.on_change_text)
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
