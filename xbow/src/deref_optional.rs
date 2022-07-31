use crate::{listeners::Listeners, mapper::Mapper};

pub trait ProjectedDeref {
    type Target: ?Sized;
    fn deref_optional(&self) -> Option<&Self::Target>;
    fn fire_listeners(&self);
}
pub trait ProjectedDerefMut: ProjectedDeref {
    fn deref_mut_optional(&mut self) -> Option<&mut Self::Target>;
}
pub struct BorrowWrapped<'b, W, M>
where
    W: ProjectedDeref,
    M: Mapper<In = W::Target>,
{
    wrapped: W,
    mapper: M,
    listeners: Option<&'b Listeners>,
}

impl<'b, W, M> BorrowWrapped<'b, W, M>
where
    W: ProjectedDeref,
    M: Mapper<In = W::Target>,
{
    pub fn new(wrapped: W, mapper: M, listeners: Option<&'b Listeners>) -> Self {
        Self {
            wrapped,
            mapper,
            listeners,
        }
    }
}

impl<'b, W, M> ProjectedDeref for BorrowWrapped<'b, W, M>
where
    W: ProjectedDeref,
    M: Mapper<In = W::Target>,
{
    type Target = M::Out;

    fn deref_optional(&self) -> Option<&Self::Target> {
        let input = self.wrapped.deref_optional()?;
        self.mapper.map(input)
    }
    fn fire_listeners(&self) {
        if let Some(listeners) = self.listeners {
            listeners.fire();
        }
        self.wrapped.fire_listeners();
    }
}
impl<'b, W, M> ProjectedDerefMut for BorrowWrapped<'b, W, M>
where
    W: ProjectedDeref + ProjectedDerefMut,
    M: Mapper<In = W::Target>,
{
    fn deref_mut_optional(&mut self) -> Option<&mut Self::Target> {
        let input = self.wrapped.deref_mut_optional()?;
        self.mapper.map_mut(input)
    }
}
