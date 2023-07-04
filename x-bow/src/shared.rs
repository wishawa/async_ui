use std::cell::RefCell;

use async_ui_internal_utils::wakers_list::WakersList;

#[derive(Default)]
pub struct Shared {
    pub allocator: bumpalo::Bump,
    pub wakers_list: RefCell<WakersList>,
}
