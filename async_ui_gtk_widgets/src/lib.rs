use std::future::pending;

use async_ui_gtk::manual_apis::put_node;
use glib::Cast;
use gtk::{Label, Widget};

pub async fn label(text: &str) {
    let label = Label::new(Some(text));
    let widget: Widget = label.upcast();
    let _guard = put_node(widget);
    pending().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
