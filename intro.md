This blog post is intended for audience with experience in async Rust. It assumes you know what a [Future](TODO ADD LINK) is. If you don't know about async Rust but just want to write UIs, read the [README](TODO ADD LINK).

## Why Async for UIs?
A UI component gets mounted, stays there, and gets repeatedly updated throughout its lifetime. A Future gets polled, stays there, and gets polled again on updates until it completes.

A UI component composes children components together.
A Future awaits children Futures inside it.

A UI component gets unmounted, and unmounts all its children in the process.
A Future gets dropped, and drops all its children Futures in the process.


Future, in all, is a good abstraction to build UIs on. And so I made `async_ui` — a UI framework where everything is a `Future`.

## Futures as Components

```rust
async fn hello_world() {
	(Text {
		text: &"Hello World!",
		..Default::default()
	}).await
}
```

Here `Text` is a built-in component provided by the library. It is a struct that implements `Future`. To render it you simply await it. The async function we just made is also a component. To render it, just `hello_world().await`.

## Easily Compose Components
Now what if you want to render two or more things at once? Our `fragment!` macro let you put in many Futures and get a single Future in return. You can pass the returned `Fragment` around and await it where you want to render the things you put in.

```rust
async fn hello_world_2() {
	fragment![
		hello_world(),
		Button {
			children: fragment![
				Text {
					text: &"Say hello back",
					..Default::default()
				}
			],
			on_press: &mut |_ev: PressEvent| {
				todo!()
			},
			..Default::default()
		}
	].await
}
```

Here, we are rendering the "Hello World!" and a button next to it. Inside the button we have a text "Say hello back".

Pretty simple right? Just create components with async functions (or structs, or async blocks), combine them with `fragment!`, and await them where you want them to render.

## Bring your own Reactivity — or use ours
The core of async-ui doesn't know anything about reactivity. It's just async Rust! You can use channels ([async-channel](https://crates.io/crates/async-channel) is a great crate). You can use the excellent [futures-signals](https://crates.io/crates/futures-signals) crate. You can use whatever works in async Rust.

The built-in components support reactivity through a simple interface provided by the `observables` crate. The crate provides bare-bone reactivity for basic uses.

```rust
async fn counter(initial_count: i32) {
	let mut count = initial_count;
	let count_string = ObservableCell::new(count.to_string());
	fragment![
		Text {
			// When count_string changes, the text will change
			text: count_string.as_observable(),
			..Default::default()
		},
		Button {
			children: fragment![
				Text: {
					text: &"+",
					..Default::default()
				}
			],
			on_press: &mut |_press_event| {
				count += 1;
				*count_string.borrow_mut() = count.to_string();
			},
			..Default::default()
		}
	].await
}
```

Mostly, though, the observables crate is just an interface for more complete reactivity libraries. It interfaces with `async-channel`, `futures-signals`, and also the state management library I created together with async-ui: X-Bow. Read more about X-Bow in my other blog post [here](TODO ADD LINK). See observables' [docs](TODO ADD LINK) for more information on how to use it.