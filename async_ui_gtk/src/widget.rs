use glib::Cast;
use gtk::{
    traits::{BoxExt, WidgetExt},
    Widget,
};

use self::gtk_box::GtkBoxOp;
mod gtk_box;
mod gtk_center_box;
mod gtk_flow_box;
mod single;

pub trait WrappedWidgetTrait {}
pub struct WrappedWidget {
    widget: Widget,
    inner_widget: Option<Widget>,
    op: WidgetOp,
}

impl WrappedWidget {
    pub fn add_child_node(&mut self, child: &mut Self, insert_before_sibling: Option<&Self>) {
        let this = self.inner_widget.as_ref().unwrap_or(&self.widget);
        match self.op {
            WidgetOp::MultiChild(mc) => {
                mc.add_child(this, child, insert_before_sibling);
            }
            WidgetOp::SingleChild(sc) => {
                if let Some(exs) = sc.get_child(this) {
                    let b = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                    b.append(&exs);
                    let widget: Widget = b.upcast();
                    let op = WidgetOp::MultiChild(&GtkBoxOp);
                    sc.set_child(
                        this,
                        &mut WrappedWidget {
                            widget: widget.clone(),
                            inner_widget: None,
                            op,
                        },
                    );
                    self.inner_widget = Some(widget);
                    self.op = op;
                    self.add_child_node(child, insert_before_sibling);
                } else {
                    sc.set_child(this, child);
                }
            }
            WidgetOp::NoChild => {}
        }
    }
    pub fn del_child_node(&mut self, child: &mut Self) {
        let this = self.inner_widget.as_ref().unwrap_or(&self.widget);
        match self.op {
            WidgetOp::MultiChild(mc) => {
                mc.remove_child(this, child);
            }
            WidgetOp::SingleChild(sc) => {
                sc.unset_child(this);
            }
            WidgetOp::NoChild => {}
        }
    }
}

#[derive(Clone, Copy)]
pub enum WidgetOp {
    MultiChild(&'static dyn MultiChildWidgetOp),
    SingleChild(&'static dyn SingleChildWidgetOp),
    NoChild,
}

pub trait MultiChildWidgetOp {
    fn add_child(&self, this: &Widget, child: &mut WrappedWidget, before: Option<&WrappedWidget>) {
        child.widget.insert_before(this, before.map(|w| &w.widget));
    }
    fn remove_child(&self, this: &Widget, child: &mut WrappedWidget);
}

pub trait SingleChildWidgetOp {
    fn set_child(&self, this: &Widget, child: &mut WrappedWidget);
    fn get_child(&self, this: &Widget) -> Option<Widget>;
    fn unset_child(&self, this: &Widget);
}
