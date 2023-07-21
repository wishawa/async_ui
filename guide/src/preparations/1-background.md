# Backgrounds
Async UI itself is quite easy to learn!
But there are some prerequisites before you start:
you need to know Rust, Async Rust, and the Web API.
These are no small things to learn. However, all three are highly applicable.
They will be useful even if you don't end up using Async UI.

### Rust
If you don't know Rust yet, the [Rust Book](https://doc.rust-lang.org/book/)
is a good place to start!

### Async Rust
You should know
* what `async`/`await` does in Rust
* what the Future trait is
* how to run multiple Futures concurrently (i.e. "joining" Futures)

Mastery of manual `Future` implementation, `Pin`, or pin-projection is
**not** required for using Async UI.

If you don't know Async Rust yet, or want to brush up on your knowledge, the
[Async Book](https://rust-lang.github.io/async-book/) is a great resource.
Chapter 1, 3, and 6 of the book cover most of what you need for working with
Async UI.

### Web API

Async UI exposes most of the Web API. Make sure you know some HTML and
JavaScript. To start with Async UI, you only need the basics:
how to get the text value in an `<input>`, how to disable a `<button>`, etc.

The [MDN Web Docs](https://developer.mozilla.org/en-US/) is a good reference
for the Web API.
