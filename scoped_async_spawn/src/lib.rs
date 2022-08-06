#![deny(unsafe_op_in_unsafe_fn)]
pub mod boxed;
mod common;
mod many;
mod one;
mod pointer;
mod scope;

pub use common::RemoteStaticFuture;
pub use many::{ExecutorSpawn, SpawnGuard, SpawnedTask};
pub use one::SpawnedFuture;
pub use scope::GiveUnforgettableScope;
