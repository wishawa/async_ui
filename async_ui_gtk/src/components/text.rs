use glib::Cast;
use observables::{ObservableAs, ObservableAsExt};

use crate::widget::{WidgetOp, WrappedWidget};

use super::ElementFuture;

pub async fn text<'c>(text: &'c dyn ObservableAs<str>) {
    let node = gtk::Label::new(None);
    let widget: gtk::Widget = node.clone().upcast();
    ElementFuture::new(
        text.for_each(|t| node.set_label(t)),
        WrappedWidget {
            widget: widget.clone(),
            inner_widget: widget.upcast(),
            op: WidgetOp::NoChild,
        },
    )
    .await;
}
