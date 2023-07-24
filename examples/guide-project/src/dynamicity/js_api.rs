use async_ui_web::html::Input;

// ANCHOR: input-placeholder
async fn input_with_placeholder() {
    let input = Input::new();
    input.set_placeholder("enter something"); // 👈 set the placeholder
    input.render().await;
}
// ANCHOR_END: input-placeholder

// ANCHOR: countdown
use async_ui_web::{html::Text, join};
use gloo_timers::future::TimeoutFuture; // nice async interface to `setTimeout`

async fn countdown(mut seconds: i32) {
    let text = Text::new(); // create an HTML text node

    // join two Futures:
    // * one to render the text node
    // * the other to keep updating the content of the node
    join((
        text.render(), // render the text node
        async {
            while seconds > 0 {
                // 👇 Set the content of the text node. This is `text.data = "..."` in JS.
                text.set_data(&seconds.to_string());

                // wait 1 second
                TimeoutFuture::new(1000).await;

                // decrement the count
                seconds -= 1;
            }
            // count has reached zero!
            text.set_data("boom!");
        },
    ))
    .await;
}
// ANCHOR_END: countdown
