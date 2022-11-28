/*! Built-in components provided by Async UI.
 *
 * ## Props
 * The built-in components each take a *props* struct as argument.
 * The struct contains many fields, each representing a possible customization of the component.
 * You should use [Default::default] and
 * the [struct update syntax](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-from-other-instances-with-struct-update-syntax)
 * to create prop structs.
 *
 * ```rust
 * button(ButtonProps {
 *     on_press: &mut |_| { todo!() },
 *     ..Default::default() // this will fill the remaining fields with default values
 * })
 * ```
 *
 * ## Children
 * Many of the components accept a *children* prop of type [Fragment][super::Fragment].
 * They will render the given fragment inside their node.
 * ```rust
 * button(ButtonProps {
 *     // have the text "Say Hello" inside the button
 *     children: fragment((
 *         text(&["Say Hello"]),
 *     )),
 *     ..Default::default()
 * })
 * ```
 *
 * ## Reactivity
 * These built-in components support reactivity based on [observables].
 *
 * Reactive props are taken in form of `prop: &'c dyn ObservableAs<T>`.
 * For example, [text][text::text] takes `&'c dyn ObservableAs<str>` as its prop.
 *
 * To provide an unchanging value as a reactive prop, wrap it in square brackets.
 * ```rust
 * text(&["Hello World!"]).await; // the text is always "Hello World!" - never changes
 * ```
 *
 * You can use [ReactiveCell][observables::cell::ReactiveCell] for basic reactivity.
 * ```rust
 * async fn counter() {
 *     let mut count = 0;
 *     
 *     // Like a RefCell that you can subscribe to!
 *     let count_string = ReactiveCell::new(count.to_string());
 *
 *     fragment((
 *         // When count_string changes, the text will change.
 *         text(&count_string.as_observable()),
 *     
 *         button(ButtonProps {
 *             children: fragment((
 *                 text(&"+"),
 *             )),
 *             on_press: Some(&mut |_ev| {
 *                 // Upon press, increment count and update the string accordingly.
 *                 count += 1;
 *                 *count_string.borrow_mut() = count.to_string();
 *             }),
 *             ..Default::default()
 *         })
 *     )).await;
 * }
 * ```
 * There is a more complete state management library in the works.
 */
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use async_ui_core::{
    backend::BackendTrait,
    vnode::{
        node_concrete::{ConcreteNodeVNode, RefNode},
        VNode, VNodeTrait,
    },
};
use pin_project_lite::pin_project;
use web_sys::Node;

mod dummy;
mod events;

mod button;
mod checkbox;
mod link;
mod list;
mod radio;
mod slider;
mod text;
mod text_input;
mod view;
pub use button::{button, ButtonProps};
pub use checkbox::{checkbox, CheckboxProps};
pub use link::{link, LinkProps};
pub use list::{list, ListModel, ListProps};
pub use radio::{radio_button, radio_group, RadioGroupProps, RadioProps};
pub use slider::{slider, SliderProps};
pub use text::text;
pub use text_input::{text_input, TextInputProps};
pub use view::{view, ViewProps};

use crate::backend::Backend;

pin_project! {
    /** For creating your own component through web_sys.
     *
     * This future type can render a web_sys node, and render children inside it.
     * ```rust
     * let future = async {
     *     // anything element awaited in here will be rendered as a child of `node`
     * }
     * // `node` must be a web_sys Node.
     * let n = ElementFuture::new(future, node);
     * n.await;
     * ```
     */
    pub struct ElementFuture<F: Future> {
        #[pin]
        future: F,
        inner: ElementFutureInner
    }
}
struct ElementFutureInner {
    node: Node,
    vnodes: Option<MyAndParentVNodes>,
}
struct MyAndParentVNodes {
    my: Rc<VNode<Backend>>,
    parent: Rc<VNode<Backend>>,
}

impl Drop for ElementFutureInner {
    fn drop(&mut self) {
        if let Some(MyAndParentVNodes { parent, .. }) = &self.vnodes {
            parent.del_child_node(Default::default());
        }
    }
}
impl<F: Future> ElementFuture<F> {
    pub fn new(future: F, node: Node) -> Self {
        Self {
            future,
            inner: ElementFutureInner { node, vnodes: None },
        }
    }
}
impl<F: Future> Future for ElementFuture<F> {
    type Output = F::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let vnk = Backend::get_vnode_key();
        let vnodes = this.inner.vnodes.get_or_insert_with(|| {
            let parent_vnode = vnk.with(Clone::clone);
            parent_vnode.add_child_node(this.inner.node.to_owned(), Default::default());
            let parent_context = parent_vnode.get_context_map().clone();
            let my = Rc::new(
                ConcreteNodeVNode::new(
                    RefNode::Parent {
                        parent: this.inner.node.clone(),
                    },
                    parent_context,
                )
                .into(),
            );
            MyAndParentVNodes {
                my,
                parent: parent_vnode,
            }
        });
        vnk.set(&vnodes.my, || this.future.poll(cx))
    }
}
