use std::{cell::RefCell, rc::Rc};

use async_ui_core::local::{
    backend::Spawner,
    control::Control,
    drop_check::PropagateDropScope,
    render::{put_node as base_put_node, render_with_control, NodeGuard, RenderFuture},
};
use glib::{Cast, IsA};
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::GtkWindowExt,
    Application, ApplicationWindow, Widget, Window,
};

use crate::{
    backend::GtkBackend,
    executor::GtkSpawner,
    vnode::{ContainerHandler, ContainerVNode},
    Element,
};

pub fn render_in_node<'e>(
    children: Vec<Element<'e>>,
    widget: Widget,
    handler: &'static dyn ContainerHandler,
) -> RenderFuture<'e, GtkBackend> {
    render_with_control(
        children,
        Some(Control::new_with_vnode(Rc::new(ContainerVNode::new(
            widget, handler,
        )))),
    )
}
pub fn render<'e>(children: Vec<Element<'e>>) -> RenderFuture<'e, GtkBackend> {
    render_with_control(children, None)
}
pub fn put_node(node: Widget) -> NodeGuard<GtkBackend> {
    base_put_node::<GtkBackend>(node)
}

struct WindowHandler;
static WINDOW_HANDLER: WindowHandler = WindowHandler;
impl ContainerHandler for WindowHandler {
    fn get_support_multichild(&self) -> bool {
        false
    }
    fn set_single_child(&self, this: &Widget, child: Option<&Widget>) {
        let downcasted: &Window = this.downcast_ref().unwrap();
        downcasted.set_child(child);
    }
}
pub fn mount_and_present<W: IsA<Window> + IsA<Widget>>(root: Element<'static>, window: W) {
    let widget: Widget = window.clone().upcast();
    let fut = PropagateDropScope::new(Box::pin(render_in_node(
        vec![root],
        widget,
        &WINDOW_HANDLER,
    )));
    let task = GtkSpawner::spawn(fut);
    window.present();
    task.detach();
}
