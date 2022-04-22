mod connect;
mod containers;
mod elem;
mod list_view;
pub use elem::{Wrappable, WrappedWidget};
pub use list_view::{list_view, ListViewItems};
mod widgets;
// use std::future::pending;

// use async_ui_gtk::manual_apis::put_node;
// use glib::Cast;
// use gtk::{Label, Widget};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
