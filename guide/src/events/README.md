# Handling Events

Unlike other UI frameworks,
Async UI does **not** let you set event handler callbacks.

Instead, we provide an async API.
Waiting for a user click is no different from waiting for the `TimeoutFuture`
we used in previous chapters.

```rust
{{ #include ../../../examples/guide-project/src/events/mod.rs:quick-example }}
```

The methods for waiting for events are all named `until_*`.
They are provided in 3 different traits

*	[EmitEvent](https://docs.rs/async_ui_web/latest/async_ui_web/event_traits/trait.EmitEvent.html)
	is implemented on anything that is a [JavaScript Event Target](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget).
	It provides only one method:
	```rust
	fn until_event<E>(&self, name: Cow<'static, str>) -> EventFutureStream<E>;
	```
	It listens to event of the given name.
	
	We will discuss what `EventFutureStream` does shortly.
*	[EmitElementEvent](https://docs.rs/async_ui_web/latest/async_ui_web/event_traits/trait.EmitElementEvent.html)
	is implemented for [the Element JavaScript class](https://developer.mozilla.org/en-US/docs/Web/API/Element).
	It provides methods like:
	```rust
	fn until_click(&self) -> EventFutureStream<MouseEvent>;
	fn until_focus(&self) -> EventFutureStream<UiEvent>;
	fn until_keydown(&self) -> EventFutureStream<KeyboardEvent>;
	// ... and more ...
	```
*	[EmitHtmlElementEvent](https://docs.rs/async_ui_web/latest/async_ui_web/event_traits/trait.EmitHtmlElementEvent.html)
	is implemented for [the HTMLElement JavaScript class](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement)
	(note that this is not the same thing as the `Element` class).
	It provides methods like:
	```rust
	fn until_input(&self) -> EventFutureStream<Event>;
	fn until_drag(&self) -> EventFutureStream<DragEvent>;
	// ... and more ...
	```

Usually, you would just type in the method you want to use, and [rust-analyzer](https://rust-analyzer.github.io/)
will figure out which trait to import for you.

## `EventFutureStram<E>`
The return type of all those methods is
[EventFutureStream](https://docs.rs/async_ui_web/latest/async_ui_web/event_handling/struct.EventFutureStream.html).

### Use as Future

As you have already seen in the previous example,
you can `await` an EventFutureStream. It is a Future.

When `await`-ed, an EventFutureStream will return the JavaScript Event object that
it listens for (the object is translated to Rust via web-sys;
see for instance [web_sys::MouseEvent](https://docs.rs/web-sys/latest/web_sys/struct.MouseEvent.html)).

You can, for example, call [`preventDefault`](https://developer.mozilla.org/en-US/docs/Web/API/Event/preventDefault)
on the returned event object.
```rust
{{ #include ../../../examples/guide-project/src/events/mod.rs:return-type }}
```

In this example, notice that we only listen to `click` event *once*.
The first time the user click the link, `preventDefault` will be called and the
link won't be opened. The second time, however, the link will open normally.

If you want to handle the event every time it fires, you can put the code in
a loop. Try it! now the link won't open however many times you click.

### Use as Stream

EventFutureStream is not only a Future, it is also a
[Stream](https://docs.rs/futures-core/latest/futures_core/stream/trait.Stream.html).
It can be quite convenient to work with it through the Stream API instead of
the Future API.

For example, let's use the Stream API from [futures-lite](https://docs.rs/futures-lite).

The crate provides a [for_each method](https://docs.rs/futures-lite/latest/futures_lite/stream/trait.StreamExt.html#method.for_each)
for Streams. It is perfect for our use case.

```rust
{{ #include ../../../examples/guide-project/src/events/mod.rs:prevent-default-stream }}
```