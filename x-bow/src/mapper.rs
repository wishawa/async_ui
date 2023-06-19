/// A mapper is an object that extracts part of an object
/// * A mapper might map a struct to one of its fields.
/// * A different mapper might map an enum to one of its variants.
/// * A more complicated mapper might map an array to its element at a specific index.
/// The act of "mapping" is quite similar to dereferencing.
/// The difference is that we have runtime mapper objects to determine how the derefing is done.
/// Note that in many cases, these runtime objects are zero-sized (so there is no additional cost).
pub trait Mapper {
    type In;
    type Out;
    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> Option<&'d Self::Out>;
    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> Option<&'d mut Self::Out>;
}
