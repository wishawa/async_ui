use std::rc::Rc;

use async_ui_internal_utils::reactive_cell::ReactiveCell;
use js_sys::Function;
use wasm_bindgen::{convert::FromWasmAbi, prelude::Closure, JsCast};

pub struct CallbackToFuture<I, V> {
    closure: Closure<dyn Fn(I)>,
    pub signal: Rc<ReactiveCell<Option<V>>>,
}

impl<I: 'static + FromWasmAbi, V: 'static> CallbackToFuture<I, V> {
    pub fn new(mapper: impl 'static + Fn(I) -> V) -> Self {
        let signal = Rc::new(ReactiveCell::new(None));
        let signal_cloned = signal.clone();
        let closure = Closure::new(move |input: I| {
            *signal_cloned.borrow_mut() = Some(mapper(input));
        });
        Self { closure, signal }
    }
    pub fn get_function(&self) -> &Function {
        self.closure.as_ref().unchecked_ref()
    }
}
