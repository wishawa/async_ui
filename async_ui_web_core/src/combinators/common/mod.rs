//! This module implements a combinator similar to TryJoin.
//! The actual TryJoin, along with Join, Race, and RaceOk, can delegate to this.

mod array;
mod tuple;
mod vec;

pub(crate) use array::{CombinatorArray, CombinatorBehaviorArray};
pub(crate) use tuple::{CombineTuple, TupleMaybeReturn, TupleWhenCompleted};
pub(crate) use vec::{CombinatorBehaviorVec, CombinatorVec};
