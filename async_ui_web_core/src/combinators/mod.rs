mod common;
mod join;
mod race;
mod race_ok;
mod try_join;
mod utils;

/// Wait for multiple futures to complete.
///
/// Join takes in many "subfutures" and return a single Future.
/// When awaited, the returned Future will drive all the subfutures
/// to completion and return all their results.
///
/// Subfutures may be passed in as either
/// * a tuple of up to 12 Futures (signature: `(F1, F2, ...) -> (F1::Output, F2::Output, ...)`)
/// * an array of Futures (signature: `[F; N] -> [F::Output; N]`)
/// * a [Vec] of Futures (signature: `Vec<F> -> Vec<F::Output>`)
///
/// ```rust
/// # use async_ui_web_core::combinators::join;
/// # let _ = async {
/// async fn do_something(input: i32) -> i32 {
///     // ...make a network request of something...
///     input * 2
/// }
/// // Join 2-tuple of Futures
/// let (res_1, res_2) = join((
///     do_something(21),
///     do_something(100)
/// )).await;
/// assert_eq!(res_1, 42);
/// assert_eq!(res_2, 200);
///
/// // Join array of Futures
/// let results: [i32; 20] = join(
///     core::array::from_fn(|idx| do_something(idx as i32))
/// ).await;
/// assert_eq!(
///     &results,
///     &*(0..20).map(|x| x * 2).collect::<Vec<_>>()
/// );
///
/// // Join vector of Futures
/// let results: Vec<i32> = join(
///     (0..100).map(|i| do_something(i as i32)).collect::<Vec<_>>()
/// ).await;
/// assert_eq!(
///     results,
///     (0..100).map(|x| x * 2).collect::<Vec<_>>()
/// );
/// # };
/// ```
pub fn join<F: join::Join>(f: F) -> F::Future {
    f.join()
}

/// Wait for the first future to complete.
///
/// Race takes in many "subfutures" and return a single Future.
/// When awaited, the returned Future will drive all of the subfuture until
/// any of them complete, and return the result of that completed subfuture.
///
/// Subfutures may be passed in as either
/// *   a tuple of up to 12 Futures, all with the same output type
///     (signature: `(F1, F2, F3, ...) -> Output`)
/// *   an array of Futures (signature: `[F; N] -> F::Output`)
/// *   a [Vec] of Futures (signature: `Vec<F> -> F::Output`)
///
/// ```rust
/// # use async_ui_web_core::combinators::race;
/// # let _ = async {
/// async fn do_something(input: i32) -> i32 {
///     // ...make a network request of something...
///     input * 2
/// }
/// // Race 2-tuple of Futures
/// let result = race((
///     do_something(21),
///     do_something(100)
/// )).await;
/// // Don't know which one will win the race.
/// assert!(result == 42 || result == 200);
///
/// // Race array of Futures
/// let result: i32 = race(
///     core::array::from_fn::<_, 10, _>(|idx| do_something(idx as i32))
/// ).await;
///
/// // Race vector of Futures
/// let result: i32 = race(
///     (0..100).map(|i| do_something(i as i32)).collect::<Vec<_>>()
/// ).await;
/// # };
/// ```
pub fn race<F: race::Race>(f: F) -> F::Future {
    f.race()
}

/// Wait for all futures to complete successfully, or return early on error.
///
/// TryJoin takes in many fallible (returns [Result]) "subfutures" and return
/// a single Future.
/// When awaited, the returned Future will drive all of the subfuture until
/// either all of them return `Ok(_)` or any of them return `Err(_)`.
///
/// Subfutures may be passed in as either
/// *   a tuple of up to 12 Futures, all with the same Error type
///     (signature: `(F1, F2, ...) -> Result<(F1::Ok, F2::Ok, ...), Error>`)
/// *   an array of Futures
///     (signature: `[F; N] -> Result<[F::Ok; N], F::Error>`)
/// *   a [Vec] of Futures
///     (signature: `Vec<F> -> Result<Vec<F::Ok>, F::Error>`)
pub fn try_join<F: try_join::TryJoin>(f: F) -> F::Future {
    f.try_join()
}

/// Wait for any future to complete successfully.
///
/// RaceOk takes in many fallible (returns [Result]) "subfutures" and return
/// a single Future.
/// When awaited, the returned Future will drive all of the subfuture until
/// either one of them return `Ok(_)` or all of them return `Err(_)`.
///
/// Subfutures may be passed in as either
/// *   a tuple of up to 12 Futures, all with the same Ok type
///     (signature: `(F1, F2, ...) -> Result<Ok, (F1::Error, F2::Error, ...)>`)
/// *   an array of Futures
///     (signature: `[F; N] -> Result<F::Ok, [F::Error; N]>`)
/// *   a [Vec] of Futures
///     (signature: `Vec<F> -> Result<F::Ok, Vec<F::Error>>`)
pub fn race_ok<F: race_ok::RaceOk>(f: F) -> F::Future {
    f.race_ok()
}

#[cfg(test)]
fn block_for_testing<F: core::future::Future>(f: F) -> F::Output {
    use crate::context::{DomContext, DOM_CONTEXT};
    DOM_CONTEXT.set(&DomContext::Null, || futures_lite::future::block_on(f))
}
