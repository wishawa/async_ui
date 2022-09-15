use glib::Cast;
use gtk::traits::BoxExt;

use super::{MultiChildWidgetOp, WrappedWidget};

pub struct GtkBoxOp;
impl MultiChildWidgetOp for GtkBoxOp {
    fn remove_child(&self, this: &gtk::Widget, child: &mut WrappedWidget) {
        let casted = this.downcast_ref::<gtk::Box>().unwrap();
        casted.remove(&child.widget);
    }
}
