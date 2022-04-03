use std::{future::pending, rc::Rc};

use async_ui_core::local::{backend::Backend, control::Control, render::render_with_control};

use crate::{
    backend::WebBackend,
    vnode::{PortalVNode, VNode, VNodeEnum},
    Element,
};

pub struct PortalEntry {
    vnode: Rc<VNodeEnum>,
}
pub struct PortalExit {
    vnode: Rc<VNodeEnum>,
}

pub fn create_portal() -> (PortalEntry, PortalExit) {
    let vnode = Rc::new(VNodeEnum::from(PortalVNode::new()));
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
            Some(Control::new_with_vnode(VNode(self.vnode.clone()))),
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
                VNodeEnum::PortalVNode(portal) => {
                    WebBackend::get_tls().with(|ctr| portal.set_target(ctr));
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
