use std::rc::Rc;

use async_ui_core::{
    control::Control,
    render::{
        put_node as base_put_node, set_render_control as base_set_render_control, spawn_root,
    },
    runtime::drive_runtime,
};
use glib::{Cast, IsA, MainContext, Object};
use gtk::{traits::GtkWindowExt, Widget, Window};

use crate::{
    backend::GtkBackend,
    manual_apis::NodeGuard,
    vnode::{ContainerHandler, ContainerVNode},
};

struct WindowHandler;
impl ContainerHandler for WindowHandler {
    fn get_support_multichild(&self) -> bool {
        false
    }
    fn set_single_child(&self, this: &Object, child: Option<&Widget>) {
        let downcasted: &Window = this.downcast_ref().unwrap();
        downcasted.set_child(child);
    }
}
pub fn control_from_node(
    widget: Object,
    handler: &'static dyn ContainerHandler,
) -> Control<GtkBackend> {
    Control::new_with_vnode(Rc::new(ContainerVNode::new(widget, handler)))
}
pub fn set_render_control<'e>(render: &mut Render<'e>, control: Control<GtkBackend>) {
    base_set_render_control(render, control);
}
pub type Render<'e> = async_ui_core::render::Render<'e, GtkBackend>;
pub fn mount_and_present<W: IsA<Window> + IsA<Object>>(
    root: impl Into<Render<'static>>,
    window: W,
) {
    let widget: Object = window.clone().upcast();
    let control = control_from_node(widget, &WindowHandler);
    let mut children = root.into();
    set_render_control(&mut children, control);
    let task = spawn_root(children);
    task.detach();
    window.present();
    MainContext::default().spawn_local(drive_runtime());
}

pub fn put_node(widget: Widget) -> NodeGuard {
    base_put_node(widget)
}
