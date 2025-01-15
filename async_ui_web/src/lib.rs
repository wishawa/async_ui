/*!
# Async UI Web: Web UI with Async Rust

See the [guide book](https://wishawa.github.io/async_ui/book/) to get started!

*/

pub mod components;
#[cfg(feature = "csr")]
pub mod executor;
#[cfg(feature = "ssr")]
#[path = "./executor_ssr.rs"]
pub mod executor;
pub mod lists;
#[cfg(feature = "csr")]
mod mount;
mod no_child;
mod shortcuts;
#[cfg(feature = "ssr")]
mod ssr;

pub use async_ui_internal_utils::reactive_cell::ReactiveCell;
pub use async_ui_web_core::combinators::{join, race, race_ok, try_join};
pub use async_ui_web_html::nodes as html;
pub use async_ui_web_macros::css;
pub use async_ui_web_macros::select;
#[cfg(feature = "csr")]
pub use mount::{mount, mount_at};
#[cfg(feature = "ssr")]
pub use ssr::render_to_string;
pub use no_child::NoChild;

#[doc(hidden)]
pub mod __private_macro_only {
    #[doc(hidden)]
    pub use wasm_bindgen;
}

pub mod event_handling {
    /*!
    Types used in event handling mechanism.
    You shouldn't need to interact with this module directly often.
    */
    pub use async_ui_web_html::events::EventFutureStream;
}

pub mod event_traits {
    /*!
    Traits for event handling.
    */
    pub use async_ui_web_html::events::{EmitElementEvent, EmitEvent, EmitHtmlElementEvent};
}

pub mod shortcut_traits {
    /*!
    Traits provided for convenience.
     */
    pub use super::shortcuts::{ShortcutClassList, ShortcutClassListBuilder, ShortcutRenderStr};
    pub use async_ui_web_core::combinators::UiFutureExt;
}

pub mod prelude_traits {
    /*!
    Includes all traits from [event_traits][super::event_traits]
    and [shortcut_traits][super::shortcut_traits].
    ```
    use async_ui_web::prelude_traits::*;
    ```
     */
    pub use super::shortcuts::{
        ShortcutClassList as _, ShortcutClassListBuilder as _, ShortcutRenderStr as _,
    };
    pub use async_ui_web_core::combinators::UiFutureExt as _;
    pub use async_ui_web_html::events::{
        EmitElementEvent as _, EmitEvent as _, EmitHtmlElementEvent as _,
    };
}
