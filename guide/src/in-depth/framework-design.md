# Framework Design

## Design

### Basics
*	UI as side effect: Rust [Future](https://doc.rust-lang.org/nightly/core/future/trait.Future.html)
	objects can render things.
	*	These Futures are long-running. Their UI elements stay on the screen
		until they are dropped.

### DOM
*	HTML structure is built by joining and nesting Futures.
	*	The "base" Future is [ContainerNodeFuture](https://docs.rs/async_ui_web_core/latest/async_ui_web_core/struct.ContainerNodeFuture.html).
		It puts a single HTML node on the screen.
		It use the described context to insert its node on first poll,
		and remove the node when dropped.
	*	The code for joining is based on [futures-concurrency](https://docs.rs/futures-concurrency/),
		but rewritten to significantly improve performance and reduce allocations.
*	We use an implicit rendering context, provided as a scoped thread-local set
	for each Future `poll`. The context provides the information needed for
	[`insertBefore`](https://developer.mozilla.org/en-US/docs/Web/API/Node/insertBefore)
	*	The HTML element that the polled Future should insert its own element to.
		This is the "parent element".
	*	An ordered map containing all the children of the parent element,
		each keyed by a lexicographical "path" from the parent element.
		```rust
		Div::new().render( // this Div is the parent element for A-F
		join((
			A, // path = [0]
			join((
				C, // path = [1, 0]
				D, // path = [1, 1]
				join((
					E, // path = [1, 2, 0]
					F, // path = [1, 2, 1]
				))
			)),
			B, // path = [2]
			Div::new().render( // this Div is the parent of G and H
			join((
				G, // path = [0]
				H, // path = [1]
			))
			),
		))
		)
		```
		```html
		<div>
			A
			C
			D
			E
			F
			B
			<div>
				G
				H
			</div>
		</div>
		```
		*	The "path" is stored as a `SmallVec<[u32; 4]>`.
*	No diffing virtual DOM. Dynamicity is special case
	([DynamicSlot](https://docs.rs/async_ui_web/latest/async_ui_web/components/struct.DynamicSlot.html)).
*	The event listener [EventFutureStream](https://docs.rs/async_ui_web/latest/async_ui_web/event_handling/struct.EventFutureStream.html)
	type add a callback to put the event object in a cell and wake its waker.
	On every poll it the check if there is an event in the cell.

### State
*	We don't provide reactivity or state management.
*	State can be put in Future objects.
	*	Usually, this comes in the form of variables in async functions.

### Scheduling
*	The whole app is one big Future.
*	A "main" async executor is provided ([async-executor](https://docs.rs/async-executor/)).
	It drives the app Future and can also be used to spawn things.
*	The main executor is driven by a hand-rolled "root" executor.
	*	The root executor `setTimeout` itself to execute when woken.

## Comparisons
*	The UI-as-side-effect design of Async UI is quite unique, as far as I know.
	*	Blocking UI (like [`alert`](https://developer.mozilla.org/en-US/docs/Web/API/Window/alert)
		or [`MessageBox`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messagebox))
		technically present UI as side-effect too, but they're not very flexible
		since they block the entire application.
*	Async UI has long-living async functions and store component state as
	variables in functions. This approach is similar to [Crank.js](https://crank.js.org/).

## Motivations
Async UI is motivated by the fact that async is an effect system, and in many cases,
UI is too.

But that isn't the only possible motive for this framework design.
[This blog post by *notgull*](https://notgull.github.io/async-gui/) describes
the same idea of using async for UI, but from a less abstracted perspective.