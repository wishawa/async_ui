use std::{future::pending, rc::Rc};

use async_ui_utils::run_on_drop::RunOnDrop;

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
    pub async fn render_borrowed(&mut self, children: Vec<Element<'_>>) {
        render_with_control(children, ElementControl::new_with_vnode(self.vnode.clone())).await
    }
    pub async fn render(mut self, children: Vec<Element<'_>>) {
        self.render_borrowed(children).await
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}
impl PortalExit {
    pub async fn render_borrowed(&mut self) {
        match &*self.vnode {
            VNode::PortalVNode(portal) => {
                ELEMENT_CONTROL.with(|control| portal.set_target(control));
            }
            _ => panic!("unexpected vnode type in portal token"),
        }
        let _guard = RunOnDrop::new(|| match &*self.vnode {
            VNode::PortalVNode(portal) => {
                portal.unset_target();
            }
            _ => panic!("unexpected vnode type in portal token"),
        });
        pending().await
    }
    pub async fn render(mut self) {
        self.render_borrowed().await
    }
    pub fn carefully_clone(&self) -> Self {
        Self {
            vnode: self.vnode.clone(),
        }
    }
}
