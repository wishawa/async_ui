use std::marker::PhantomData;

pub trait Mapper {
	type Input<'i>
	where
		Self: 'i;
	type Output<'o>
	where
		Self: 'o;
	fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
	where
		Self: 'm;
}

pub struct SimpleMapper<I, O, F>
where
	F: Fn(I) -> O,
{
	mapper: F,
	_phantom: PhantomData<(I, O)>,
}

impl<I, O, F> Mapper for SimpleMapper<I, O, F>
where
	F: Fn(I) -> O,
{
	type Input<'i> = I where Self: 'i;
	type Output<'o> = O where Self: 'o;
	fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
	where
		Self: 'm,
	{
		(self.mapper)(input)
	}
}

pub struct RefMapper<I, O, F>
where
	I: ?Sized,
	O: ?Sized,
	F: Fn(&I) -> &O,
{
	mapper: F,
	_phantom: PhantomData<(*const I, *const O)>,
}

impl<I, O, F> Mapper for RefMapper<I, O, F>
where
	F: Fn(&I) -> &O,
{
	type Input<'i> = &'i I where Self: 'i;
	type Output<'o> = &'o O where Self: 'o;
	fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
	where
		Self: 'm,
	{
		(self.mapper)(input)
	}
}
