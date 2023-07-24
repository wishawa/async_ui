use std::{
    cell::RefCell,
    future::{poll_fn, Future},
    pin::pin,
    task::{Poll, Waker},
};

use async_ui_internal_utils::dummy_waker::dummy_waker;

/**
A "slot" that renders a future which you can change dynamically.

```
# use async_ui_web::{components::DynamicSlot, join, html::Button, prelude_traits::*};
# let _ = async {
let slot = DynamicSlot::new();
let btn = Button::new();
let mut count = 0;
join((
    slot.render(),
    btn.render("increment".render()),
    async {
        loop {
            // set the future in the slot
            // to be a future that renders the `count` text
            slot.set_future(count.to_string().render());
            btn.until_click().await;
            count += 1;
        }
    }
)).await;
# };
```
 */
pub struct DynamicSlot<F: Future> {
    next: RefCell<(Waker, Next<F>)>,
}

enum Next<F> {
    Set(F),
    Clear,
    NoChange,
}

impl<F: Future> Default for DynamicSlot<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Future> DynamicSlot<F> {
    /// Create a new `Dynamic`, containing no Future inside.
    pub fn new() -> Self {
        Self {
            next: RefCell::new((dummy_waker(), Next::NoChange)),
        }
    }

    /// Render the Future set inside the `Dynamic` here.
    ///
    /// The UI will update when you call
    /// [set_future][Dynamic::set_future] or [clear_future][Dynamic::clear_future].
    ///
    /// This async method never completes.
    ///
    /// This method should only ba called once. It may misbehave otherwise.
    pub async fn render(&self) {
        let mut fut = pin!(None);
        poll_fn(|cx| {
            {
                let mut next = self.next.borrow_mut();
                match std::mem::replace(&mut next.1, Next::NoChange) {
                    Next::Set(new_fut) => fut.set(Some(new_fut)),
                    Next::Clear => fut.set(None),
                    Next::NoChange => {}
                }
                if !next.0.will_wake(cx.waker()) {
                    next.0 = cx.waker().to_owned();
                }
            }
            if let Some(fut) = fut.as_mut().as_pin_mut() {
                let _ = fut.poll(cx);
            }
            Poll::Pending
        })
        .await
    }

    /// Set the future to render in the slot, dropping the previous one.
    pub fn set_future(&self, fut: F) {
        let mut bm = self.next.borrow_mut();
        bm.1 = Next::Set(fut);
        bm.0.wake_by_ref();
    }
    /// Remove the future currently rendering in the slot (if any).
    pub fn clear_future(&self) {
        let mut bm = self.next.borrow_mut();
        bm.1 = Next::Clear;
        bm.0.wake_by_ref();
    }
}
