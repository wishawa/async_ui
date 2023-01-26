use std::future::Future;

use crate::ChildFuture;
use futures_concurrency::future::{Join, Race, RaceOk as TryRace, TryJoin};

pub trait FragmentWrap: Sized {
    type Wrapped;
    type Joined
    where
        Self::Wrapped: Join;
    type Raced
    where
        Self::Wrapped: Race;
    type TryJoined
    where
        Self::Wrapped: TryJoin;
    type TryRaced
    where
        Self::Wrapped: TryRace;

    fn fragment_wrap(self) -> Self::Wrapped;

    fn fragment_join(self) -> Self::Joined
    where
        Self::Wrapped: Join;
    fn fragment_race(self) -> Self::Raced
    where
        Self::Wrapped: Race;
    fn fragment_try_join(self) -> Self::TryJoined
    where
        Self::Wrapped: TryJoin;
    fn fragment_try_race(self) -> Self::TryRaced
    where
        Self::Wrapped: TryRace;
}

macro_rules! make_fragment {
	($($ty:ident)+) => {
		impl<$($ty: Future,)+> FragmentWrap for ($($ty,)+) {
			type Wrapped = ($(ChildFuture<$ty>,)+);
			type Joined = <Self::Wrapped as Join>::Future where Self::Wrapped: Join;
			type Raced = <Self::Wrapped as Race>::Future where Self::Wrapped: Race;
			type TryJoined = <Self::Wrapped as TryJoin>::Future where Self::Wrapped: TryJoin;
			type TryRaced = <Self::Wrapped as TryRace>::Future where Self::Wrapped: TryRace;
			fn fragment_wrap(self) -> Self::Wrapped {
				#[allow(non_snake_case)]
				let ($($ty,)+) = self;
				let mut idx = 0;
				(
					$(
						ChildFuture::new($ty, {idx += 1; idx}),
					)+
				)
			}
			fn fragment_join(self) -> Self::Joined where Self::Wrapped: Join { self.fragment_wrap().join() }
			fn fragment_race(self) -> Self::Raced where Self::Wrapped: Race { self.fragment_wrap().race() }
			fn fragment_try_join(self) -> Self::TryJoined where Self::Wrapped: TryJoin { self.fragment_wrap().try_join() }
			fn fragment_try_race(self) -> Self::TryRaced where Self::Wrapped: TryRace { self.fragment_wrap().race_ok() }
		}
	};
}

make_fragment!(A0 A1);
make_fragment!(A0 A1 A2);
make_fragment!(A0 A1 A2 A3);
make_fragment!(A0 A1 A2 A3 A4);
make_fragment!(A0 A1 A2 A3 A4 A5);
make_fragment!(A0 A1 A2 A3 A4 A5 A6);
make_fragment!(A0 A1 A2 A3 A4 A5 A6 A7);
make_fragment!(A0 A1 A2 A3 A4 A5 A6 A7 A8);
make_fragment!(A0 A1 A2 A3 A4 A5 A6 A7 A8 A9);
make_fragment!(A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10);
make_fragment!(A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11);
