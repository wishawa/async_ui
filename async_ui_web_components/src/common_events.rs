use crate::events::{EmitEvent, NextEvent};
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

macro_rules! make_event_impl {
    ($ev_name:literal, $func_name:ident, $ty:ty) => {
        fn $func_name(&self) -> NextEvent<$ty> {
            self.as_ref().until_event($ev_name.into())
        }
    };
}

pub trait EmitElementEvent: AsRef<HtmlElement> {
    make_event_impl!("cancel", until_cancel, web_sys::Event);
    make_event_impl!("error", until_error, web_sys::Event);
    make_event_impl!("scroll", until_scroll, web_sys::Event);
    make_event_impl!(
        "securitypolicyviolation",
        until_securitypolicyviolation,
        web_sys::Event
    );
    make_event_impl!("select", until_select, web_sys::Event);
    make_event_impl!("wheel", until_wheel, web_sys::WheelEvent);

    make_event_impl!(
        "compositionend",
        until_compositionend,
        web_sys::CompositionEvent
    );
    make_event_impl!(
        "compositionstart",
        until_compositionstart,
        web_sys::CompositionEvent
    );
    make_event_impl!(
        "compositionupdate",
        until_compositionupdate,
        web_sys::CompositionEvent
    );

    make_event_impl!("blur", until_blur, web_sys::UiEvent);
    make_event_impl!("focus", until_focus, web_sys::UiEvent);
    make_event_impl!("focusin", until_focusin, web_sys::UiEvent);
    make_event_impl!("focusout", until_focusout, web_sys::UiEvent);

    make_event_impl!("fullscreenchange", until_fullscreenchange, web_sys::Event);
    make_event_impl!("fullscreenerror", until_fullscreenerror, web_sys::Event);

    make_event_impl!("keydown", until_keydown, web_sys::KeyboardEvent);
    make_event_impl!("keyup", until_keyup, web_sys::KeyboardEvent);

    make_event_impl!("auxclick", until_auxclick, web_sys::MouseEvent);
    make_event_impl!("click", until_click, web_sys::MouseEvent);
    make_event_impl!("contextmenu", until_contextmenu, web_sys::MouseEvent);
    make_event_impl!("dblclick", until_dblclick, web_sys::MouseEvent);
    make_event_impl!("mousedown", until_mousedown, web_sys::MouseEvent);
    make_event_impl!("mouseenter", until_mouseenter, web_sys::MouseEvent);
    make_event_impl!("mouseleave", until_mouseleave, web_sys::MouseEvent);
    make_event_impl!("mousemove", until_mousemove, web_sys::MouseEvent);
    make_event_impl!("mouseout", until_mouseout, web_sys::MouseEvent);
    make_event_impl!("mouseover", until_mouseover, web_sys::MouseEvent);
    make_event_impl!("mouseup", until_mouseup, web_sys::MouseEvent);

    make_event_impl!("touchcancel", until_touchcancel, web_sys::TouchEvent);
    make_event_impl!("touchend", until_touchend, web_sys::TouchEvent);
    make_event_impl!("touchmove", until_touchmove, web_sys::TouchEvent);
    make_event_impl!("touchstart", until_touchstart, web_sys::TouchEvent);

    // make_event_impl!("copy", event_copy, web_sys::ClipboardEvent);
    // make_event_impl!("cut", event_cut, web_sys::ClipboardEvent);
    // make_event_impl!("paste", event_paste, web_sys::ClipboardEvent);
}

impl EmitElementEvent for HtmlElement {}

pub trait EmitEditEvent: AsRef<HtmlElement> {
    make_event_impl!("input", until_input, web_sys::Event);
    make_event_impl!("change", until_change, web_sys::Event);
}

impl EmitEditEvent for HtmlInputElement {}
impl EmitEditEvent for HtmlTextAreaElement {}
impl EmitEditEvent for HtmlSelectElement {}
