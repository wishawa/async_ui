use std::{cell::RefCell, collections::BTreeMap};

use async_ui_core::local::control::{position::PositionIndices, vnode::VNode};
use glib::Cast;
use gtk::{traits::BoxExt, Widget};

use crate::manual_apis::GtkBackend;

#[derive(Debug)]
pub(crate) struct ContainerVNode {
    inner: RefCell<Inner>,
}
struct Inner {
    widget: Widget,
    children: BTreeMap<PositionIndices, Widget>,
    handler: &'static dyn ContainerHandler,
}
#[allow(unused_variables)]
pub trait ContainerHandler {
    fn get_support_multichild(&self) -> bool;
    fn set_single_child(&self, this: &Widget, child: Option<&Widget>) {
        todo!()
    }
    fn insert_child_after(&self, this: &Widget, child: &Widget, previous_sibling: Option<&Widget>) {
        todo!()
    }
    fn remove_child(&self, this: &Widget, child: &Widget) {
        todo!()
    }
    fn reorder_child_after(
        &self,
        this: &Widget,
        child: &Widget,
        new_previous_sibling: Option<&Widget>,
    ) {
        self.remove_child(this, child);
        self.insert_child_after(this, child, new_previous_sibling);
    }
}
impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("widget", &self.widget)
            .field("children", &self.children)
            .finish()
    }
}

struct BoxHandler;
impl ContainerHandler for BoxHandler {
    fn get_support_multichild(&self) -> bool {
        true
    }

    fn insert_child_after(&self, this: &Widget, child: &Widget, previous_sibling: Option<&Widget>) {
        let downcasted: &gtk::Box = this.downcast_ref().unwrap();
        downcasted.insert_child_after(child, previous_sibling);
    }

    fn remove_child(&self, this: &Widget, child: &Widget) {
        let downcasted: &gtk::Box = this.downcast_ref().unwrap();
        downcasted.remove(child);
    }

    fn reorder_child_after(
        &self,
        this: &Widget,
        child: &Widget,
        new_previous_sibling: Option<&Widget>,
    ) {
        let downcasted: &gtk::Box = this.downcast_ref().unwrap();
        downcasted.reorder_child_after(child, new_previous_sibling);
    }
}
static BOX_HANDLER: BoxHandler = BoxHandler;

impl VNode<GtkBackend> for ContainerVNode {
    fn ins_node(&self, position: PositionIndices, new_node: Widget) {
        let mut inner = self.inner.borrow_mut();
        if !inner.handler.get_support_multichild() {
            if inner.children.len() == 0 {
                inner
                    .handler
                    .set_single_child(&inner.widget, Some(&new_node));
                inner.children.insert(position, new_node);
            } else {
                inner.handler.set_single_child(&inner.widget, None);
                let wrapper = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                inner.children.iter().for_each(|(_k, v)| {
                    wrapper.append(v);
                });
                inner.widget = wrapper.upcast();
                inner.handler = &BOX_HANDLER;
            }
        } else {
            let previous_sibling = inner
                .children
                .range(..position.clone())
                .rev()
                .next()
                .map(|(_k, v)| v);
            inner
                .handler
                .insert_child_after(&inner.widget, &new_node, previous_sibling);
            if inner.children.insert(position, new_node).is_some() {
                panic!("more than one node added at the same position");
            }
        }
    }
    fn del_node(&self, position: PositionIndices) -> Widget {
        let mut inner = self.inner.borrow_mut();
        let child = inner
            .children
            .remove(&position)
            .expect("node not found for removal");
        inner.handler.remove_child(&inner.widget, &child);
        child
    }
}

impl ContainerVNode {
    pub fn new(widget: Widget, handler: &'static dyn ContainerHandler) -> Self {
        let children = BTreeMap::new();
        Self {
            inner: RefCell::new(Inner {
                widget,
                children,
                handler,
            }),
        }
    }
}
