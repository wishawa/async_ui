use std::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

pub struct Race<F> {
	futures: F,
}
pub struct Join<F, O> {
	futures: F,
	outputs: O,
}

macro_rules! make_tuples {
	($($id:expr),+) => {
		paste::paste! {
			impl<T, $([<F $id>]: Future<Output = T>),+> From<($([<F $id>],)+)> for Race<($([<F $id>],)+)> {
                fn from(futures: ($([<F $id>],)+)) -> Self {
                    Self {
                        futures
                    }
                }
            }
			impl<T, $([<F $id>]: Future<Output = T>),+> Future for Race<($([<F $id>],)+)> {
                type Output = T;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                    let inner = &mut unsafe { Pin::get_unchecked_mut(self) }.futures;
                    $({
                        let fut: &mut [<F $id>] = &mut inner.$id;
                        let pinned = unsafe { Pin::new_unchecked(fut) };
                        if let Poll::Ready(v) = pinned.poll(cx) {
                            return Poll::Ready(v);
                        }
                    })+;
                    Poll::Pending
                }
			}
            impl<$([<F $id>]: Future),+> From<($([<F $id>],)+)> for Join<($([<F $id>],)+), ($(Option<[<F $id>]::Output>,)+)> {
                fn from(futures: ($([<F $id>],)+)) -> Self {
                    let outputs = ($(Option::<[<F $id>]::Output>::None,)+);
                    Self {
                        futures,
                        outputs
                    }
                }
            }
			impl<$([<F $id>]: Future),+> Future for Join<($([<F $id>],)+), ($(Option<[<F $id>]::Output>,)+)> {
                type Output = ($([<F $id>]::Output,)+);
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let mut inner = &mut unsafe { Pin::get_unchecked_mut(self) };
                    $({
                        if inner.outputs.$id.is_none() {
                            let fut: &mut [<F $id>] = &mut inner.futures.$id;
                            let pinned = unsafe { Pin::new_unchecked(fut) };
                            if let Poll::Ready(v) = pinned.poll(cx) {
                                inner.outputs.$id = Some(v);
                            }
                        }
                    })+;
                    inner.outputs = match std::mem::take(&mut inner.outputs) {
                        ($(Some([<v_ $id>]),)+) => return Poll::Ready(($([<v_ $id>],)+)),
                        x => x
                    };
                    Poll::Pending
                }
			}
		}
	};
}
make_tuples!(0);
make_tuples!(0, 1);
make_tuples!(0, 1, 2);
make_tuples!(0, 1, 2, 3);
make_tuples!(0, 1, 2, 3, 4);
make_tuples!(0, 1, 2, 3, 4, 5);
make_tuples!(0, 1, 2, 3, 4, 5, 6);
make_tuples!(0, 1, 2, 3, 4, 5, 6, 7);
make_tuples!(0, 1, 2, 3, 4, 5, 6, 7, 8);
make_tuples!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
make_tuples!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
make_tuples!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);

// pub fn join<F, O>(futures: F) -> Join<F, O>
// where
//     Join<F, O>: From<F>,
// {
//     Join::from(futures)
// }
// pub fn race<F>(futures: F) -> Race<F>
// where
//     Race<F>: From<F>,
// {
//     Race::from(futures)
// }
