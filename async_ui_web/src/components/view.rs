use crate::{utils::class_list::ClassList, window::DOCUMENT, Fragment};

use super::ElementFuture;

#[derive(Default)]
pub struct ViewProps<'c> {
    pub children: Option<Fragment<'c>>,
    pub class: Option<&'c ClassList<'c>>,
}
pub async fn view<'c>(ViewProps { children, class }: ViewProps<'c>) {
    let elem = DOCUMENT.with(|doc| doc.create_element("div").expect("create element failed"));
    if let Some(class) = class {
        class.set_dom(elem.class_list());
    }
    ElementFuture::new(children.unwrap_or_default(), elem.into()).await;
}
