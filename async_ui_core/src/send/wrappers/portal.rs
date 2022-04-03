use std::future::pending;

use super::super::{
    backend::Backend,
    control::{vnode::portal::PortalVNode, Control},
    element::Element,
    render::render_with_control,
    Shared,
};
pub struct PortalEntry<B: Backend> {
    vnode: Shared<PortalVNode<B>>,
}
pub struct PortalExit<B: Backend> {
    vnode: Shared<PortalVNode<B>>,
}

pub fn create_portal<B: Backend>() -> (PortalEntry<B>, PortalExit<B>) {
    let vnode = Shared::new(PortalVNode::new());
    (
        PortalEntry {
            vnode: vnode.clone(),
        },
        PortalExit { vnode },
    )
}
impl<B: Backend> PortalEntry<B> {
    pub fn to_element_borrowed<'e>(&mut self, children: Vec<Element<'e, B>>) -> Element<'e, B> {
        render_with_control(children, Some(Control::new_with_vnode(self.vnode.clone()))).into()
    }
    pub fn to_element<'e>(mut self, children: Vec<Element<'e, B>>) -> Element<'e, B> {
        self.to_element_borrowed(children)
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}

impl<B: Backend> PortalExit<B> {
    pub fn to_element_borrowed<'s>(&'s mut self) -> Element<'static, B> {
        let vnd = self.vnode.clone();
        let block = async move {
            B::get_tls().with(|ctr| vnd.set_target(ctr));
            let _guard = scopeguard::guard((), |_| vnd.unset_target());
            pending().await
        };
        block.into()
    }
    pub fn to_element(mut self) -> Element<'static, B> {
        self.to_element_borrowed()
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}
