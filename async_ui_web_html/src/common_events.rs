use crate::events::EventFutureStream;

use async_ui_web_core::dom::{Element, HtmlElement};

macro_rules! make_event_impl {
    ($ev_name:literal, $func_name:ident, $ty:ty, $link:tt) => {
        #[must_use = "the returned object is a Future+Stream that does nothing unless polled"]
        #[doc = "Like [until_event][EmitEvent::until_event] for the `"]
        #[doc = $ev_name]
        #[doc = "` event."]
        #[doc = "See"]
        #[doc = $link]
        #[doc = "."]
        fn $func_name(&self) -> EventFutureStream<$ty> {
            #[cfg(feature = "csr")]
            {
                use crate::events::EmitEvent;
                self.as_ref().until_event($ev_name.into())
            }
            #[cfg(feature = "ssr")]
            {
                EventFutureStream::new_dummy($ev_name.into())
            }
        }
    };
}

/// Subscribe to common events emitted by HTML `Element`s such as `click` or `scroll`.
///
/// See [MDN Web Docs for the events](https://developer.mozilla.org/en-US/docs/Web/API/Element#events).
#[rustfmt::skip]
pub trait EmitElementEvent: AsRef<Element> {
    make_event_impl!("cancel", until_cancel, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mousemove_event)");
    make_event_impl!("error", until_error, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/error_event)");
    make_event_impl!("scroll", until_scroll, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/scroll_event)");
    make_event_impl!("securitypolicyviolation", until_securitypolicyviolation, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/securitypolicyviolation_event)");
    make_event_impl!("select", until_select, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/select_event)");
    make_event_impl!("wheel", until_wheel, web_sys::WheelEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/wheel_event)");

    make_event_impl!("compositionend", until_compositionend, web_sys::CompositionEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionend_event)");
    make_event_impl!("compositionstart", until_compositionstart, web_sys::CompositionEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionstart_event)");
    make_event_impl!("compositionupdate", until_compositionupdate, web_sys::CompositionEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionupdate_event)");

    make_event_impl!("blur", until_blur, web_sys::UiEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/blur_event)");
    make_event_impl!("focus", until_focus, web_sys::UiEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/focus_event)");
    make_event_impl!("focusin", until_focusin, web_sys::UiEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/focusin_event)");
    make_event_impl!("focusout", until_focusout, web_sys::UiEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/focusout_event)");

    make_event_impl!("fullscreenchange", until_fullscreenchange, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/fullscreenchange_event)");
    make_event_impl!("fullscreenerror", until_fullscreenerror, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/fullscreenerror_event)");

    make_event_impl!("keydown", until_keydown, web_sys::KeyboardEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/keydown_event)");
    make_event_impl!("keyup", until_keyup, web_sys::KeyboardEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/keyup_event)");

    make_event_impl!("auxclick", until_auxclick, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/auxclick_event)");
    make_event_impl!("click", until_click, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/click_event)");
    make_event_impl!("contextmenu", until_contextmenu, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/contextmenu_event)");
    make_event_impl!("dblclick", until_dblclick, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/dblclick_event)");
    make_event_impl!("mousedown", until_mousedown, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mousedown_event)");
    make_event_impl!("mouseenter", until_mouseenter, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseenter_event)");
    make_event_impl!("mouseleave", until_mouseleave, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseleave_event)");
    make_event_impl!("mousemove", until_mousemove, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mousemove_event)");
    make_event_impl!("mouseout", until_mouseout, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseout_event)");
    make_event_impl!("mouseover", until_mouseover, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseover_event)");
    make_event_impl!("mouseup", until_mouseup, web_sys::MouseEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/mouseup_event)");

    make_event_impl!("touchcancel", until_touchcancel, web_sys::TouchEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchcancel_event)");
    make_event_impl!("touchend", until_touchend, web_sys::TouchEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchend_event)");
    make_event_impl!("touchmove", until_touchmove, web_sys::TouchEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchmove_event)");
    make_event_impl!("touchstart", until_touchstart, web_sys::TouchEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchstart_event)");

    // make_event_impl!("copy", event_copy, web_sys::ClipboardEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/copy_event)");
    // make_event_impl!("cut", event_cut, web_sys::ClipboardEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/cut_event)");
    // make_event_impl!("paste", event_paste, web_sys::ClipboardEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/Element/paste_event)");

}

impl EmitElementEvent for Element {}

/// Subscribe to common events emitted by HTML `HTMLElement`s such as `input` or `drag`.
/// 
/// See [MDN Web Docs for the events](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement#events).
#[rustfmt::skip]
pub trait EmitHtmlElementEvent: AsRef<HtmlElement> {
    make_event_impl!("input", until_input, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/input_event)");
    make_event_impl!("change", until_change, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/change_event)");

    make_event_impl!("drag", until_drag, web_sys::DragEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/drag_event)");
    make_event_impl!("dragend", until_dragend, web_sys::DragEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragend_event)");
    make_event_impl!("dragenter", until_dragenter, web_sys::DragEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragenter_event)");
    make_event_impl!("dragleave", until_dragleave, web_sys::DragEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragleave_event)");
    make_event_impl!("dragover", until_dragover, web_sys::DragEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragover_event)");
    make_event_impl!("dragstart", until_dragstart, web_sys::DragEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/dragstart_event)");
    make_event_impl!("drop", until_drop, web_sys::DragEvent, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/drop_event)");

    make_event_impl!("load", until_load, web_sys::Event, "[MDN documentation for this event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/load_event)");
}

impl EmitHtmlElementEvent for HtmlElement {}
