use crate::{
    widget::{gtk_box::GtkBoxOp, WidgetOp::MultiChild, WrappedWidget},
    Fragment,
};

use super::ElementFuture;
use glib::Cast;
use gtk::traits::WidgetExt;

pub struct ViewProps<'c> {
    pub children: Fragment<'c>,
    pub width: i32,
    pub height: i32,
}
impl<'c> Default for ViewProps<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            width: -1,
            height: -1,
        }
    }
}
pub async fn view<'c>(
    ViewProps {
        children,
        width,
        height,
    }: ViewProps<'c>,
) {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 0);
    b.set_size_request(width, height);
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
