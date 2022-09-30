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
pub struct TextInputProp<'c> {
    pub text: Option<&'c dyn ObservableAs<str>>,
    pub on_change_text: Option<&'c mut dyn FnMut(TextInputEvent)>,
    pub on_submit: Option<&'c mut dyn FnMut(TextInputEvent)>,
    pub on_blur: Option<&'c mut dyn FnMut(TextInputEvent)>,
    pub on_focus: Option<&'c mut dyn FnMut(TextInputEvent)>,
    pub multiline: Option<bool>,
    pub class: Option<&'c ClassList<'c>>,
    pub placeholder: Option<&'c dyn ObservableAs<str>>,
}

pub async fn text_input<'c>(
    TextInputProp {
        text,
        mut on_change_text,
        mut on_submit,
        mut on_blur,
        mut on_focus,
        multiline,
        class,
        placeholder,
    }: TextInputProp<'c>,
) {
    let text = text.unwrap_or(&"");
    let placeholder = placeholder.unwrap_or(&"");
    let multiline = multiline.unwrap_or_default();

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
