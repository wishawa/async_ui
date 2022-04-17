use std::{future::Future, marker::PhantomPinned, pin::Pin, task::Poll};

use async_executor::Task;
use smallvec::SmallVec;

use crate::{drop_check::PropagateDropScope, element::Element, runtime::RUNTIME};

pub use super::control::node_guard::NodeGuard;
use super::{backend::Backend, control::Control, drop_check::check_drop_scope};

pin_project_lite::pin_project! {
    #[must_use = "Render is a Future and should be awaited"]
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
    pub(crate) fn set_control(&mut self, control: Control<B>) {
        self.control = Some(control);
    }
}
pub fn set_render_control<'e, B: Backend>(render: &mut Render<'e, B>, control: Control<B>) {
    render.set_control(control);
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
                .unwrap_or_else(|| B::get_tls().with(|ctr| ctr.clone()));
            this.children
                .iter_mut()
                .enumerate()
                .for_each(|(idx, child)| {
                    unsafe { child.mount(control.nest(idx)) };
                });
        }
        Poll::Pending
    }
}

macro_rules! make_tuples {
	($($id:expr),*) => {
		paste::paste! {
			impl<'f, B: Backend, $([<F $id>]: Future<Output = ()> + 'f,)*> From<($([<F $id>],)*)> for Render<'f, B> {
                fn from(tuple: ($([<F $id>],)*)) -> Self {
					let ($([<v_ $id>],)*) = tuple;
					let children = smallvec::smallvec![$(
						[<v_ $id>].into()
					),*];
                    Self {
                        control: None,
                        has_rendered: false,
                        children,
                        _pin: PhantomPinned
                    }
                }
			}
		}
	};
}
make_tuples!();
make_tuples!(1);
make_tuples!(1, 2);
make_tuples!(1, 2, 3);
make_tuples!(1, 2, 3, 4);
make_tuples!(1, 2, 3, 4, 5);
make_tuples!(1, 2, 3, 4, 5, 6);
make_tuples!(1, 2, 3, 4, 5, 6, 7);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
make_tuples!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);

pub fn put_node<B: Backend>(node: B::NodeType) -> NodeGuard<B> {
    B::get_tls().with(|vn| vn.put_node(node))
}

pub fn spawn_root<F: Future<Output = ()> + 'static>(future: F) -> Task<()> {
    let task = RUNTIME.with(|rt| rt.spawn(PropagateDropScope::new(future)));
    task
}
