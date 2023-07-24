use async_ui_web::html::Anchor;
use async_ui_web::{html::Button, join, shortcut_traits::ShortcutRenderStr};

// ANCHOR: quick-example
use async_ui_web::event_traits::EmitElementEvent; // 👈 new import!

async fn quick_example() {
    let button = Button::new();
    join((
        // render the button
        button.render("Click me!".render()),
        async {
            button.until_click().await; // 👈 wait for the button to be clicked

            // handle click
        },
    ))
    .await;
}
// ANCHOR_END: quick-example

// ANCHOR: return-type
async fn return_type() {
    let link = Anchor::new();
    link.set_href("https://example.com/");
    join((
        // render the link
        link.render("I'm a link!".render()),
        async {
            let ev = link.until_click().await; // 👈 wait for the button to be clicked

            ev.prevent_default(); // 👈 use the event object
                                  // we called preventDefault so example.com won't be opened
        },
    ))
    .await;
}
// ANCHOR_END: return-type

// ANCHOR: prevent-default-stream
use futures_lite::StreamExt; // 👈 new!

async fn prevent_default_with_stream() {
    let link = Anchor::new();
    link.set_href("https://example.com/");
    join((
        // render the link
        link.render("I'm a link!".render()),
        // for each click event, `preventDefault` it
        link.until_click().for_each(|ev| {
            ev.prevent_default();
        }),
    ))
    .await;
}
// ANCHOR_END: prevent-default-stream
