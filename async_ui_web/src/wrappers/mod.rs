use std::hash::Hash;

use async_ui_core::wrappers as base;
use async_ui_reactive::local::Rx;

use crate::{manual_apis::WebBackend, render::Render};
pub type PortalEntry = base::portal::PortalEntry<WebBackend>;
pub type PortalExit = base::portal::PortalExit<WebBackend>;
pub fn create_portal() -> (PortalEntry, PortalExit) {
    base::portal::create_portal()
}

pub async fn list<K: Eq + Hash + Clone>(children: &Rx<Vec<(K, Option<Render<'_>>)>>) {
    base::list::list(children).await
}
pub async fn hidable<'e>(is_visible: &Rx<bool>, children: impl Into<Render<'e>>) {
    base::hidable::hidable(is_visible, children).await
}
