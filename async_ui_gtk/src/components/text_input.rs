use futures_lite::FutureExt;
use glib::Cast;
use gtk::{
    prelude::{EntryBufferExt, EntryBufferExtManual, TextBufferExt},
    traits::{EntryExt, TextViewExt, WidgetExt},
};
use observables::{ObservableAs, ObservableAsExt};

use crate::widget::{WidgetOp, WrappedWidget};

use super::{
    dummy::{dummy_handler, is_dummy_handler},
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

pub struct TextInputProps<'c> {
    pub text: &'c (dyn ObservableAs<str> + 'c),
    pub on_change_text: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_submit: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_blur: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_focus: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub multiline: bool,
    pub placeholder: &'c (dyn ObservableAs<str> + 'c),
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
        placeholder,
    }: TextInputProps<'c>,
) {
    let manager = EventsManager::new();
    let input: gtk::Widget;
    let buffer;
    let mut entry_node = None;
    match multiline {
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
            if !is_dummy_handler(on_submit) {
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
    if !is_dummy_handler(on_change_text) {
        let mgr = manager.clone();
        buffer.connect_changed(move || {
            mgr.add_event(QueuedEvent::Input);
        });
    }
    if !is_dummy_handler(on_blur) || !is_dummy_handler(on_focus) {
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
                        QueuedEvent::Input => on_change_text(text_input_event),
                        QueuedEvent::Blur => on_blur(text_input_event),
                        QueuedEvent::Focus => on_focus(text_input_event),
                        QueuedEvent::Submit => on_submit(text_input_event),
                        _ => {}
                    };
                }
            }
        })
        .or(text.for_each(|t| buffer.set_value(t)))
        .or(placeholder.for_each(|t| {
            if let Some(ent) = entry_node.as_ref() {
                ent.set_placeholder_text(Some(t));
            }
        })),
        WrappedWidget {
            widget: input.clone().upcast(),
            inner_widget: input.upcast(),
            op: WidgetOp::NoChild,
        },
    )
    .await;
}
