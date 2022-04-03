use std::{cell::RefCell, rc::Rc};

use async_ui_core::local::{
    backend::Spawner,
    control::Control,
    drop_check::PropagateDropScope,
    render::{put_node as base_put_node, render_with_control, NodeGuard, RenderFuture},
};
use glib::Cast;
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::GtkWindowExt,
    Application, ApplicationWindow, Widget,
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
        true
    }

    fn set_single_child(&self, this: &Widget, child: Option<&Widget>) {
        let downcasted: &ApplicationWindow = this.downcast_ref().unwrap();
        downcasted.set_child(child);
    }
}
pub fn mount(root: Element<'static>, app_id: &str) {
    let app = Application::builder().application_id(app_id).build();
    let root = RefCell::new(Some(root));
    let build_ui = move |app: &Application| {
        if let Some(root) = root.borrow_mut().take() {
            let window = ApplicationWindow::new(app);
            let fut = PropagateDropScope::new(Box::pin(render_in_node(
                vec![root],
                window.clone().upcast(),
                &WINDOW_HANDLER,
            )));
            let task = GtkSpawner::spawn(fut);
            window.present();
            task.detach();
        }
    };
    app.connect_activate(build_ui);
    app.run();
}
