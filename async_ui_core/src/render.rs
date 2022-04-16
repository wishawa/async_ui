use std::{future::Future, marker::PhantomPinned, pin::Pin, task::Poll};

use smallvec::SmallVec;

use crate::{element::Element, tuple::TupleOfFutures};

pub use super::control::node_guard::NodeGuard;
use super::{backend::Backend, control::Control, drop_check::check_drop_scope};

pin_project_lite::pin_project! {
    pub struct Render<'e, B>
    where B: Backend
    {
        control: Option<Control<B>>,
        has_rendered: bool,
        children: SmallVec<[Element<'e, B>; 4]>,
        _pin: PhantomPinned
    }
}

impl<'e, B: Backend> Render<'e, B> {
    pub fn set_control(&mut self, control: Control<B>) {
        self.control = Some(control);
    }
}

impl<'e, B: Backend> Future for Render<'e, B> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let ptr = &*self as *const _ as *const ();
        let this = self.project();
        if !*this.has_rendered {
            *this.has_rendered = true;
            check_drop_scope(ptr);
            let control = this
                .control
                .take()
                .unwrap_or_else(|| B::get_dummy_control());
            this.children.iter_mut().for_each(|child| {
                let cc = control.clone();
                unsafe { child.mount(cc) };
            });
        }
        Poll::Pending
    }
}

pub fn render_with_control<'e, B: Backend, C: TupleOfFutures<'e>>(
    children: C,
    control: Option<Control<B>>,
) -> Render<'e, B> {
    let children = children.internal_convert_to_smallvec_element();
    Render {
        control,
        has_rendered: false,
        children,
        _pin: PhantomPinned,
    }
}

pub fn put_node<B: Backend>(node: B::NodeType) -> NodeGuard<B> {
    B::get_tls().with(|vn| vn.put_node(node))
}
