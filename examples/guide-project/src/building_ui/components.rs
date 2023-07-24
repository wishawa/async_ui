use async_ui_web::{
    html::{Button, Div, Input, Span},
    join,
    shortcut_traits::ShortcutRenderStr, // for rendering text as Text Nodes
};

// ANCHOR: many-spans
async fn lots_of_span() {
    Div::new() // the wrapping <div>
        .render(join((
            // the <button> at the top
            Button::new().render("Hello World".render()),
            // the 100 <span>s, made by joining a vec of 100 Futures
            join(
                (1..=100)
                    .map(|number| Span::new().render(number.to_string().render()))
                    .collect::<Vec<_>>(),
            ),
            // the <input> at the end
            Input::new().render(),
        )))
        .await;
}
// ANCHOR_END: many-spans

// ANCHOR: many-spans-componentified
pub async fn lots_of_span_2() {
    Div::new()
        .render(join((
            Button::new().render("Hello World".render()),
            hundred_spans(),
            Input::new().render(),
        )))
        .await;
}
// ANCHOR_END: many-spans-componentified
// ANCHOR: many-spans-components
async fn hundred_spans() {
    join((1..=100).map(one_span).collect::<Vec<_>>()).await;
}
async fn one_span(number: i32) {
    Span::new().render(number.to_string().render()).await;
}
// ANCHOR_END: many-spans-components

// ANCHOR: type-component
struct HelloWorld;
impl HelloWorld {
    async fn render(&self) {
        "Hello World".render().await;
    }
}
async fn app() {
    let hello_world = HelloWorld;
    hello_world.render().await;
}
// ANCHOR_END: type-component
