use async_ui_core::local::element::Element as ElementBase;

use crate::backend::GtkBackend;

pub type Element<'e> = ElementBase<'e, GtkBackend>;
