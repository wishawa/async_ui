# Async UI

A web UI framework where Futures are components.

## Async UI is...

* **Just async Rust**; if you know what Futures are and how to join them, you know 90% of Async UI already.
* **Transparent**; we favor imperative async code, avoiding callbacks and custom DSLs.
* **Flexible**; you get access to the entire Web API (through [web_sys](https://docs.rs/web-sys/latest/web_sys/)).

## Example Code: Hello World
```rust
async fn hello_world() {
    "Hello World".render().await;
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

## Example Code: Async Control Flow
```rust
async fn app() {
    let resource = loading_indicator(
        fetch_resource()
    ).await;
    show_resource(&resource).await;
}
```

## Design
WIP. Please check back later.