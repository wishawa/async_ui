use async_ui_gtk::{manual_apis::ContainerHandler, Element};
use glib::{Cast, IsA};
use gtk::{
    traits::{BoxExt, ButtonExt},
    Widget,
};

use crate::elem::WrappedWidget;

trait SingleChildContainer {
    fn set_single_child(&self, child: Option<&Widget>);
}
trait MultiChildContainer {
    fn insert_child_after(&self, child: &Widget, sibling: Option<&Widget>);
    fn remove_child(&self, child: &Widget);
    fn move_child_after(&self, child: &Widget, sibling: Option<&Widget>);
}

pub trait GtkContainerWidget: IsA<Widget> {
    fn get_handler() -> &'static dyn ContainerHandler;
}

impl<'a, H: GtkContainerWidget + 'a> WrappedWidget<'a, H> {
    pub fn children(mut self, children: Vec<Element<'a>>) -> Self {
        self.children = Some((children, H::get_handler()));
        self
    }
}

macro_rules! make_handler_multi {
	($name:ident, $handler:ident) => {
		paste::paste! {
			struct [<$handler>];
			impl ContainerHandler for [<$handler>] {
				fn get_support_multichild(&self) -> bool {
					true
				}
				fn insert_child_after(&self, this: &Widget, child: &Widget, previous_sibling: Option<&Widget>) {
					let downcasted: &gtk::$name = this.downcast_ref().unwrap();
					MultiChildContainer::insert_child_after(downcasted, child, previous_sibling);
				}
				fn remove_child(&self, this: &Widget, child: &Widget) {
					let downcasted: &gtk::$name = this.downcast_ref().unwrap();
					MultiChildContainer::remove_child(downcasted, child);
				}
				fn reorder_child_after(
					&self,
					this: &Widget,
					child: &Widget,
					new_previous_sibling: Option<&Widget>,
				) {
					let downcasted: &gtk::Box = this.downcast_ref().unwrap();
					MultiChildContainer::move_child_after(downcasted, child, new_previous_sibling);
				}
			}

		}
	};
	($name:ident, $handler:ident, _) => {
		make_handler_multi! ($name, $handler);
		paste::paste! {
			static [<$name:snake:upper _HANDLER>]: [<$handler>] = [<$handler>];
			impl GtkContainerWidget for gtk::$name {
				fn get_handler() -> &'static dyn ContainerHandler {
					&[<$name:snake:upper _HANDLER>]
				}
			}
		}
	}
}
macro_rules! make_handler_single {
    ($name:ident, $handler:ident) => {
        paste::paste! {
            struct [<$handler>];
            impl ContainerHandler for [<$handler>] {
                fn get_support_multichild(&self) -> bool {
                    false
                }
                fn set_single_child(&self, this: &Widget, child: Option<&Widget>) {
                    let downcasted: &gtk::$name = this.downcast_ref().unwrap();
                    SingleChildContainer::set_single_child(downcasted, child);
                }
            }

        }
    };
    ($name:ident, $handler:ident, _) => {
        make_handler_single!($name, $handler);
        paste::paste! {
            static [<$name:snake:upper _HANDLER>]: [<$handler>] = [<$handler>];
            impl GtkContainerWidget for gtk::$name {
                fn get_handler() -> &'static dyn ContainerHandler {
                    &[<$name:snake:upper _HANDLER>]
                }
            }
        }
    };
}
impl SingleChildContainer for gtk::Button {
    fn set_single_child(&self, child: Option<&Widget>) {
        self.set_child(child);
    }
}
make_handler_single!(Button, ButtonHandler, _);

impl SingleChildContainer for gtk::ScrolledWindow {
    fn set_single_child(&self, child: Option<&Widget>) {
        self.set_child(child);
    }
}
make_handler_single!(ScrolledWindow, ScrolledWindowHandler, _);

impl MultiChildContainer for gtk::Box {
    fn insert_child_after(&self, child: &Widget, sibling: Option<&Widget>) {
        BoxExt::insert_child_after(self, child, sibling);
    }
    fn remove_child(&self, child: &Widget) {
        self.remove(child);
    }
    fn move_child_after(&self, child: &Widget, sibling: Option<&Widget>) {
        self.reorder_child_after(child, sibling);
    }
}
make_handler_multi!(Box, BoxHandler, _);
