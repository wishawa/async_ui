use async_ui_core::element::Element as ElementBase;

use crate::backend::WebBackend;

pub type Element<'e> = ElementBase<'e, WebBackend>;
