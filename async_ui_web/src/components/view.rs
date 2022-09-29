use crate::{utils::class_list::ClassList, window::DOCUMENT, Fragment};

use super::ElementFuture;

pub enum ViewProp<'c> {
    Children(Fragment<'c>),
    Class(&'c ClassList<'c>),
}
pub async fn view<'c, I: IntoIterator<Item = ViewProp<'c>>>(props: I) {
    let mut children = None;
    let elem = DOCUMENT.with(|doc| doc.create_element("div").expect("create element failed"));
    for prop in props {
        match prop {
            ViewProp::Children(v) => children = Some(v),
            ViewProp::Class(v) => {
                v.set_dom(elem.class_list());
            }
        }
    }
    ElementFuture::new(children.unwrap_or_default(), elem.into()).await;
}
