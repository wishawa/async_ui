use std::rc::Rc;

use observables::{cell::ReactiveCell, ObservableAs, ObservableAsExt};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::WINDOW;

pub struct Animator {
    cell: Rc<ReactiveCell<f64>>,
    func: Closure<dyn Fn(f64)>,
}

impl Animator {
    pub fn new() -> Self {
        let cell = Rc::new(ReactiveCell::new(0f64));
        let cell_1 = cell.clone();
        let func = Closure::new(move |ts: f64| {
            *cell_1.borrow_mut() = ts;
        });
        Animator { func, cell }
    }
    pub async fn next_frame(&self) -> f64 {
        WINDOW.with(|win| {
            win.request_animation_frame(self.func.as_ref().unchecked_ref())
                .expect("request frame failed");
        });
        self.cell.as_observable().until_change().await;
        return *self.cell.as_observable().borrow_observable_as();
    }
}
