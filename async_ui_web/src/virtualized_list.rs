use std::{
    cell::RefCell,
    future::pending,
    hash::Hash,
    ops::{Range, RangeBounds},
};

use async_ui_web_core::{combinators::join, window, ContainerNodeFuture};
use futures_lite::Future;
use js_sys::Function;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use web_sys::{Element, HtmlElement, IntersectionObserver, IntersectionObserverInit};

use crate::{callback_to_future::CallbackToFuture, utils::MiniScopeGuard, DynamicList};

pub struct VirtualizedList<'c, Fut: Future + 'c, Updater: FnMut(Option<usize>, usize) -> Fut> {
    list: DynamicList<'c, usize, Fut>,
    state: RefCell<State<Updater>>,
    spacers: (HtmlElement, HtmlElement),
    direction: Direction,
    root: &'c Element,
    refresh: CallbackToFuture<JsValue, ()>,
}

struct State<Updater> {
    reuse_stack: Vec<usize>,
    updater: Updater,
    range: Range<usize>,
    num_items: usize,
    callback_id: Option<i32>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
    Vertical,
    Horizontal,
}

impl<'c, Fut: Future + 'c, Updater: FnMut(Option<usize>, usize) -> Fut>
    VirtualizedList<'c, Fut, Updater>
{
    pub fn new(
        root: &'c Element,
        spacer_front: HtmlElement,
        spacer_back: HtmlElement,
        updater: Updater,
    ) -> Self {
        let state = State {
            reuse_stack: Vec::new(),
            updater,
            range: 0..0,
            num_items: 0,
            callback_id: None,
        };
        Self {
            list: DynamicList::new(),
            spacers: (spacer_front, spacer_back),
            state: RefCell::new(state),
            direction: Direction::Vertical,
            root,
            refresh: CallbackToFuture::new(|_| ()),
        }
    }
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
    pub fn set_num_items(&self, num: usize) {
        self.state.borrow_mut().num_items = num;
        self.update_visible();
    }
    pub fn force_update(&self, _range: impl RangeBounds<usize>) {
        todo!()
    }
    pub async fn render(&self) {
        let (spf, spb) = &self.spacers;

        let [spf_render, spb_render] = [spf, spb]
            .map(|spacer| ContainerNodeFuture::new(pending::<()>(), spacer.clone().into()));
        let guard = MiniScopeGuard(|| {
            if let Some(callback_id) = self.state.borrow_mut().callback_id {
                window::WINDOW.with(|w| w.clear_timeout_with_handle(callback_id));
            }
        });
        join((spf_render, self.list.render(), spb_render, async {
            let _observer = Observer::new(&self.root, &[spf, spb], self.refresh.get_function());
            loop {
                self.refresh.signal.until_change().await;
                self.update_visible();
            }
        }))
        .await;
        let _ = guard;
    }
    fn update_visible(&self) {
        fn top_bottom(rect: &web_sys::DomRect) -> (f64, f64) {
            (rect.top(), rect.bottom())
        }
        let mut state = self.state.borrow_mut();

        let (spf, spb) = &self.spacers;
        let rect_f = top_bottom(&spf.get_bounding_client_rect());
        let rect_b = top_bottom(&spb.get_bounding_client_rect());
        let rect_root = top_bottom(&self.root.get_bounding_client_rect());
        let root_size = rect_root.1 - rect_root.0;
        if (rect_f.1 + root_size < rect_root.0 || state.range.start == 0)
            && (rect_b.0 - root_size > rect_root.1 || state.range.end == state.num_items)
        {
            web_sys::console::log_1(&"all good".into());
            state.callback_id = None;
            return;
        }
        web_sys::console::log_1(&"update".into());

        let num_visible = state.range.len();
        let avg_size = if num_visible > 0 {
            (rect_b.0 - rect_f.1) / num_visible as f64
        } else {
            root_size / 4.0
        };
        let new_start = ((rect_root.0 - root_size - rect_f.0) / avg_size).max(0.0) as usize;
        let new_end = (((rect_root.1 + root_size - rect_f.0) / avg_size).ceil() as usize)
            .min(state.num_items);

        let preferred_height_f = avg_size * new_start as f64;
        let preferred_height_b = avg_size * (state.num_items - new_end) as f64;

        spf.style()
            .set_property("height", &format!("{preferred_height_f}px"))
            .unwrap_throw();
        spb.style()
            .set_property("height", &format!("{preferred_height_b}px"))
            .unwrap_throw();
        web_sys::console::log_1(
            &format!("range: {new_start}..{new_end}, avg_size: {avg_size:.2}").into(),
        );
        for to_remove in (state.range.start..state.range.end.min(new_start))
            .chain(state.range.start.max(new_end)..state.range.end)
        {
            self.list.remove(&to_remove);
            state.reuse_stack.push(to_remove);
        }
        let remaining = (state.range.start >= new_start && state.range.start < new_end)
            .then_some(state.range.start);
        for to_add in new_start..new_end.min(state.range.start) {
            let reuse = state.reuse_stack.pop();
            let new_fut = (state.updater)(reuse, to_add);
            self.list.insert(to_add, new_fut, remaining.as_ref());
        }
        for to_add in new_start.max(state.range.end)..new_end {
            let reuse = state.reuse_stack.pop();
            let new_fut = (state.updater)(reuse, to_add);
            self.list.insert(to_add, new_fut, None);
        }
        state.range = new_start..new_end;
        state.callback_id = Some(window::WINDOW.with(|w| {
            w.set_timeout_with_callback_and_timeout_and_arguments_0(
                self.refresh.get_function(),
                100,
            )
            .unwrap_throw()
        }));
    }
}

struct Observer {
    observer: IntersectionObserver,
}

impl Observer {
    fn new(root: &Element, spacers: &[&HtmlElement], function: &Function) -> Self {
        let observer = IntersectionObserver::new_with_options(
            function,
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
