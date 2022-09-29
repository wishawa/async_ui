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
    events::{create_handler, EventHandler, EventsManager, QueuedEvent},
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
#[derive(Default)]
pub enum TextInputProp<'c> {
    Text(&'c dyn ObservableAs<str>),
    OnChangeText(&'c mut dyn FnMut(TextInputEvent)),
    OnSubmit(&'c mut dyn FnMut(TextInputEvent)),
    OnBlur(&'c mut dyn FnMut(TextInputEvent)),
    OnFocus(&'c mut dyn FnMut(TextInputEvent)),
    MultiLine(bool),
    #[default]
    Null,
}

pub struct TextInputFuture<'c> {
    obs: &'c (dyn ObservableAs<str> + 'c),
    change_fut: NextChangeFuture<dyn ObservableAs<str> + 'c, &'c (dyn ObservableAs<str> + 'c)>,
    node: InputNode,
    first: bool,
    manager: Rc<EventsManager>,
    _handlers: SmallVec<[EventHandler<'c>; 3]>,
    on_change_text: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
    on_submit: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
    on_blur: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
    on_focus: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
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
        if reset || this.first {
            let txt = this.obs.borrow_observable_as();
            this.node.set_value(&*txt);
        }
        if this.first {
            this.first = false;
            this.manager.set_waker(cx.waker());
        }
        if let Some(mut events) = this.manager.borrow_queue_mut() {
            for event in events.drain(..) {
                let node = this.node.clone();
                let text_input_event = TextInputEvent { node };
                match event {
                    QueuedEvent::Input(_e) => {
                        this.on_change_text.as_mut().map(|f| f(text_input_event));
                    }
                    QueuedEvent::KeyPress(e) => {
                        if e.key() == "Enter" {
                            e.prevent_default();
                            this.on_submit.as_mut().map(|f| f(text_input_event));
                        }
                    }
                    QueuedEvent::Blur(_e) => {
                        this.on_blur.as_mut().map(|f| f(text_input_event));
                    }
                    QueuedEvent::Focus(_e) => {
                        this.on_focus.as_mut().map(|f| f(text_input_event));
                    }
                    _ => {}
                }
            }
        }
        Poll::Pending
    }
}

pub async fn text_input<'c, I: IntoIterator<Item = TextInputProp<'c>>>(props: I) {
    let mut text = None;
    let mut on_change_text = None;
    let mut on_submit = None;
    let mut on_blur = None;
    let mut on_focus = None;
    let mut multiline = false;
    for prop in props {
        match prop {
            TextInputProp::Text(v) => text = Some(v),
            TextInputProp::OnChangeText(v) => on_change_text = Some(v),
            TextInputProp::OnSubmit(v) => on_submit = Some(v),
            TextInputProp::OnBlur(v) => on_blur = Some(v),
            TextInputProp::OnFocus(v) => on_focus = Some(v),
            TextInputProp::MultiLine(v) => multiline = v,
            TextInputProp::Null => {}
        }
    }

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

    if on_change_text.is_some() {
        let h = create_handler(&manager, |e| QueuedEvent::Input(e));
        input_elem.set_oninput(Some(h.get_function()));
        handlers.push(h);
    }
    if on_submit.is_some() {
        let h = create_handler(&manager, |e| QueuedEvent::KeyPress(e));
        input_elem.set_onkeypress(Some(h.get_function()));
        handlers.push(h);
    }
    if on_blur.is_some() {
        let h = create_handler(&manager, |e| QueuedEvent::Blur(e));
        input_elem.set_onblur(Some(h.get_function()));
        handlers.push(h);
    }
    if on_focus.is_some() {
        let h = create_handler(&manager, |e| QueuedEvent::Focus(e));
        input_elem.set_onfocus(Some(h.get_function()));
        handlers.push(h);
    }

    let text = text.unwrap_or(&"");
    ElementFuture::new(
        TextInputFuture {
            obs: text,
            change_fut: text.until_change(),
            node: input.clone().into(),
            first: true,
            _handlers: handlers,
            manager,
            on_change_text,
            on_submit,
            on_blur,
            on_focus,
        },
        input.as_elem().clone().into(),
    )
    .await;
}
