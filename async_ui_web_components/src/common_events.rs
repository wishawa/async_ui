use futures_core::Stream;
use std::{borrow::Cow, future::Future, rc::Rc};

use wasm_bindgen::JsCast;

use super::{event_handling::EventsHandler, IsHtmlElement, WrappedHtmlElement};

impl<E: IsHtmlElement> WrappedHtmlElement<E> {
    // Some weird interaction with wasm-bindgen is blocking Rust-Analyzer
    // from inferring the Future output type when we return the concrete NextEvent<V>.
    // So we are returning impl trait instead.
    // TODO: fix this.
    pub fn until<V: Unpin + JsCast>(
        &self,
        ev_type: Cow<'static, str>,
    ) -> impl Future<Output = V> + Stream<Item = V> {
        let mut handlers = self.handlers.borrow_mut();
        let handlers = handlers
            .get_or_insert_with(|| EventsHandler::new(self.element.as_ref().clone().into()));

        EventsHandler::on_event(Rc::clone(handlers), ev_type)
    }
}

macro_rules! make_event_impl {
    ($ev_name:literal, $func_name:ident, $ty:ty) => {
        pub fn $func_name(&self) -> impl Future<Output = $ty> + Stream<Item = $ty> {
            self.until($ev_name.into())
        }
    };
}
#[rustfmt::skip]
impl<E: IsHtmlElement> WrappedHtmlElement<E> {
	make_event_impl!("cancel", until_cancel, web_sys::Event);
	make_event_impl!("error", until_error, web_sys::Event);
	make_event_impl!("scroll", until_scroll, web_sys::Event);
	make_event_impl!("securitypolicyviolation", until_securitypolicyviolation, web_sys::Event);
	make_event_impl!("select", until_select, web_sys::Event);
	make_event_impl!("wheel", until_wheel, web_sys::WheelEvent);

	make_event_impl!("compositionend", until_compositionend, web_sys::CompositionEvent);
	make_event_impl!("compositionstart", until_compositionstart, web_sys::CompositionEvent);
	make_event_impl!("compositionupdate", until_compositionupdate, web_sys::CompositionEvent);

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
