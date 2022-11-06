/*!
 * # `scoped_async_spawn`
 *
 * This crate can be used to spawn non-'static futures.
 *
 * For most people, [join](https://docs.rs/futures/latest/futures/future/fn.join.html) or [FuturesUnordered](https://docs.rs/futures/latest/futures/stream/struct.FuturesUnordered.html) should suffice.
 * Only use this crate if you know you need to.
 *
 * This crate allows you to convert non-'static futures into 'static ones.
 * The [SpawnGuard] type keeps track of conversion and will end any future that goes beyond its lifetime.
 *
 * To enforce that the guard must not be forgotten, the guard only workss when it is pinned to a local scope.
 * **Pinning the guard inside a box or other heap pointer will cause panic.**
 *
 * This crate might be subtly unsound.
 * Pinning and leaking the guard in some form of stack allocation can cause unsound behavior (this is only possible with additional unsafe code).
 * See [issue](https://github.com/wishawa/async_ui/issues/6).
 */
#![deny(unsafe_op_in_unsafe_fn)]
pub mod boxed;
mod common;
mod guard;
mod pointer;
mod scope;

pub use common::RemoteStaticFuture;
pub use guard::SpawnGuard;
pub use scope::GiveUnforgettableScope;
