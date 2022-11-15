use crate::{
    widget::{gtk_box::GtkBoxOp, WidgetOp::MultiChild, WrappedWidget},
    Fragment,
};

use super::ElementFuture;
use glib::Cast;
use gtk::traits::WidgetExt;

#[derive(Default)]
pub struct ViewProps<'c> {
    pub children: Fragment<'c>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
pub async fn view<'c>(
    ViewProps {
        children,
        width,
        height,
    }: ViewProps<'c>,
) {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 0);
    b.set_size_request(width.unwrap_or(-1), height.unwrap_or(-1));
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
