mod mutex;
mod ref_cell;
mod rw_lock;
mod shared;

pub mod multithread {
    pub use super::mutex::channel::*;
    pub use super::mutex::reactive::Reactive as ReactiveMutex;
    pub use super::rw_lock::reactive::Reactive as ReactiveRwLock;
}
pub mod singlethread {
    pub use super::ref_cell::channel::*;
    pub use super::ref_cell::reactive::Reactive as ReactiveRefCell;
}
