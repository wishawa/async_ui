use std::{
    cell::RefCell,
    future::{poll_fn, Future},
    pin::{pin, Pin},
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
    // a tiny "channel" implementation to send what change should be made to the slot
    channel: RefCell<(Waker, Message<F>)>,
}

enum Message<F> {
    // replace the Future rendering in the slot with this new Future
    Set(F),
    // remove the Future rendering in the slot
    Clear,
    // don't do anything
    NoChange,
}

impl<F: Future> Default for DynamicSlot<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Future> DynamicSlot<F> {
    /// Create a new `DynamicSlot`, containing no Future inside.
    pub fn new() -> Self {
        Self {
            channel: RefCell::new((dummy_waker(), Message::NoChange)),
        }
    }

    /// Render the Future set inside the `DynamicSlot` here.
    ///
    /// The UI will update when you call
    /// [set_future][Self::set_future] or [clear_future][Self::clear_future].
    ///
    /// This async method never completes.
    ///
    /// This method should only ba called once. It may misbehave otherwise.
    pub async fn render(&self) {
        // this is where the Future lives
        let mut fut_slot: Pin<&mut Option<F>> = pin!(None);
        poll_fn(|cx| {
            {
                let mut channel = self.channel.borrow_mut();
                match std::mem::replace(&mut channel.1, Message::NoChange) {
                    Message::Set(new_fut) => fut_slot.set(Some(new_fut)),
                    Message::Clear => fut_slot.set(None),
                    Message::NoChange => {}
                }
                // if the Waker in the channel is outdated, update it
                if !channel.0.will_wake(cx.waker()) {
                    channel.0 = cx.waker().to_owned();
                }
            }
            // poll the Future in the slot, if there is one
            if let Some(fut) = fut_slot.as_mut().as_pin_mut() {
                let _ = fut.poll(cx);
            }
            Poll::Pending
        })
        .await
    }

    /// Set the future to render in the slot, dropping the previous one.
    pub fn set_future(&self, fut: F) {
        let mut channel = self.channel.borrow_mut();
        channel.1 = Message::Set(fut);
        channel.0.wake_by_ref();
    }

    /// Remove the future currently rendering in the slot (if any).
    pub fn clear_future(&self) {
        let mut channel = self.channel.borrow_mut();
        channel.1 = Message::Clear;
        channel.0.wake_by_ref();
    }
}
