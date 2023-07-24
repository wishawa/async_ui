use std::{
    cell::RefCell,
    future::pending,
    hash::Hash,
    ops::{Range, RangeBounds},
    rc::Rc,
};

use async_ui_internal_utils::reactive_cell::ReactiveCell;
use async_ui_web_core::{combinators::join, ContainerNodeFuture};
use futures_lite::{Future, StreamExt};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{Element, HtmlElement, IntersectionObserver, IntersectionObserverInit};

use super::DynamicList;

/**
For displaying large lists.

Instead of rendering all items at once, this list renders only the part
that is visible on the screen. This allows very large lists to be displayed.

```
# use async_ui_web::{html::Div, prelude_traits::*, lists::VirtualizedList};
# async fn app() {
let root = Div::new();
let list = VirtualizedList::new(
    &root.element,
    Div::new().element.into(),
    Div::new().element.into(),
    |index| Div::new().render(index.to_string().render())
);
list.set_num_items(100000);
root.render(list.render()).await;
# }
```

Implementation is still quite incomplete.
* Only fixed-height items are supported currently.
* Lots of configuration options are missing.
* LTR/RTL?
* Let me know what else is missing. Create a GitHub issue.

*/
pub struct VirtualizedList<'c, Fut: Future + 'c, Renderer: FnMut(usize) -> Fut> {
    list: DynamicList<'c, usize, Fut>,
    state: RefCell<State<Renderer>>,
    spacers: (HtmlElement, HtmlElement),
    direction: Direction,
    root: &'c HtmlElement,
    signal: Rc<ReactiveCell<()>>,
    wake_closure: Closure<dyn Fn(JsValue)>,
}

struct State<Updater> {
    renderer: Updater,
    range: Range<usize>,
    num_items: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
    Vertical,
    Horizontal,
}

impl<'c, Fut: Future + 'c, Renderer: FnMut(usize) -> Fut> VirtualizedList<'c, Fut, Renderer> {
    /**
    Create a new list.

    Arguments:
    *   `root`: the element in which this list will be scrolling
        (the element where the scrollbar appears; for page scrolling, this is the `<html>`).
        [HtmlElement]s are is usually obtained from accessing the `element` field
        in elements created by Async UI. For example, [Div][crate::html::Div]
        exposes its [HtmlElement] in the `element` field of the `Div` struct.

        You are the one responsible for rendering the root, and rendering
        this VirtualizedList inside the root.

    *   `spacer_front`: an element to render as the top or left scroll spacer.
        The scroll spacer is what is displayed in place of items that are
        off the viewport (and thus not rendered). Usually, you will simply use
        `<div>`s. However, if you're rendering a table, you might want to use
        `<tr>`.

        You should *not* render the spacers in addition to the list.
        Give the spacers to the list, the list will render them for you.

    *   `spacer_back`: like `spacer_front`, but for bottom or right.

    *   `renderer`: this is where you actually render things. This argument
        must be a function/closure that takes a `usize` index, and return
        the future that should appear at that position in the list.
    */
    pub fn new(
        root: &'c HtmlElement,
        spacer_front: HtmlElement,
        spacer_back: HtmlElement,
        renderer: Renderer,
    ) -> Self {
        let state = State {
            renderer,
            range: 0..0,
            num_items: 0,
        };
        let signal = Rc::new(ReactiveCell::new(()));
        let signal_cloned = signal.clone();
        Self {
            list: DynamicList::new(),
            spacers: (spacer_front, spacer_back),
            state: RefCell::new(state),
            direction: Direction::Vertical,
            root,
            signal,
            wake_closure: Closure::new(move |_: JsValue| {
                signal_cloned.borrow_mut();
                async_ui_web_core::executor::run_now();
            }),
        }
    }
    /// Should the list be for vertical or horizontal scrolling?
    ///
    /// Default is vertical.
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
    /// How many items does the list have?
    pub fn set_num_items(&self, num: usize) {
        self.state.borrow_mut().num_items = num;
        self.update_visible();
    }
    #[doc(hidden)]
    pub fn force_update(&self, _range: impl RangeBounds<usize>) {
        todo!()
    }
    pub async fn render(&self) {
        let (spf, spb) = &self.spacers;

        let [spf_render, spb_render] = [spf, spb]
            .map(|spacer| ContainerNodeFuture::new(pending::<()>(), spacer.clone().into()));
        let _guard = scopeguard::guard((), |_| {
            self.root
                .remove_event_listener_with_callback(
                    "scroll",
                    self.wake_closure.as_ref().unchecked_ref(),
                )
                .unwrap_throw();
        });
        join((spf_render, self.list.render(), spb_render, async {
            let _observer = Observer::new(
                self.root,
                &[spf, spb],
                self.wake_closure.as_ref().unchecked_ref(),
            );
            self.update_visible();
            self.root
                .add_event_listener_with_callback(
                    "scroll",
                    self.wake_closure.as_ref().unchecked_ref(),
                )
                .unwrap_throw();
            let mut uc = self.signal.until_change();
            loop {
                uc.next().await;
                self.update_visible();
            }
        }))
        .await;
    }
    fn update_visible(&self) {
        fn top_bottom(rect: &web_sys::DomRect, direction: Direction) -> (f64, f64) {
            match direction {
                Direction::Vertical => (rect.top(), rect.bottom()),
                Direction::Horizontal => (rect.left(), rect.right()),
            }
        }
        let mut state = self.state.borrow_mut();

        let (spf, spb) = &self.spacers;
        let rect_f = top_bottom(&spf.get_bounding_client_rect(), self.direction);
        let rect_b = top_bottom(&spb.get_bounding_client_rect(), self.direction);
        let rect_root = top_bottom(&self.root.get_bounding_client_rect(), self.direction);
        let root_size = rect_root.1 - rect_root.0;
        let tres_size = root_size * 1.25;
        let pad_size = root_size * 2.0;
        if (rect_f.1 + tres_size < rect_root.0 || state.range.start == 0)
            && (rect_b.0 - tres_size > rect_root.1 || state.range.end == state.num_items)
        {
            return;
        }

        let num_visible = state.range.len();
        let avg_size = if num_visible > 0 {
            (rect_b.0 - rect_f.1) / num_visible as f64
        } else {
            root_size / 4.0
        };
        let new_start = ((rect_root.0 - pad_size - rect_f.0) / avg_size).max(0.0) as usize;
        let new_end =
            (((rect_root.1 + pad_size - rect_f.0) / avg_size).ceil() as usize).min(state.num_items);

        let preferred_height_f = avg_size * new_start as f64;
        let preferred_height_b = avg_size * (state.num_items - new_end) as f64;

        {
            spf.style()
                .set_property("block-size", &format!("{preferred_height_f}px"))
                .ok();
            spb.style()
                .set_property("block-size", &format!("{preferred_height_b}px"))
                .ok();
        }
        for to_remove in (state.range.start..state.range.end.min(new_start))
            .chain(state.range.start.max(new_end)..state.range.end)
        {
            self.list.remove(&to_remove);
        }
        let remaining = (state.range.start >= new_start && state.range.start < new_end)
            .then_some(state.range.start);
        for to_add in new_start..new_end.min(state.range.start) {
            self.list
                .insert(to_add, (state.renderer)(to_add), remaining.as_ref());
        }
        for to_add in new_start.max(state.range.end)..new_end {
            self.list.insert(to_add, (state.renderer)(to_add), None);
        }
        state.range = new_start..new_end;
    }
}

struct Observer {
    observer: IntersectionObserver,
}

impl Observer {
    fn new(root: &Element, spacers: &[&HtmlElement], wake: &js_sys::Function) -> Self {
        let observer = IntersectionObserver::new_with_options(
            wake,
            IntersectionObserverInit::new()
                .root(Some(root))
                .root_margin("100%"),
        )
        .unwrap_throw();
        spacers.iter().for_each(|sp| observer.observe(sp));

        Self { observer }
    }
}

impl Drop for Observer {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}
