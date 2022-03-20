pub use futures::{join, try_join};
use std::future::Future;

#[macro_export]
macro_rules! race {
	($($e:expr),*) => {
		($crate::future::try_join! (
			$(async {
				::std::result::Result::<(), _>::Err(($e).await)
			}),*
		)).unwrap_err()
	};
}
macro_rules! make_race_fn {
	($name:ident, $(($an:ident, $tn:ident),)+) => {
		pub async fn $name <$($tn,)+ R> ($($an : $tn,)+) -> R
		where $($tn: Future<Output = R>,)+
		{
			race!($($an),+)
		}
	};
}
make_race_fn!(race2, (fut1, F1), (fut2, F2),);
make_race_fn!(race3, (fut1, F1), (fut2, F2), (fut3, F3),);
make_race_fn!(race4, (fut1, F1), (fut2, F2), (fut3, F3), (fut4, F4),);
make_race_fn!(
    race5,
    (fut1, F1),
    (fut2, F2),
    (fut3, F3),
    (fut4, F4),
    (fut5, F5),
);
pub async fn race_all<I>(iter: I) -> <I::Item as Future>::Output
where
    I: IntoIterator,
    I::Item: Future,
{
    use futures::{future::try_join_all, FutureExt};
    try_join_all(
        iter.into_iter()
            .map(|f| f.map(|res| Result::<(), _>::Err(res))),
    )
    .await
    .unwrap_err()
}
pub use futures::future::{join, join3, join4, join5, join_all};
