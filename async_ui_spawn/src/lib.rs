#![deny(unsafe_op_in_unsafe_fn)]
pub mod multithread;
mod shared;
pub mod singlethread;
pub mod wasm;
pub use shared::{
    check_drop_guarantee, check_drop_guarantee_async, is_unmounting, RootSpawnWrappedFuture,
};
