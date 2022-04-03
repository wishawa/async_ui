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
    traits::{BoxExt, GtkWindowExt, WidgetExt},
    Application, ApplicationWindow, Label, Widget,
};

use crate::{
    backend::GtkBackend,
    executor::GtkSpawner,
    vnode::{NodeVNode, VNode, VNodeEnum},
    Element,
};

pub fn render_in_node<'e>(
    children: Vec<Element<'e>>,
    node: Widget,
) -> RenderFuture<'e, GtkBackend> {
    render_with_control(
        children,
        Some(Control::new_with_vnode(VNode(Rc::new(VNodeEnum::from(
            NodeVNode::new(node),
        ))))),
    )
}
pub fn render<'e>(children: Vec<Element<'e>>) -> RenderFuture<'e, GtkBackend> {
    render_with_control(children, None)
}
pub fn put_node(node: Widget) -> NodeGuard<GtkBackend> {
    base_put_node::<GtkBackend>(node)
}

pub fn mount(root: Element<'static>, app_id: &str) {
    let app = Application::builder().application_id(app_id).build();
    let root = RefCell::new(Some(root));
    let build_ui = move |app: &Application| {
        if let Some(root) = root.borrow_mut().take() {
            let window = ApplicationWindow::new(app);
            let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
            // let label = Label::new(Some("test"));
            // container.append(&label);
            // label.insert_before(&container, Option::<&Widget>::None);
            window.set_child(Some(&container));
            let container_widget: Widget = container.upcast();
            let fut =
                PropagateDropScope::new(Box::pin(render_in_node(vec![root], container_widget)));
            let task = GtkSpawner::spawn(fut);
            window.present();
            task.detach();
        }
    };
    app.connect_activate(build_ui);
    app.run();
}
