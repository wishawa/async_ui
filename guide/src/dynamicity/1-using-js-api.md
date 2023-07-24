# Using the JS API

JavaScript provide API to manipulate HTML elements.
Async UI provides the same API, translated to Rust via wasm-bindgen
and the web-sys crate. Anything you can do in JavaScript,
you should be able to do here.

Let's start simple: we'll set the placeholder text in a text `<input />` field.
```rust
{{ #include ../../../examples/guide-project/src/dynamicity/js_api.rs:input-placeholder }}
```
The `.set_placeholder(_)` method used is [from web_sys](https://docs.rs/web-sys/latest/web_sys/struct.HtmlInputElement.html#method.set_placeholder).
It is implemented on [`web_sys::HtmlInputElement`](https://docs.rs/web-sys/latest/web_sys/struct.HtmlInputElement.html),
which is web-sys' translation of the [same JS API](https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement)
(`x.set_placeholder(y);` is equal to `x.placeholder = y;` in JS).

You can call the method on [`async_ui_web::html::Input`](https://docs.rs/async_ui_web/latest/async_ui_web/html/struct.Input.html)
because our `Input` derefs to `web_sys::HtmlInputElement`.

Other Async UI HTML components deref to their web-sys counterpart too.
All the methods are listed in [the documentation](https://docs.rs/async_ui_web/latest/async_ui_web/html/index.html).

## Example: Countdown
This example update the content of an HTML Text Node every second.

We'll use [gloo-timers](https://docs.rs/gloo-timers/latest/gloo_timers/index.html)
to conveniently access [JavaScript `setTimeout`](https://developer.mozilla.org/en-US/docs/Web/API/setTimeout).
Add this in your `Cargo.toml` dependencies

```toml
gloo-timers = { version = "0.2.6", features = ["futures"] }
```

> #### Leveraging the Ecosystem
> The `gloo-timers` crate isn't related to Async UI,
> but since it provides an async API, we can use it in our Async UI app
> very easily.
> 
> This is one of the strengths of Async UI: it integrates very well with
> any async Rust code.

Now, for our countdown code
```rust
{{ #include ../../../examples/guide-project/src/dynamicity/js_api.rs:countdown }}
```

*Note*: Our example here is just for demonstration.
For a correct countdown implementation, use [`setInterval`](https://developer.mozilla.org/en-US/docs/Web/API/setInterval)
instead of `setTimeout`.
