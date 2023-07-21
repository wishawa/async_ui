#![allow(dead_code)]

// ANCHOR: div-with-input
use async_ui_web::html::{Div, Input}; // import Div and Input components

async fn my_input_field() {
    let div = Div::new();
    let input = Input::new();

    // render the UI!
    div.render(
        input.render(), // input is inside the div
    )
    .await;
}
// ANCHOR_END: div-with-input

// ANCHOR: div-empty
use async_ui_web::NoChild; // 👈 new import!

async fn just_div() {
    let div = Div::new();

    div.render(NoChild).await;
}
// ANCHOR_END: div-empty

// ANCHOR: text-node
use async_ui_web::shortcut_traits::ShortcutRenderStr; // 👈 new import!

async fn hello_world() {
    "Hello World".render().await;
}
// ANCHOR_END: text-node

// ANCHOR: exercise
use async_ui_web::html::Button;

async fn quiz() {
    Div::new()
        .render(Button::new().render("Hello World".render()))
        .await;
}
// ANCHOR_END: exercise
