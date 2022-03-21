#[macro_export]
macro_rules! vec_into {
	($($e:expr),*) => {
		vec![
			$($e.into()),*
		]
	};
	($($e:expr,)*) => {
		$crate::vec_into![$($e),*]
	}
}
