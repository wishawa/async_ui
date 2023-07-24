// ANCHOR: example
use async_ui_web::{
    components::DynamicSlot, html::Button, join, shortcut_traits::ShortcutRenderStr,
};
use futures_lite::FutureExt; // for .boxed_local(), which converts a Future to a `Box<dyn Future>`
use gloo_timers::future::TimeoutFuture; // nice async interface to `setTimeout`

async fn show_button_and_remove() {
    let slot = DynamicSlot::new();

    slot.set_future(
        // put <button>I will disappear soon!</button> in the slot
        Button::new()
            .render("I will disappear soon!".render())
            .boxed_local(), // make it dynamically typed so we can put Futures of other types in the slot
    );

    // join two Futures:
    // * one to render the slot
    // * the other to manipulate the content of the slot
    join((
        slot.render(), // render the slot
        async {
            // wait 3 seconds
            TimeoutFuture::new(3000).await;
            // 👇 replace the button in the slot with a text
            slot.set_future("The button is gone!".render().boxed_local());

            // wait another 3 seconds
            TimeoutFuture::new(3000).await;
            // 👇 remove the text in the slot
            slot.clear_future();
        },
    ))
    .await;
}
// ANCHOR_END: example

// ANCHOR: extra-quiz-helper
use async_ui_web::race; // 👈 new!
use std::future::Future;

/// Run the given Future.
/// If it is still running after 3 seconds, just drop it and return.
async fn run_for_3_seconds(f: impl Future<Output = ()>) {
    race((
        f,                        // the Future to run
        TimeoutFuture::new(3000), // a Future that waits 3000 ms
    ))
    .await
}
// ANCHOR_END: extra-quiz-helper
// ANCHOR: extra-quiz
async fn show_button_and_remove_2() {
    run_for_3_seconds(
        // show <button>I will disappear soon!</button>
        Button::new().render("I will disappear soon!".render()),
    )
    .await;

    run_for_3_seconds(
        // the text
        "The button is gone!".render(),
    )
    .await;
}
// ANCHOR_END: extra-quiz
