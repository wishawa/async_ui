use async_ui_core::local::element::Element as ElementBase;

use crate::backend::WebBackend;

pub type Element<'e> = ElementBase<'e, WebBackend>;
