use futures_lite::FutureExt;
use glib::Cast;
use gtk::{
    prelude::{EntryBufferExt, EntryBufferExtManual, TextBufferExt},
    traits::{EntryExt, TextViewExt, WidgetExt},
};
use observables::{ObservableAs, ObservableAsExt};

use crate::widget::{WidgetOp, WrappedWidget};

use super::{
    events::{EventsManager, QueuedEvent},
    ElementFuture,
};

#[derive(Clone)]
enum InputBuffer {
    OneLine(gtk::EntryBuffer),
    MultiLine(gtk::TextBuffer),
}

impl InputBuffer {
    fn set_value(&self, text: &str) {
        match self {
            InputBuffer::OneLine(e) => e.set_text(text),
            InputBuffer::MultiLine(e) => e.set_text(text),
        }
    }
    fn get_value(&self) -> String {
        match self {
            InputBuffer::OneLine(e) => e.text(),
            InputBuffer::MultiLine(e) => e.text(&e.start_iter(), &e.end_iter(), false).to_string(),
        }
    }
    fn connect_changed<F: Fn() + 'static>(&self, func: F) {
        match self {
            InputBuffer::OneLine(e) => {
                e.connect_text_notify(move |_eb| {
                    func();
                });
            }
            InputBuffer::MultiLine(e) => {
                e.connect_changed(move |_tb| {
                    func();
                });
            }
        }
    }
}

pub struct TextInputEvent {
    buffer: InputBuffer,
}

impl TextInputEvent {
    pub fn get_text(&self) -> String {
        self.buffer.get_value()
    }
}

#[derive(Default)]
pub struct TextInputProps<'c> {
    pub text: Option<&'c (dyn ObservableAs<str> + 'c)>,
    pub on_change_text: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
    pub on_submit: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
    pub on_blur: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
    pub on_focus: Option<&'c mut (dyn FnMut(TextInputEvent) + 'c)>,
    pub multiline: Option<bool>,
    pub placeholder: Option<&'c (dyn ObservableAs<str> + 'c)>,
}

pub async fn text_input<'c>(
    TextInputProps {
        text,
        mut on_change_text,
        mut on_submit,
        mut on_blur,
        mut on_focus,
        multiline,
        placeholder,
    }: TextInputProps<'c>,
) {
    let text = text.unwrap_or(&"");
    let placeholder = placeholder.unwrap_or(&"");

    let manager = EventsManager::new();
    let input: gtk::Widget;
    let buffer;
    let mut entry_node = None;
    match multiline.unwrap_or_default() {
        true => {
            let text_buffer = gtk::TextBuffer::new(None);
            let text_view = gtk::TextView::new();
            text_view.set_buffer(Some(&text_buffer));
            input = text_view.upcast();
            buffer = InputBuffer::MultiLine(text_buffer);
        }
        false => {
            let entry_buffer = gtk::EntryBuffer::new(None);
            let entry = gtk::Entry::new();
            if on_submit.is_some() {
                let mgr = manager.clone();
                entry.connect_activate(move |_e| {
                    mgr.add_event(QueuedEvent::Submit);
                });
            }
            entry.set_buffer(&entry_buffer);
            input = entry.clone().upcast();
            buffer = InputBuffer::OneLine(entry_buffer);
            entry_node = Some(entry);
        }
    };
    if on_change_text.is_some() {
        let mgr = manager.clone();
        buffer.connect_changed(move || {
            mgr.add_event(QueuedEvent::Input);
        });
    }
    if on_blur.is_some() || on_focus.is_some() {
        let mgr = manager.clone();
        let focus_controller = gtk::EventControllerFocus::new();
        input.add_controller(&focus_controller);
        focus_controller.connect_contains_focus_notify(move |s| {
            mgr.add_event(if s.contains_focus() {
                QueuedEvent::Focus
            } else {
                QueuedEvent::Blur
            });
        });
    }
    ElementFuture::new(
        (async {
            manager.grab_waker().await;
            loop {
                let mut events = manager.get_queue().await;
                for event in events.drain(..) {
                    let text_input_event = TextInputEvent {
                        buffer: buffer.clone(),
                    };
                    match event {
                        QueuedEvent::Input => on_change_text.as_mut().map(|f| f(text_input_event)),
                        QueuedEvent::Blur => on_blur.as_mut().map(|f| f(text_input_event)),
                        QueuedEvent::Focus => on_focus.as_mut().map(|f| f(text_input_event)),
                        QueuedEvent::Submit => on_submit.as_mut().map(|f| f(text_input_event)),
                        _ => None,
                    };
                }
            }
        })
        .or(async {
            loop {
                buffer.set_value(&*text.borrow_observable_as());
                text.until_change().await;
            }
        })
        .or(async {
            loop {
                if let Some(ent) = entry_node.as_ref() {
                    ent.set_placeholder_text(Some(&*placeholder.borrow_observable_as()));
                }
                placeholder.until_change().await;
            }
        }),
        WrappedWidget {
            widget: input.clone().upcast(),
            inner_widget: input.upcast(),
            op: WidgetOp::NoChild,
        },
    )
    .await;
}
