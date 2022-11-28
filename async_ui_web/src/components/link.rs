use futures_lite::FutureExt;
use observables::{ObservableAs, ObservableAsExt};
use smallvec::SmallVec;
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;

use crate::{utils::class_list::ClassList, window::DOCUMENT, Fragment};

use super::{
    button::PressEvent,
    events::{create_handler, EventsManager, QueuedEvent},
    ElementFuture,
};

#[derive(Default)]
pub struct LinkProps<'c> {
    pub children: Fragment<'c>,
    pub href: Option<&'c dyn ObservableAs<str>>,
    pub on_press: Option<&'c mut dyn FnMut(PressEvent)>,
    pub class: Option<&'c ClassList<'c>>,
}

pub async fn link<'c>(
    LinkProps {
        href,
        mut on_press,
        class,
        children,
    }: LinkProps<'c>,
) {
    let anchor = DOCUMENT.with(|doc| {
        let elem = doc.create_element("a").expect("create element failed");
        let elem: HtmlAnchorElement = elem.unchecked_into();
        elem
    });
    let href = href.unwrap_or(&[""]);

    let mut handlers = SmallVec::<[_; 1]>::new();
    let manager = EventsManager::new();

    if on_press.is_some() {
        let h = create_handler(&manager, |e| QueuedEvent::Click(e));
        anchor.set_onclick(Some(h.get_function()));
        handlers.push(h);
    }
    if let Some(class) = class {
        class.set_dom(anchor.class_list());
    }
    let anchor_copy = anchor.clone();

    let future = (children)
        .or(async {
            manager.grab_waker().await;
            loop {
                let mut events = manager.get_queue().await;
                for event in events.drain(..) {
                    match event {
                        QueuedEvent::Click(native_event) => {
                            on_press.as_mut().map(|f| f(PressEvent { native_event }));
                        }
                        _ => {}
                    }
                }
            }
        })
        .or(href.for_each(|href| {
            if !href.is_empty() {
                anchor_copy.set_href(href);
            } else {
                anchor_copy
                    .remove_attribute("href")
                    .expect("anchor remove attribute failed");
            }
        }));
    ElementFuture::new(future, anchor.into()).await
}
