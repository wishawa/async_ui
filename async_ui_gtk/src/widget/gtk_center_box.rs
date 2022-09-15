use glib::Cast;

use super::{SingleChildWidgetOp, WrappedWidget};

pub struct GtkCenterBoxOpStart;
pub struct GtkCenterBoxOpCenter;
pub struct GtkCenterBoxOpEnd;
macro_rules! impl_for {
    ($name:ident, $set_wg:ident, $get_wg:ident) => {
        impl SingleChildWidgetOp for $name {
            fn set_child(&self, this: &gtk::Widget, child: &mut WrappedWidget) {
                let casted = this.downcast_ref::<gtk::CenterBox>().unwrap();
                casted.$set_wg(Some(&child.widget));
            }

            fn get_child(&self, this: &gtk::Widget) -> Option<gtk::Widget> {
                let casted = this.downcast_ref::<gtk::CenterBox>().unwrap();
                casted.$get_wg()
            }
            fn unset_child(&self, this: &gtk::Widget) {
                let casted = this.downcast_ref::<gtk::CenterBox>().unwrap();
                casted.$set_wg(Option::<&gtk::Widget>::None);
            }
        }
    };
}

impl_for!(GtkCenterBoxOpStart, set_start_widget, start_widget);
impl_for!(GtkCenterBoxOpCenter, set_center_widget, center_widget);
impl_for!(GtkCenterBoxOpEnd, set_end_widget, end_widget);
