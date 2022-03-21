mod executor;
mod spawner;

use std::{future::Future, pin::Pin};

use crate::shared::SpawnWrappedFuture;

pub type Task = executor::Task;
type SpawnJob = SpawnWrappedFuture<dyn Future<Output = ()> + 'static>;
use executor::spawn;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub use executor::start_executor;
pub use spawner::*;
