# Building Nested HTML

In the previous subchapter, we were able to render
```html
<input />
```

Let's do something more complicated. Let's render
```html
<div>
	<input />
</div>
```

```rust,noplayground
{{ #include ../../../examples/guide-project/src/building_ui/html.rs:div-with-input }}
```

Notice that **there is only one `.await`** in the example above.

```rust,ignore
// ❌❌ this wouldn't work
div.render(
	input.render().await
	//             👆👆
	// this await above is incorrect
).await;
```

It is very important to understand why using two `await`s here is incorrect.
The two concepts behind this rule are core concepts in Async UI.

### `.render(_)` *wraps* its argument Future
The signature of `div.render(_)` is
```rust,ignore
fn render<F: Future>(&self, c: F) -> impl Future<Output = F::Output>;
```
It **takes a Future object** and returns a "wrapped" Future object.
The new Future places a `<div>` on the screen, and if the inner Future
put anything on the screen, that thing will appear inside the `<div>`.

Our `input.render()` is a Future that puts an `<input>` on the screen.
We wrap it with `div.render(_)`, giving us
```html
<div>
	<input />
</div>
```

### UI Futures are *long-lived*
`input.render()` returns a Future object that *never finishes*.
If we `await` it, our code would just be stuck there.
> #### Why does the Future never finish?
> The `<input>` element **stays on the screen for as long as the Future is
> running**. We wouldn't want the element to suddenly disappear!
> 
> In later chapters, we will learn how to remove rendered elements.

> #### Does `div.render(_)` also never finish?
> `div.render(_)` finishes when the inner Future finishes.
> This is why we say `.render(_)` "wraps" the inner Future.
> 
> In our case, though, the inner Future never finishes anyway

> #### Why does `div.render(_)` take an argument while `input.render()` doesn't?
> [Per the HTML spec](https://html.spec.whatwg.org/multipage/syntax.html#void-elements),
> `<div>` elements are allowed to have children, while `<input>` elements are not.
