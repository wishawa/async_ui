mod dynamic_list;
pub mod executor;
mod mount;
mod no_child;
mod shortcuts;

pub use async_ui_internal_utils::reactive_cell::ReactiveCell;
pub use async_ui_web_components::components;
pub use async_ui_web_core::combinators::{join, race, race_ok, try_join};
pub use async_ui_web_macros::css;
pub use async_ui_web_macros::select;
pub use dynamic_list::DynamicList;
pub use mount::{mount, mount_at};
pub use no_child::NoChild;

pub mod event_handling {
    pub use async_ui_web_components::events::EventFutureStream;
}

pub mod event_traits {
    pub use async_ui_web_components::events::{EmitEditEvent, EmitElementEvent, EmitEvent};
}
pub mod shortcut_traits {
    pub use super::shortcuts::{ShortcutClassList, ShortcutRenderStr};
}

pub mod prelude_traits {
    pub use super::shortcuts::{ShortcutClassList as _, ShortcutRenderStr as _};
    pub use async_ui_web_components::events::{
        EmitEditEvent as _, EmitElementEvent as _, EmitEvent as _,
    };
}
