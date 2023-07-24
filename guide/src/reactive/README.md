# Reactivity and State Management

Async UI **does not** provide reactivity or state management solution.
The async API should be flexible enough for you to "bring your own reactivity".

The `async_ui_web` crate does expose a [ReactiveCell](https://docs.rs/async_ui_web/latest/async_ui_web/struct.ReactiveCell.html)
type that provides basic reactivity. However, it is unlikely to be powerful
enough to manage the state of complex applications.

There are, as far as I am aware, two Rust state management libraries out there
that provide async API.

### Futures-Signals

The [futures-signals crate](https://crates.io/crates/futures-signals)
provide reactivity based on "signals". If you're interested, [the crate's
tutorial](https://docs.rs/futures-signals/latest/futures_signals/tutorial/index.html)
explains everything.

### X-Bow

Along with Async UI, I have also been working on a state management library.
It's name is `X-Bow` (because early prototypes were inspired by the JavaScript
[MobX](https://mobx.js.org/) library). You can access it's documentation
[here](https://docs.rs/x-bow).
