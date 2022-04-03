use std::hash::Hash;

use async_ui_core::local::wrappers as base;
use async_ui_reactive::Rx;

use crate::{manual_apis::WebBackend, Element};
pub type PortalEntry = base::portal::PortalEntry<WebBackend>;
pub type PortalExit = base::portal::PortalExit<WebBackend>;
pub fn create_portal() -> (PortalEntry, PortalExit) {
    base::portal::create_portal()
}

pub async fn list<K: Eq + Hash + Clone>(children: &Rx<Vec<(K, Option<Element<'_>>)>>) {
    base::list::list(children).await
}
pub async fn hidable(is_visible: &Rx<bool>, children: Vec<Element<'_>>) {
    base::hidable::hidable(is_visible, children).await
}
