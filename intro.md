# Async-UI: a Rust UI library where everything is a Future

This blog post is intended for audience with some experiences in async Rust. It assumes you know what a [Future](https://doc.rust-lang.org/std/future/trait.Future.html) is!

## Why Async for UIs?

UI components are *retained*: they run some code, stay there and wait for some events (such as user interaction), and then run some more code to handle those events.

Rust's Futures behave in the same way: they run some code when first polled, stay there to wait for events, and then get polled again to run more code to handle those events.

This makes Future a good abstraction to build UIs on. And so I made Async-UI — a UI framework where everything is a Future.

## Components are Futures

```rust
async fn hello_world() {
	(Text {
		text: &"Hello World!"
	}).await;
}
```

Here `Text` is a built-in component provided by the library. It is a struct that implements `Future`. To render it you simply await it. The async function we just made is also a component. To render it, just `hello_world().await`.

## Easily Compose Components
Now what if you want to render two or more things at once? You can put many Futures together into a `Fragment`. You can then pass the `Fragment` around and await it where you want to render the things you put in. Think of `Fragment` as a glorified `Vec<Box<dyn Future>>`.

```rust
async fn hello_world_2() {
	Fragment::new((
		hello_world(),
		Button {
			children: Fragment::new((
				Text {
					text: &"Say hello back"
				},
			)),
			on_press: &mut |_ev: PressEvent| {
				todo!()
			},
			..Default::default()
		}
	)).await;
}
```

Here, we are rendering the "Hello World!" and a button next to it. Inside the button we have the text "Say hello back".

Pretty simple right? Just create components with async functions (or structs, or async blocks), combine them with `Fragment`, and await them where you want them to render.

## Bring your own Reactivity — or use ours
The core of async-ui doesn't know anything about reactivity. It's just async Rust! You can use channels ([async-channel](https://crates.io/crates/async-channel) is a great crate). You can use the excellent [futures-signals](https://crates.io/crates/futures-signals) crate. You can use whatever works in async Rust.

The built-in components support reactivity through a simple interface provided by the `observables` crate. The crate provides basic reactivity: `ReactiveCell<T>`.

```rust
async fn counter() {
	let mut count = 0;

	// ReactiveCell is for ReactiveCell! It is like a RefCell that you can subscribe to.
	let count_string = ReactiveCell::new(count.to_string());

	Fragment::new((
		Text {
			// When count_string changes, the text will change.
			text: &count_string.as_observable(),
		},
		Button {
			children: Fragment::new((
				Text: {
					text: &"+",
				},
			)),
			on_press: &mut |_press_event| {
				// Upon press, increment count and update the string accordingly.
				count += 1;
				*count_string.borrow_mut() = count.to_string();
			},
		}
	)).await;
}
```

For advanced reactivity and state management, I'm not sure what model will fit best with Async-UI yet, but I've been experimenting with [X-Bow](TODO ADD LINK).

## Benefits of the Async Model
### Lifetime-Friendly


## Platforms
Async-UI currently works on **Web** (`async_ui_web`) and **GTK 4** (`async_ui_gtk`).

## More Coming!

Async-UI is **not ready yet**. The core idea is complete, but lots of work remain to be done on the built-in components.

In the next few months, I'll continue to write more blog posts detailing how Async-UI works and what you can do with it. Stay tuned!