use crate::{
    widget::{gtk_box::GtkBoxOp, WidgetOp::MultiChild, WrappedWidget},
    Fragment,
};

use super::ElementFuture;
use glib::Cast;

#[derive(Default)]
pub struct ViewProps<'c> {
    pub children: Fragment<'c>,
}
pub async fn view<'c>(ViewProps { children }: ViewProps<'c>) {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 0);
    ElementFuture::new(
        children,
        WrappedWidget {
            widget: b.clone().upcast(),
            inner_widget: b.upcast(),
            op: MultiChild(&GtkBoxOp),
        },
    )
    .await;
}
