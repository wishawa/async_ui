use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use glib::Cast;
use gtk::{
    prelude::{EntryBufferExt, EntryBufferExtManual, TextBufferExt},
    traits::{EntryExt, TextViewExt, WidgetExt},
};
use observables::{NextChangeFuture, ObservableAs, ObservableAsExt};

use crate::widget::WrappedWidget;

use super::{
    dummy::{create_dummy, is_dummy},
    event_channel::EventHandler,
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

pub struct TextInput<'c> {
    pub text: &'c (dyn ObservableAs<str> + 'c),
    pub on_change_text: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_submit: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub on_blur: &'c mut (dyn FnMut(TextInputEvent) + 'c),
    pub multiline: bool,
}

impl<'c> Default for TextInput<'c> {
    fn default() -> Self {
        Self {
            text: &"",
            on_change_text: create_dummy(),
            on_submit: create_dummy(),
            on_blur: create_dummy(),
            multiline: false,
        }
    }
}

pub struct TextInputFuture<'c> {
    obs: &'c (dyn ObservableAs<str> + 'c),
    change_fut: NextChangeFuture<dyn ObservableAs<str> + 'c, &'c (dyn ObservableAs<str> + 'c)>,
    buffer: InputBuffer,
    set: bool,
    on_input: Option<(
        EventHandler<'c, ()>,
        &'c mut (dyn FnMut(TextInputEvent) + 'c),
    )>,
    on_submit: Option<(
        EventHandler<'c, ()>,
        &'c mut (dyn FnMut(TextInputEvent) + 'c),
    )>,
    on_blur: Option<(
        EventHandler<'c, ()>,
        &'c mut (dyn FnMut(TextInputEvent) + 'c),
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
            this.buffer.set_value(&*txt);
        }
        if let Some((on_input_listener, on_input_handler)) = &mut this.on_input {
            match Pin::new(on_input_listener).poll_next(cx) {
                Poll::Ready(Some(_ev)) => on_input_handler(TextInputEvent {
                    buffer: this.buffer.clone(),
                }),
                _ => (),
            }
        }
        if let Some((on_submit_listener, on_submit_handler)) = &mut this.on_submit {
            match Pin::new(on_submit_listener).poll_next(cx) {
                Poll::Ready(Some(_ev)) => on_submit_handler(TextInputEvent {
                    buffer: this.buffer.clone(),
                }),
                _ => (),
            }
        }
        if let Some((on_blur_listener, on_blur_handler)) = &mut this.on_blur {
            match Pin::new(on_blur_listener).poll_next(cx) {
                Poll::Ready(Some(_ev)) => on_blur_handler(TextInputEvent {
                    buffer: this.buffer.clone(),
                }),
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
        let input: gtk::Widget;
        let buffer;
        let on_submit;
        match self.multiline {
            true => {
                let text_buffer = gtk::TextBuffer::new(None);
                let text_view = gtk::TextView::new();
                text_view.set_buffer(Some(&text_buffer));
                on_submit = None;
                input = text_view.upcast();
                buffer = InputBuffer::MultiLine(text_buffer);
            }
            false => {
                let entry_buffer = gtk::EntryBuffer::new(None);
                let entry = gtk::Entry::new();
                on_submit = (!is_dummy(self.on_submit)).then(|| {
                    let listener = EventHandler::new();
                    let receiver = listener.get_receiver();
                    entry.connect_activate(move |_e| {
                        receiver.send(());
                    });
                    (listener, self.on_submit)
                });
                entry.set_buffer(&entry_buffer);
                input = entry.upcast();
                buffer = InputBuffer::OneLine(entry_buffer);
            }
        };
        let on_input = (!is_dummy(self.on_change_text)).then(|| {
            let listener = EventHandler::new();
            let receiver = listener.get_receiver();
            buffer.connect_changed(move || receiver.send(()));
            (listener, self.on_change_text)
        });
        let on_blur = (!is_dummy(self.on_blur)).then(|| {
            let listener = EventHandler::new();
            let receiver = listener.get_receiver();
            let focus_controler = gtk::EventControllerFocus::new();
            input.add_controller(&focus_controler);
            focus_controler.connect_contains_focus_notify(move |s| {
                if !s.contains_focus() {
                    receiver.send(());
                }
            });
            (listener, self.on_blur)
        });
        ElementFuture::new(
            TextInputFuture {
                obs: self.text,
                change_fut: self.text.until_change(),
                buffer,
                set: false,
                on_input,
                on_submit,
                on_blur,
            },
            WrappedWidget {
                widget: input.upcast(),
                inner_widget: None,
                op: crate::widget::WidgetOp::NoChild,
            },
        )
    }
}
