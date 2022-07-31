pub trait Mapper {
    type In;
    type Out;
    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out>;
    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out>;
}
