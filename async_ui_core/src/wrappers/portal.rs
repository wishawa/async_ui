use std::{future::pending, rc::Rc};

use crate::{
    render::{render_with_control, Render},
    tuple::TupleOfFutures,
};

use super::super::{
    backend::Backend,
    control::{vnode::portal::PortalVNode, Control},
};
pub struct PortalEntry<B: Backend> {
    vnode: Rc<PortalVNode<B>>,
}
pub struct PortalExit<B: Backend> {
    vnode: Rc<PortalVNode<B>>,
}

pub fn create_portal<B: Backend>() -> (PortalEntry<B>, PortalExit<B>) {
    let vnode = Rc::new(PortalVNode::new());
    (
        PortalEntry {
            vnode: vnode.clone(),
        },
        PortalExit { vnode },
    )
}
impl<B: Backend> PortalEntry<B> {
    pub fn render_borrowed<'e, C: TupleOfFutures<'e>>(&mut self, children: C) -> Render<'e, B> {
        render_with_control(children, Some(Control::new_with_vnode(self.vnode.clone())))
    }
    pub fn render<'e, C: TupleOfFutures<'e>>(mut self, children: C) -> Render<'e, B> {
        self.render_borrowed(children)
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}

impl<B: Backend> PortalExit<B> {
    pub fn render_borrowed<'s>(&'s mut self) -> Render<'static, B> {
        let vnd = self.vnode.clone();
        let block = async move {
            B::get_tls().with(|ctr| vnd.set_target(ctr));
            let _guard = scopeguard::guard((), |_| vnd.unset_target());
            pending().await
        };
        render_with_control((block,), None)
    }
    pub fn render(mut self) -> Render<'static, B> {
        self.render_borrowed()
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}
