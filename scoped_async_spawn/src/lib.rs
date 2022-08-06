#![deny(unsafe_op_in_unsafe_fn)]
pub mod boxed;
mod common;
mod guard;
mod pointer;
mod scope;

pub use common::RemoteStaticFuture;
pub use guard::SpawnGuard;
pub use scope::GiveUnforgettableScope;
