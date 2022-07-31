use crate::mapper::Mapper;

pub trait ProjectedDeref {
    type Target: ?Sized;
    fn deref_optional(&self) -> Option<&Self::Target>;
}
pub trait ProjectedDerefMut: ProjectedDeref {
    fn deref_mut_optional(&mut self) -> Option<&mut Self::Target>;
}
pub struct BorrowWrapped<W, M>
where
    W: ProjectedDeref,
    M: Mapper<In = W::Target>,
{
    wrapped: W,
    mapper: M,
}

impl<W, M> BorrowWrapped<W, M>
where
    W: ProjectedDeref,
    M: Mapper<In = W::Target>,
{
    pub fn new(wrapped: W, mapper: M) -> Self {
        Self { wrapped, mapper }
    }
}

impl<W, M> ProjectedDeref for BorrowWrapped<W, M>
where
    W: ProjectedDeref,
    M: Mapper<In = W::Target>,
{
    type Target = M::Out;

    #[inline]
    fn deref_optional(&self) -> Option<&Self::Target> {
        let input = self.wrapped.deref_optional()?;
        self.mapper.map(input)
    }
}
impl<W, M> ProjectedDerefMut for BorrowWrapped<W, M>
where
    W: ProjectedDeref + ProjectedDerefMut,
    M: Mapper<In = W::Target>,
{
    #[inline]
    fn deref_mut_optional(&mut self) -> Option<&mut Self::Target> {
        let input = self.wrapped.deref_mut_optional()?;
        self.mapper.map_mut(input)
    }
}
