use glib::Cast;
use gtk::traits::FlowBoxChildExt;

use super::{MultiChildWidgetOp, WrappedWidget};

pub struct FlowBoxOp;
impl MultiChildWidgetOp for FlowBoxOp {
    fn add_child(
        &self,
        this: &gtk::Widget,
        child: &mut WrappedWidget,
        before: Option<&WrappedWidget>,
    ) {
        if child.widget.downcast_ref::<gtk::FlowBoxChild>().is_none() {
            let fb = gtk::FlowBoxChild::new();
            fb.set_child(Some(&child.widget));
            child.inner_widget = Some(std::mem::replace(&mut child.widget, fb.upcast()));
        }
        let index = if let Some(before) = before {
            let casted = before.widget.downcast_ref::<gtk::FlowBoxChild>().unwrap();
            casted.index()
        } else {
            -1
        };
        let casted = this.downcast_ref::<gtk::FlowBox>().unwrap();
        casted.insert(&child.widget, index);
    }
    fn remove_child(&self, this: &gtk::Widget, child: &mut WrappedWidget) {
        let casted = this.downcast_ref::<gtk::FlowBox>().unwrap();
        casted.remove(&child.widget);
    }
}
