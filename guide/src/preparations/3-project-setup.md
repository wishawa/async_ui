# Project Setup

### Skip by Cloning from Template

If you want to skip all the setup steps below, simply clone the template.
```shell
git clone TODO TODO TODO
```

### Setting Up an Async UI App

An Async UI project is just a Rust project.

```shell
cargo new --lib guide-project
cd guide-project
```

Now you need to set up for WASM.
Modify your `Cargo.toml` as so

```toml
[package]
name = "guide-project"
version = "0.1.0"
edition = "2021"

# 👇 Add this section so that your app can be compiled to WASM
[lib]
crate-type = ["cdylib"]

[dependencies]
# ...
```

Now add Async UI as a dependency.
You'll also need [wasm-bindgen](https://docs.rs/wasm-bindgen/latest/wasm_bindgen/).

I also recommend adding a utility crate for working with Futures.
In this tutorial I will use [futures-lite](https://docs.rs/futures-lite/latest/futures_lite/),
but feel free to use [futures](https://docs.rs/futures/latest/futures/) instead if you want.
```toml
# ...
[dependencies]
async_ui_web = "0.2.0" # 👈 Async UI
wasm-bindgen = "0.2.87" # 👈 For interfacing with JavaScript
futures-lite = "1.13.0" # 👈 Helper
```

You also need an HTML "boilerplate" for the WebAssembly.
Create a file `index.html` in your project directory (**not** inside `src/`).
Put in the following content.
```html
{{#include ../../../examples/guide-project/index.html}}
```

Almost there! Now we'll add an entrypoint for our application.
Open your `src/lib.rs` and put in the following content.
```rust
{{#include ../../../examples/guide-project/src/lib.rs::8}}
```

Now run it!
```shell
wasm-pack build --dev --target web && microserver
```
Open your browser at [localhost:9090](http://localhost:9090) to see the result.
You should just get an empty page (after all we haven't put in any UI code yet).
Check the console to make sure there are no errors.