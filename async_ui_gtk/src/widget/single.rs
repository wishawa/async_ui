use super::{SingleChildWidgetOp, WrappedWidget};
use glib::Cast;
macro_rules! impl_for {
    ($name:ident, $opname:ident) => {
        pub struct $opname;
        impl SingleChildWidgetOp for $opname {
            fn set_child(&self, this: &glib::Object, child: &mut WrappedWidget) {
                let casted = this.downcast_ref::<gtk::$name>().unwrap();
                casted.set_child(Some(&child.widget));
            }

            fn get_child(&self, this: &glib::Object) -> Option<gtk::Widget> {
                let casted = this.downcast_ref::<gtk::$name>().unwrap();
                casted.child()
            }

            fn unset_child(&self, this: &glib::Object) {
                let casted = this.downcast_ref::<gtk::$name>().unwrap();
                casted.set_child(Option::<&gtk::Widget>::None);
            }
        }
    };
}

impl_for!(ScrolledWindow, ScrolledWindowOp);

use gtk::prelude::FrameExt;
impl_for!(Frame, FrameOp);

impl_for!(Expander, ExpanderOp);
use gtk::prelude::ButtonExt;
impl_for!(Button, ButtonOp);
