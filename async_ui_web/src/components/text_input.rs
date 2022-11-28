use futures_lite::FutureExt;
use observables::{ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};

use crate::{utils::class_list::ClassList, window::DOCUMENT};

use super::{
    dummy::{dummy_handler, is_dummy_handler},
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
pub struct TextInputProps<'c> {
    pub text: &'c dyn ObservableAs<str>,
    pub on_change_text: &'c mut dyn FnMut(TextInputEvent),
    pub on_submit: &'c mut dyn FnMut(TextInputEvent),
    pub on_blur: &'c mut dyn FnMut(TextInputEvent),
    pub on_focus: &'c mut dyn FnMut(TextInputEvent),
    pub multiline: bool,
    pub placeholder: &'c dyn ObservableAs<str>,
    pub class: Option<&'c ClassList<'c>>,
}

impl<'c> Default for TextInputProps<'c> {
    fn default() -> Self {
        Self {
            text: &[""],
            on_change_text: dummy_handler(),
            on_submit: dummy_handler(),
            on_blur: dummy_handler(),
            on_focus: dummy_handler(),
            multiline: false,
            placeholder: &[""],
            class: None,
        }
    }
}

pub async fn text_input<'c>(
    TextInputProps {
        text,
        on_change_text,
        on_submit,
        on_blur,
        on_focus,
        multiline,
        class,
        placeholder,
    }: TextInputProps<'c>,
) {
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

    if !is_dummy_handler(on_change_text) {
        let h = create_handler(&manager, |e| QueuedEvent::Input(e));
        input_elem.set_oninput(Some(h.get_function()));
        handlers.push(h);
    }
    if !is_dummy_handler(on_submit) {
        let h = create_handler(&manager, |e| QueuedEvent::KeyPress(e));
        input_elem.set_onkeypress(Some(h.get_function()));
        handlers.push(h);
    }
    if !is_dummy_handler(on_blur) {
        let h = create_handler(&manager, |e| QueuedEvent::Blur(e));
        input_elem.set_onblur(Some(h.get_function()));
        handlers.push(h);
    }
    if !is_dummy_handler(on_focus) {
        let h = create_handler(&manager, |e| QueuedEvent::Focus(e));
        input_elem.set_onfocus(Some(h.get_function()));
        handlers.push(h);
    }
    if let Some(class) = class {
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
                        on_change_text(text_input_event);
                    }
                    QueuedEvent::KeyPress(e) => {
                        if e.key() == "Enter" {
                            e.prevent_default();
                            on_submit(text_input_event);
                        }
                    }
                    QueuedEvent::Blur(_e) => {
                        on_blur(text_input_event);
                    }
                    QueuedEvent::Focus(_e) => {
                        on_focus(text_input_event);
                    }
                    _ => {}
                }
            }
        }
    })
    .or(text.for_each(|t| input.set_value(t)))
    .or(placeholder.for_each(|t| {
        input_elem
            .set_attribute("placeholder", t)
            .expect("set placeholder failed")
    }));

    ElementFuture::new(future, input.as_elem().clone().into()).await;
}
