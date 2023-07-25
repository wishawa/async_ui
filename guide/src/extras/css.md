# CSS Styling

## Class List Shortcuts
You can already do basic styling by accessing your elements `style` property
([MDN doc](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/style),
[web-sys doc](https://docs.rs/web-sys/latest/web_sys/struct.HtmlElement.html#method.style)).

You can also already do CSS styling by setting classnames
for your elements with the `classList`
([MDN doc](https://developer.mozilla.org/en-US/docs/Web/API/Element/classList),
[web-sys doc](https://docs.rs/web-sys/latest/web_sys/struct.Element.html#method.class_list))
```rust
{{ #include ../../../examples/guide-project/src/extras/css.rs:no-shortcut }}
```

For extra convenience, Async UI provides a few traits to make styling code a bit
less verbose.

### `ShortcutClassList`
```rust
{{ #include ../../../examples/guide-project/src/extras/css.rs:shortcut-imperative }}
```
There are a few more methods available.
View the documentation [here](https://docs.rs/async_ui_web/latest/async_ui_web/shortcut_traits/trait.ShortcutClassList.html).
### `ShortcutClassListBuilder`
```rust
{{ #include ../../../examples/guide-project/src/extras/css.rs:shortcut-builder }}
```
View the documentation [here](https://docs.rs/async_ui_web/latest/async_ui_web/shortcut_traits/trait.ShortcutClassListBuilder.html).

## Linking CSS

You can already add your CSS content by either
*	linking it in your `index.html` (with a `<link />` tag), or
*	constructing a `<style>` element with Async UI.

Async UI provide an extra mechanism that let you write your CSS in Rust file.
```rust
{{ #include ../../../examples/guide-project/src/extras/css.rs:embedded-css }}
```
With this method, you don't have to worry about linking the CSS - 
it is done automatically by the macro.

The macro will add random **postfix** to your CSS class names so that
you don't have to worry about name collision.

The macro expose those postfixed class names as Rust `&str` constants you can use.