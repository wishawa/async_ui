/*! For scheduling Rust-driven animations
 *
 * This is not related to CSS animations.
 */
use std::rc::Rc;

use observables::{cell::ReactiveCell, ObservableAsExt};
use wasm_bindgen::{prelude::Closure, JsCast};

use crate::WINDOW;

/** A "clock" for scheduling animation at the browser's preferred frame rate.
 *
 * ```rust
 * let animator = Animator::new();
 *
 * loop {
 *     let timestamp = animator.next_frame().await; // wait until the browser wants a frame
 *     // render something
 * }
 * ```
 */
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
        return self.cell.as_observable().get();
    }
}
