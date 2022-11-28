use crate::{utils::class_list::ClassList, window::DOCUMENT, Fragment};

use super::ElementFuture;

pub struct ViewProps<'c> {
    pub children: Fragment<'c>,
    pub class: Option<&'c ClassList<'c>>,
    pub element_tag: &'c str,
}
impl<'c> Default for ViewProps<'c> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            class: Default::default(),
            element_tag: "div",
        }
    }
}
/** View - HTML <div> element
 *
 * You can customize the `element_tag` prop to use other tags (main, h3, etc.).
 */
pub async fn view<'c>(
    ViewProps {
        children,
        class,
        element_tag,
    }: ViewProps<'c>,
) {
    let elem = DOCUMENT.with(|doc| {
        doc.create_element(element_tag)
            .expect("create element failed")
    });
    if let Some(class) = class {
        class.set_dom(elem.class_list());
    }
    ElementFuture::new(children, elem.into()).await;
}
