use std::ops::{Deref, DerefMut};

use crate::mapper::Mapper;

pub struct DerefWrapped<W, M>
where
    W: Deref,
    M: Mapper<In = W::Target>,
{
    wrapped: W,
    mapper: M,
}

impl<W, M> DerefWrapped<W, M>
where
    W: Deref,
    M: Mapper<In = W::Target>,
{
    pub fn new(wrapped: W, mapper: M) -> Self {
        Self { wrapped, mapper }
    }
}

impl<W, M> Deref for DerefWrapped<W, M>
where
    W: Deref,
    M: Mapper<In = W::Target>,
{
    type Target = M::Out;

    fn deref(&self) -> &Self::Target {
        let input = self.wrapped.deref();
        self.mapper.map(input)
    }
}
impl<W, M> DerefMut for DerefWrapped<W, M>
where
    W: Deref + DerefMut,
    M: Mapper<In = W::Target>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        let input = self.wrapped.deref_mut();
        self.mapper.map_mut(input)
    }
}
