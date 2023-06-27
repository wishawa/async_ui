macro_rules! make_tuple_impl {
	($modname:ident | $($tn:ident),+) => {
		mod $modname {
			use x_bow_macros::Trackable;
			type Tuple<$($tn),+> = ($($tn,)+);
			#[allow(dead_code)]
			#[derive(Trackable)]
			#[x_bow(module_prefix = crate::__private_macro_only)]
			#[x_bow(remote_type = Tuple)]
			#[track(deep)]
			pub struct Imitator<$($tn),+>($($tn,)+);
		}
	};
}

make_tuple_impl!(t1 | T0);
make_tuple_impl!(t2 | T0, T1);
make_tuple_impl!(t3 | T0, T1, T2);
make_tuple_impl!(t4 | T0, T1, T2, T3);
make_tuple_impl!(t5 | T0, T1, T2, T3, T4);
make_tuple_impl!(t6 | T0, T1, T2, T3, T4, T5);
make_tuple_impl!(t7 | T0, T1, T2, T3, T4, T5, T6);
make_tuple_impl!(t8 | T0, T1, T2, T3, T4, T5, T6, T7);
make_tuple_impl!(t9 | T0, T1, T2, T3, T4, T5, T6, T7, T8);
make_tuple_impl!(t10 | T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
make_tuple_impl!(t11 | T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
make_tuple_impl!(t12 | T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
