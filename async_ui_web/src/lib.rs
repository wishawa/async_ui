mod dynamic;
mod dynamic_list;
pub mod executor;
mod keyed_list;
mod mount;
mod no_child;
mod shortcuts;
mod utils;
mod virtualized_list;

pub use async_ui_internal_utils::reactive_cell::ReactiveCell;
pub use async_ui_web_components::components;
pub use async_ui_web_core::combinators::{join, race, race_ok, try_join};
pub use async_ui_web_macros::css;
pub use async_ui_web_macros::select;
pub use dynamic::Dynamic;
pub use dynamic_list::DynamicList;
pub use keyed_list::KeyedList;
pub use mount::{mount, mount_at};
pub use no_child::NoChild;
pub use virtualized_list::VirtualizedList;

pub mod event_handling {
    /*!
    Types used in event handling mechanism.
    You shouldn't need to interact with this module directly often.
    */
    pub use async_ui_web_components::events::EventFutureStream;
}

pub mod event_traits {
    /*!
    Traits for event handling.
    */
    pub use async_ui_web_components::events::{EmitEditEvent, EmitElementEvent, EmitEvent};
}
pub mod shortcut_traits {
    /*!
    Convenience traits.
    ```
    # use super::components::Button;
    let b = Button::new();

    ```
     */
    pub use super::shortcuts::{ShortcutClassList, ShortcutClassListBuilder, ShortcutRenderStr};
}

pub mod prelude_traits {
    /*!
    Includes traits from [event_traits][super::event_traits]
    and [shortcut_traits][super::shortcut_traits].
    ```
    # use crate as async_ui_web;
    use async_ui_web::prelude_traits::*;
    ```
     */
    pub use super::shortcuts::{
        ShortcutClassList as _, ShortcutClassListBuilder as _, ShortcutRenderStr as _,
    };
    pub use async_ui_web_components::events::{
        EmitEditEvent as _, EmitElementEvent as _, EmitEvent as _,
    };
}
