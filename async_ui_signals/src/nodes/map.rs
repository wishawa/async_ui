use std::marker::PhantomData;

use crate::{Visitable, sub::ParentSub};

pub trait Mapper {
    type Input<'i> where Self: 'i;
    type Output<'o> where Self: 'o;
    fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
    where
        Self: 'm;
}

pub struct SimpleMapper<I, O, F>
where
    F: Fn(I) -> O
{
    mapper: F,
    _phantom: PhantomData<(I, O)>
}

impl<I, O, F> Mapper for SimpleMapper<I, O, F>
where
    F: Fn(I) -> O
{
    type Input<'i> = I where Self: 'i;
    type Output<'o> = O where Self: 'o;
    fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
    where
        Self: 'm
    {
        (self.mapper)(input)
    }
}

pub struct RefMapper<I, O, F>
where
    F: Fn(&I) -> &O
{
    mapper: F,
    _phantom: PhantomData<(I, O)>
}

impl<I, O, F> Mapper for RefMapper<I, O, F>
where
    F: Fn(&I) -> &O
{
    type Input<'i> = &'i I where Self: 'i;
    type Output<'o> = &'o O where Self: 'o;
    fn map<'m, 's>(&'s self, input: Self::Input<'m>) -> Self::Output<'m>
    where
        Self: 'm
    {
        (self.mapper)(input)
    }
}

pub struct SignalMap<'p, M>
where
    M: Mapper + 'p,
{
    parent: &'p dyn for<'k> Visitable<dyn for<'x> FnMut(M::Input<'x>) + 'k>,
    mapper: M,
}

impl<'p, M> SignalMap<'p, M>
where
    M: Mapper + 'p,
{
    pub fn new(parent: &'p dyn for<'k> Visitable<dyn for<'x> FnMut(M::Input<'x>) + 'k>, mapper: M) -> Self { Self { parent, mapper } }
}

impl<'a, 'p, M, V> Visitable<V> for SignalMap<'p, M>
where
    M: Mapper + 'p,
    V: ?Sized + for<'x> FnMut(M::Output<'x>) + 'a
{
    fn visit<'v, 's>(&'s self, visitor: &'v mut V)
    where
        Self: 'v
    {
        let mut wrapped_visitor = |input: M::Input<'_>| {
            let output = self.mapper.map(input);
            visitor(output);
        };
        self.parent
            .visit(&mut wrapped_visitor as &mut (dyn for<'x> FnMut(M::Input<'x>) + '_));
    }
	fn get_sub<'s>(&'s self) -> ParentSub<'s> {
		self.parent.get_sub()
	}
}