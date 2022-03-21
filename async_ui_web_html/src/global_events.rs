use crate::elem::{Elem, HtmlTag};
use async_ui_reactive::singlethread::ChannelEntry;
use async_ui_spawn::wasm::start_executor;
use js_sys::Function;
use std::any::Any;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlElement;

macro_rules! impl_event_handler {
    ($name:ident, $evtype:ty) => {
        paste::paste! {
            impl<'a, H: HtmlTag + 'a> Elem<'a, H>
            {
                pub fn [<$name:snake>](mut self, value: ChannelEntry<web_sys::$evtype>) -> Self {
                    let clos: Closure<dyn FnMut(_) + 'static> = Closure::wrap(Box::new(move |e: web_sys::$evtype| {
                        value.send(e);
                        start_executor();
                    }) as Box<dyn FnMut(_)>);
                    let func: &Function = clos.as_ref().unchecked_ref();
                    let elem: &HtmlElement = self.elem.as_ref();
                    elem.[<set_ $name:lower>](Some(func));
                    self.extras.push(Box::new(clos) as Box<dyn Any>);
                    self
                }
            }

        }
    };
}
// Sources:
// https://developer.mozilla.org/en-US/docs/Web/API/GlobalEventHandlers
// https://github.com/DefinitelyTyped/DefinitelyTyped/blob/f96ee5b85fd3414e8af20eb80a825a65966dd5c9/types/react/index.d.ts#L1346

impl_event_handler!(OnClick, MouseEvent);
impl_event_handler!(OnDblClick, MouseEvent);
impl_event_handler!(OnAuxClick, MouseEvent);
impl_event_handler!(OnMouseDown, MouseEvent);
impl_event_handler!(OnMouseUp, MouseEvent);
impl_event_handler!(OnMouseMove, MouseEvent);
impl_event_handler!(OnMouseEnter, MouseEvent);
impl_event_handler!(OnMouseLeave, MouseEvent);
impl_event_handler!(OnMouseOver, MouseEvent);
impl_event_handler!(OnMouseOut, MouseEvent);

impl_event_handler!(OnKeyDown, KeyboardEvent);
impl_event_handler!(OnKeyUp, KeyboardEvent);

impl_event_handler!(OnFocus, FocusEvent);
impl_event_handler!(OnBlur, FocusEvent);

impl_event_handler!(OnScroll, UiEvent);

impl_event_handler!(OnWheel, WheelEvent);

impl_event_handler!(OnInput, InputEvent);
