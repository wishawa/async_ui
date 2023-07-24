# All the HTML

Let's look at some more ways to build HTML structures with Async UI.

## Empty `<div>`

The previous subchapter ended with a note that `<div>`s can have children,
but `<input>`s cannot.

Still, it is perfectly fine to have `<div>` without children.
```html
<div></div>
```
The code to do that would be
```rust
{{ #include ../../../examples/guide-project/src/building_ui/html.rs:div-empty }}
```

Here, [`NoChild`](https://docs.rs/async_ui_web/latest/async_ui_web/struct.NoChild.html)
is a unit struct that implements Future. It puts nothing on the screen,
and it never finishes (just like `input.render()`).

## Text
We can also render [HTML Text Nodes](https://developer.mozilla.org/en-US/docs/Web/API/Text).

There is a trait that implements `.render()` on `&str`.
```rust
{{ #include ../../../examples/guide-project/src/building_ui/html.rs:text-node }}
```
The trait implementation is based on [the Text component](https://docs.rs/async_ui_web/latest/async_ui_web/html/struct.Text.html).
You can use that manually too (the next chapter will touch more on this).

## Other Elements
`Div`, `Input`, and Text Node are only three components in Async UI's suite of HTML components.
You can see the full list [here](https://docs.rs/async_ui_web/latest/async_ui_web/html/index.html).

All of them have the same form:
a struct that can be constructed by `Type::new()`
and can be rendered with the async `render()` method.

---

# Quiz
How would you make a component that renders the following HTML?
```html
<div>
	<button>Hello World</button>
</div>
```
If you haven't been running examples along the guide so far,
now is the time 😤😤😤.
Write out your answer to the quiz. Run it in your browser to check.

<details>
<summary>Click to view solution</summary>

```rust
{{ #include ../../../examples/guide-project/src/building_ui/html.rs:exercise }}
```
</details>