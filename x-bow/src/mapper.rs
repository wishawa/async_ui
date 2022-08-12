use std::marker::PhantomData;

pub trait Mapper {
    type In;
    type Out;
    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out>;
    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out>;
}

pub struct ClosureMapper<I, O, FRef, FRefMut>
where
    FRef: Fn(&I) -> Option<&O>,
    FRefMut: Fn(&mut I) -> Option<&mut O>,
{
    immutable: FRef,
    mutable: FRefMut,
    _phantom: PhantomData<(I, O)>,
}

impl<I, O, FRef, FRefMut> Mapper for ClosureMapper<I, O, FRef, FRefMut>
where
    FRef: Fn(&I) -> Option<&O>,
    FRefMut: Fn(&mut I) -> Option<&mut O>,
{
    type In = I;
    type Out = O;
    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out> {
        (self.immutable)(input)
    }

    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out> {
        (self.mutable)(input)
    }
}

impl<I, O, FRef, FRefMut> ClosureMapper<I, O, FRef, FRefMut>
where
    FRef: Fn(&I) -> Option<&O>,
    FRefMut: Fn(&mut I) -> Option<&mut O>,
{
    pub fn new(immutable: FRef, mutable: FRefMut) -> Self {
        Self {
            immutable,
            mutable,
            _phantom: PhantomData,
        }
    }
}
