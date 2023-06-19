use std::future::Pending;

use async_ui_web_components::components::Text;
use async_ui_web_core::ContainerNodeFuture;
use js_sys::Array;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

pub trait ShortcutRenderStr {
    fn render(&self) -> ContainerNodeFuture<Pending<()>>;
}
impl ShortcutRenderStr for str {
    fn render(&self) -> ContainerNodeFuture<Pending<()>> {
        let t = Text::new();
        t.set_data(self);
        t.render()
    }
}

pub trait ShortcutClassList {
    fn add_class(&self, c: &str);
    fn add_classes<'a>(&self, c: impl Iterator<Item = &'a str>);
    fn del_class(&self, c: &str);
    fn del_classes<'a>(&self, c: impl Iterator<Item = &'a str>);
    fn set_class(&self, c: &str, included: bool);
}

fn strs_to_js_array<'a>(values: impl Iterator<Item = &'a str>) -> Array {
    values.into_iter().map(|x| JsValue::from_str(x)).collect()
}

impl ShortcutClassList for web_sys::Element {
    fn add_class(&self, c: &str) {
        self.class_list().add_1(c).unwrap();
    }
    fn add_classes<'a>(&self, c: impl Iterator<Item = &'a str>) {
        self.class_list().add(&strs_to_js_array(c)).unwrap_throw();
    }

    fn del_class(&self, c: &str) {
        self.class_list().remove_1(c).unwrap();
    }
    fn del_classes<'a>(&self, c: impl Iterator<Item = &'a str>) {
        self.class_list()
            .remove(&strs_to_js_array(c))
            .unwrap_throw();
    }

    fn set_class(&self, c: &str, included: bool) {
        self.class_list()
            .toggle_with_force(c, included)
            .unwrap_throw();
    }
}
