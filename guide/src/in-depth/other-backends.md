# Future Work: Other Backends

Async UI is unique not in its implementation,
but rather in its developer-facing API design.

This design is not at all restricted to working with the web platform.
It can be adapted to run on top of essentially any node-based GUI library.

The first implementation of Async UI, in fact, came with two backends:
HTML and GTK4. I decided to drop GTK and focus effort on the web backend.
Once the implementation for web stabilizes more, GTK can be revisited.

## Purely Async GUI
Theoretically, it is also possible to build a library like Async UI without
relying on an underlying node-based GUI library at all. Once the UI ecosystem
matrues a bit more (good layout engines, good cross-platform APIs, etc.)
maybe we can take a shot at it!

## LiveView
A "LiveView" implementation
(where the Rust code runs on the server and sync DOM updates to the web browser)
is possible.

Async UI would fit this model pretty well, since most HTTP server are already
async.
