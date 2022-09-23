use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use observables::{NextChangeFuture, ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};

use crate::window::DOCUMENT;

use super::{
    dummy::create_dummy,
    events::{maybe_create_handler, EventHandler, EventsManager, QueuedEvent},
    ElementFuture,
};
#[derive(Clone)]
enum InputNode {
    OneLine(HtmlInputElement),
    MultiLine(HtmlTextAreaElement),
}

impl InputNode {
    fn as_elem(&self) -> &HtmlElement {
        match self {
            InputNode::OneLine(e) => e.unchecked_ref(),
            InputNode::MultiLine(e) => e.unchecked_ref(),
        }
    }
    fn get_value(&self) -> String {
        match self {
            InputNode::OneLine(e) => e.value(),
            InputNode::MultiLine(e) => e.inner_text(),
        }
    }
    fn set_value(&self, value: &str) {
        match self {
            InputNode::OneLine(e) => e.set_value(value),
            InputNode::MultiLine(e) => e.set_inner_text(value),
        }
    }
}

pub struct TextInputEvent {
    node: InputNode,
}

impl TextInputEvent {
    pub fn get_text(&self) -> String {
        self.node.get_value()
    }
}
pub struct TextInput<'c> {
    pub text: &'c (dyn ObservableAs<str> + 'c),
    pub on_change_text: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_submit: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_blur: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_focus: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub multiline: bool,
}

impl<'c> Default for TextInput<'c> {
    fn default() -> Self {
        Self {
            text: &"",
            on_change_text: create_dummy(),
            on_submit: create_dummy(),
            on_blur: create_dummy(),
            on_focus: create_dummy(),
            multiline: false,
        }
    }
}

pub struct TextInputFuture<'c> {
    obs: &'c (dyn ObservableAs<str> + 'c),
    change_fut: NextChangeFuture<dyn ObservableAs<str> + 'c, &'c (dyn ObservableAs<str> + 'c)>,
    node: InputNode,
    set: bool,
    manager: Rc<EventsManager>,
    _handlers: SmallVec<[EventHandler<'c>; 3]>,
    on_change_text: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    on_submit: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    on_blur: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    on_focus: &'c mut (dyn FnMut(TextInputEvent) + 'c),
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
        if let Some(mut events) = this.manager.borrow_queue_mut() {
            for event in events.drain(..) {
                let node = this.node.clone();
                let text_input_event = TextInputEvent { node };
                match event {
                    QueuedEvent::Input(_e) => (this.on_change_text)(text_input_event),
                    QueuedEvent::KeyPress(e) => {
                        if e.key() == "Enter" {
                            e.prevent_default();
                            (this.on_submit)(text_input_event)
                        }
                    }
                    QueuedEvent::Blur(_e) => (this.on_blur)(text_input_event),
                    QueuedEvent::Focus(_e) => (this.on_focus)(text_input_event),
                    _ => {}
                }
            }
        }
        Poll::Pending
    }
}

impl<'c> IntoFuture for TextInput<'c> {
    type Output = ();
    type IntoFuture = ElementFuture<TextInputFuture<'c>>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            text,
            on_change_text,
            on_submit,
            on_blur,
            on_focus,
            multiline,
        } = self;
        let input = DOCUMENT.with(|doc| {
            let elem = doc
                .create_element(match multiline {
                    true => "textarea",
                    false => "input",
                })
                .expect("create element failed");
            match multiline {
                true => InputNode::MultiLine(elem.unchecked_into()),
                false => InputNode::OneLine(elem.unchecked_into()),
            }
        });
        let mut handlers = SmallVec::new();
        let manager = EventsManager::new();
        let input_elem = input.as_elem();

        if let Some(h) = maybe_create_handler(&manager, on_change_text, |e| QueuedEvent::Input(e)) {
            input_elem.set_oninput(Some(h.get_function()));
            handlers.push(h);
        }
        if let Some(h) = maybe_create_handler(&manager, on_submit, |e| QueuedEvent::KeyPress(e)) {
            input_elem.set_onkeypress(Some(h.get_function()));
            handlers.push(h);
        }
        if let Some(h) = maybe_create_handler(&manager, on_blur, |e| QueuedEvent::Blur(e)) {
            input_elem.set_onblur(Some(h.get_function()));
            handlers.push(h);
        }
        if let Some(h) = maybe_create_handler(&manager, on_focus, |e| QueuedEvent::Focus(e)) {
            input_elem.set_onfocus(Some(h.get_function()));
            handlers.push(h);
        }

        ElementFuture::new(
            TextInputFuture {
                obs: text,
                change_fut: text.until_change(),
                node: input.clone().into(),
                set: false,
                _handlers: handlers,
                manager,
                on_change_text,
                on_submit,
                on_blur,
                on_focus,
            },
            input.as_elem().clone().into(),
        )
    }
}
