use std::{future::pending, rc::Rc};

use async_ui_core::local::{
    control::{vnode::portal::PortalVNode, Control},
    render::render_with_control,
};

use crate::{manual_apis::GtkBackend, Element};

pub struct PortalEntry {
    vnode: Rc<PortalVNode<GtkBackend>>,
}
pub struct PortalExit {
    vnode: Rc<PortalVNode<GtkBackend>>,
}

pub fn create_portal() -> (PortalEntry, PortalExit) {
    let vnode = Rc::new(PortalVNode::new());
    (
        PortalEntry {
            vnode: vnode.clone(),
        },
        PortalExit { vnode },
    )
}
impl PortalEntry {
    pub fn to_element_borrowed<'e>(&mut self, children: Vec<Element<'e>>) -> Element<'e> {
        render_with_control(children, Some(Control::new_with_vnode(self.vnode.clone()))).into()
    }
    pub fn to_element<'e>(mut self, children: Vec<Element<'e>>) -> Element<'e> {
        self.to_element_borrowed(children)
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}

impl PortalExit {
    pub fn to_element_borrowed<'s>(&'s mut self) -> Element<'static> {
        let vnd = self.vnode.clone();
        let block = async move {
            let _guard = scopeguard::guard((), |_| vnd.unset_target());
            pending().await
        };
        block.into()
    }
    pub fn to_element(mut self) -> Element<'static> {
        self.to_element_borrowed()
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}
