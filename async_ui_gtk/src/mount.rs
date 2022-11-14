use std::{cell::RefCell, future::IntoFuture, rc::Rc};

use async_ui_core::{
    mount as core_mount,
    vnode::{
        node_concrete::{ConcreteNodeVNode, RefNode},
        WithVNode,
    },
};
use glib::Cast;
use gtk::{Application, ApplicationWindow};

use crate::{
    backend::Backend,
    widget::{gtk_box::GtkBoxOp, WrappedWidget},
};

pub fn mount_at<F: IntoFuture + 'static>(root: F, node: gtk::Box) {
    let fut = WithVNode::new(
        root.into_future(),
        Rc::new(
            ConcreteNodeVNode::new(
                RefNode::<Backend>::Parent {
                    parent: WrappedWidget {
                        widget: node.clone().upcast(),
                        inner_widget: node.upcast(),
                        op: crate::widget::WidgetOp::MultiChild(&GtkBoxOp),
                    },
                },
                Default::default(),
            )
            .into(),
        ),
    );
    core_mount::<Backend, _>(fut)
}

pub fn mount<F: IntoFuture + 'static>(root: F) {
    use gtk::prelude::*;
    let app = Application::builder()
        .application_id("async-ui.test.app")
        .build();
    let rb = RefCell::new(Some(root));
    app.connect_activate(move |app: &Application| {
        let b = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Async-UI App")
            .child(&b)
            .build();
        let root = rb.borrow_mut().take().expect("app actiavted twice");
        mount_at(root, b);
        window.present();
    });

    app.run();
}
