use std::{
    cell::RefCell,
    future::{poll_fn, Future},
    pin::Pin,
    task::{Poll, Waker},
};

use async_ui_internal_utils::dummy_waker::dummy_waker;

enum Next<F> {
    Set(F),
    Clear,
    NoChange,
}

/**
A "slot" that renders a future which you can change dynamically.

```
# use async_ui_web::{Dynamic, join, components::Button, prelude_traits::*};
# let _ = async {
let slot = Dynamic::new();
let btn = Button::new();
let mut count = 0;
join((
    slot.render(),
    btn.render("increment".render()),
    async {
        loop {
            slot.set_future(count.to_string().render());
            btn.until_click().await;
            count += 1;
        }
    }
)).await;
# };
```

 */
pub struct Dynamic<F: Future + Unpin> {
    next: RefCell<(Waker, Next<F>)>,
}

impl<F: Future + Unpin> Dynamic<F> {
    /// Create a new `Dynamic`, containing no future inside.
    pub fn new() -> Self {
        Self {
            next: RefCell::new((dummy_waker(), Next::NoChange)),
        }
    }

    /// Render the future inside the `Dynamic` here.
    ///
    /// The UI will update when you call
    /// [set_future][Dynamic::set_future] or [clear_future][Dynamic::clear_future].
    pub async fn render(&self) {
        let mut fut = scopeguard::guard(None, |fut| {
            if let Some(fut) = fut {
                self.next.borrow_mut().1 = Next::Set(fut);
            }
        });
        poll_fn(|cx| {
            {
                let mut next = self.next.borrow_mut();
                match std::mem::replace(&mut next.1, Next::NoChange) {
                    Next::Set(new_fut) => *fut = Some(new_fut),
                    Next::Clear => *fut = None,
                    Next::NoChange => {}
                }
                if !next.0.will_wake(cx.waker()) {
                    next.0 = cx.waker().to_owned();
                }
            }
            if let Some(fut) = fut.as_mut() {
                let _ = Pin::new(fut).poll(cx);
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
