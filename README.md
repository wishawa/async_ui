# Async UI

A web UI framework where Futures are components.

## Overview (for the User)

Async UI is...
*   **Easy**; if you know what Futures are and how to join them, you know 90% of Async UI already.
*   **Just async Rust**; we leverage established patterns and primitives, avoiding DSL macros.
*   **Flexible**; you get direct access to the entire Web API (through [web_sys](https://docs.rs/web-sys/latest/web_sys/)).

[See hosted demos](https://wishawa.github.io/async_ui/demos/index.html)

[Get Started Now!](https://wishawa.github.io/async_ui/book/index.html)

## Overview (for the UI Framework Connoisseur)
*   **Async as UI Runtime**; the app is one long-running Future.
*   **Components are Futures**; composition is done by nesting and joining Futures.
*   **UI as Side-Effect**; running a Future displays its UI, dropping it removes that UI.

[Read more about the framework](https://wishawa.github.io/async_ui/book/in-depth/framework-design.html)

## Example Code: Hello World
```rust
async fn hello_world() {
    "Hello World".render().await;
}
```

## Example Code: Async Control Flow
```rust
async fn app() {
    let resource = loading_indicator(
        fetch_resource()
    ).await;
    show_resource(&resource).await;
}
```

## Example Code: Counter
```rust
async fn counter() {
    let mut count = 0;
    let value_text = Text::new();
    let incr_button = Button::new();
    join((
        value_text.render(),
        incr_button.render("Increment".render()),
        async {
            loop {
                value_text.set_data(&count.to_string());
                incr_button.until_click().await;
                count += 1;
            }
        },
    ))
    .await;
}
```
