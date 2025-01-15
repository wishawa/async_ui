use std::future::Pending;

use async_ui_web_core::dom::Element;
use async_ui_web_core::ContainerNodeFuture;
use async_ui_web_html::nodes::Text;
use js_sys::Array;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

pub trait ShortcutRenderStr {
    /// Render the [str] as an HTML text node with that content.
    ///
    /// ```
    /// # use async_ui_web::shortcut_traits::ShortcutRenderStr;
    /// # let _ = async {
    /// "hello world".render().await;
    /// # };
    /// ```
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
    /// Add a classname to this element, if not already present.
    ///
    /// This method is equivalent to `elem.class_list().add_1(class_name).unwrap_throw()`.
    fn add_class(&self, c: &str);
    /// Add classnames to this element, ignoring those already present.
    fn add_classes<'a>(&self, c: impl IntoIterator<Item = &'a str>);
    /// Remove a classname from this element, if present.
    ///
    /// This method is equivalent to `elem.class_list().remove_1(class_name).unwrap_throw()`.
    fn del_class(&self, c: &str);
    /// Remove classnames from this element, ignoring those not present.
    fn del_classes<'a>(&self, c: impl IntoIterator<Item = &'a str>);
    /// If the boolean `included` argument is true, add the classname to the element.
    /// If the flag is false, remove the classname from the element.
    ///
    /// This method is equivalent to `elem.class_list().toggle_with_force(c, included).unwrap_throw()`.
    fn set_class(&self, c: &str, included: bool);
}

/// Convert an iterator of str to a JS array of strings.
fn strs_to_js_array<'a>(values: impl Iterator<Item = &'a str>) -> Array {
    values.into_iter().map(JsValue::from_str).collect()
}

impl ShortcutClassList for Element {
    fn add_class(&self, c: &str) {
        self.class_list().add_1(c).unwrap_throw();
    }
    #[cfg(feature = "csr")]
    fn add_classes<'a>(&self, c: impl IntoIterator<Item = &'a str>) {
        self.class_list()
            .add(&strs_to_js_array(c.into_iter()))
            .unwrap_throw();
    }
    #[cfg(feature = "ssr")]
    fn add_classes<'a>(&self, c: impl IntoIterator<Item = &'a str>) {
        self.class_list().add(c).unwrap_throw();
    }

    fn del_class(&self, c: &str) {
        self.class_list().remove_1(c).unwrap();
    }
    #[cfg(feature = "csr")]
    fn del_classes<'a>(&self, c: impl IntoIterator<Item = &'a str>) {
        self.class_list()
            .remove(&strs_to_js_array(c.into_iter()))
            .unwrap_throw();
    }
    #[cfg(feature = "ssr")]
    fn del_classes<'a>(&self, c: impl IntoIterator<Item = &'a str>) {
        self.class_list().remove(c).unwrap_throw();
    }

    fn set_class(&self, c: &str, included: bool) {
        self.class_list()
            .toggle_with_force(c, included)
            .unwrap_throw();
    }
}

pub trait ShortcutClassListBuilder: AsRef<Element> {
    /// Add a classname to the element and return reference to the input.
    ///
    /// This is for writing the UI "declaratively".
    /// ```
    /// # let _ = async {
    /// # use async_ui_web::{html::Div, prelude_traits::*};
    /// # let children = std::future::pending::<()>();
    /// Div::new().with_class("my-wrapper").render(children).await;
    /// # };
    /// ```
    fn with_class(&self, c: &str) -> &Self {
        self.as_ref().add_class(c);
        self
    }
    /// Add classnames to the element and return reference to the input.
    ///
    /// This is for writing the UI "declaratively".
    /// ```
    /// # let _ = async {
    /// # use async_ui_web::{html::Div, prelude_traits::*};
    /// # let children = std::future::pending::<()>();
    /// Div::new().with_classes(["my-wrapper", "flex-vertical"]).render(children).await;
    /// # };
    /// ```
    fn with_classes<'a>(&self, c: impl IntoIterator<Item = &'a str>) -> &Self {
        self.as_ref().add_classes(c.into_iter());
        self
    }
}
impl<T: AsRef<Element>> ShortcutClassListBuilder for T {}
