#![deny(unsafe_op_in_unsafe_fn)]
pub mod multithread;
mod shared;
pub mod singlethread;
pub mod wasm;
pub use shared::RootSpawnWrappedFuture;
pub mod unmounting;
