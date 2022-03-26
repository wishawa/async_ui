use std::{future::pending, rc::Rc};

use crate::{
    control::{
        element_control::{ElementControl, ELEMENT_CONTROL},
        vnode::{PortalVNode, VNode},
    },
    element::Element,
    render::render_with_control,
};

pub struct PortalEntry {
    vnode: Rc<VNode>,
}
pub struct PortalExit {
    vnode: Rc<VNode>,
}

pub fn create_portal() -> (PortalEntry, PortalExit) {
    let vnode = Rc::new(VNode::from(PortalVNode::new()));
    (
        PortalEntry {
            vnode: vnode.clone(),
        },
        PortalExit { vnode },
    )
}
impl PortalEntry {
    pub fn to_element_borrowed<'e>(&mut self, children: Vec<Element<'e>>) -> Element<'e> {
        render_with_control(
            children,
            Some(ElementControl::new_with_vnode(self.vnode.clone())),
        )
        .into()
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
            let _guard: scopeguard::ScopeGuard<_, _> = match &*vnd {
                VNode::PortalVNode(portal) => {
                    ELEMENT_CONTROL.with(|control| portal.set_target(control));
                    scopeguard::guard((), |_| portal.unset_target())
                }
                _ => panic!("unexpected vnode type in portal token"),
            };
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
