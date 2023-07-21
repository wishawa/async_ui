# All the HTML

Let's look at some more ways to build HTML structures with Async UI.

## Empty `<div>`

The previous subchapter ended with a note that `<div>`s can have children,
but `<input>`s cannot.

Still, it is perfectly fine to have `<div>` without children.
```html
<div></div>
```
So how do we do that in Async UI?
```rust
{{ #include ../../../examples/guide-project/src/building_ui/html.rs:div-empty }}
```

What is [`NoChild`](https://docs.rs/async_ui_web/latest/async_ui_web/struct.NoChild.html)?
It is a unit struct that implements Future. It puts nothing on the screen,
and it never finishes (just like `input.render()`).

## Other Elements
`Div` and `Input` are only two components in Async UI's suite of HTML components.
You can see the full list [here](https://docs.rs/async_ui_web/latest/async_ui_web/html/index.html).

## Text
How do we render an [HTML Text Node](https://developer.mozilla.org/en-US/docs/Web/API/Text)?

We have a trait that implements `.render()` on `&str`.
```rust
{{ #include ../../../examples/guide-project/src/building_ui/html.rs:text-node }}
```
We will learn how to do this without the magic trait in the next chapter.

---

# Quiz
How would you make a component that renders the following HTML?
```html
<div>
	<button>
		Hello World
	</button>
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