use futures_lite::FutureExt;
use observables::{ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};

use crate::{utils::class_list::ClassList, window::DOCUMENT};

use super::{
    events::{create_handler, EventsManager, QueuedEvent},
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
    Class(&'c ClassList<'c>),
    Placeholder(&'c dyn ObservableAs<str>),
    #[default]
    Null,
}

pub async fn text_input<'c, I: IntoIterator<Item = TextInputProp<'c>>>(props: I) {
    let mut text = None;
    let mut on_change_text = None;
    let mut on_submit = None;
    let mut on_blur = None;
    let mut on_focus = None;
    let mut multiline = false;
    let mut class = None;
    let mut placeholder = None;
    for prop in props {
        match prop {
            TextInputProp::Text(v) => text = Some(v),
            TextInputProp::OnChangeText(v) => on_change_text = Some(v),
            TextInputProp::OnSubmit(v) => on_submit = Some(v),
            TextInputProp::OnBlur(v) => on_blur = Some(v),
            TextInputProp::OnFocus(v) => on_focus = Some(v),
            TextInputProp::MultiLine(v) => multiline = v,
            TextInputProp::Class(v) => class = Some(v),
            TextInputProp::Placeholder(v) => placeholder = Some(v),
            TextInputProp::Null => {}
        }
    }
    let text = text.unwrap_or(&"");
    let placeholder = placeholder.unwrap_or(&"");

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

    let mut handlers = SmallVec::<[_; 5]>::new();
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
    if let Some(class) = class.take() {
        class.set_dom(input.as_elem().class_list());
    }

    let future = (async {
        manager.grab_waker().await;
        loop {
            let mut events = manager.get_queue().await;
            for event in events.drain(..) {
                let text_input_event = TextInputEvent {
                    node: input.clone(),
                };
                match event {
                    QueuedEvent::Input(_e) => {
                        on_change_text.as_mut().map(|f| f(text_input_event));
                    }
                    QueuedEvent::KeyPress(e) => {
                        if e.key() == "Enter" {
                            e.prevent_default();
                            on_submit.as_mut().map(|f| f(text_input_event));
                        }
                    }
                    QueuedEvent::Blur(_e) => {
                        on_blur.as_mut().map(|f| f(text_input_event));
                    }
                    QueuedEvent::Focus(_e) => {
                        on_focus.as_mut().map(|f| f(text_input_event));
                    }
                    _ => {}
                }
            }
        }
    })
    .or(async {
        loop {
            input.set_value(&*text.borrow_observable_as());
            text.until_change().await;
        }
    })
    .or(async {
        loop {
            input_elem
                .set_attribute("placeholder", &*placeholder.borrow_observable_as())
                .expect("set placeholder failed");
            placeholder.until_change().await;
        }
    });

    ElementFuture::new(future, input.as_elem().clone().into()).await;
}
