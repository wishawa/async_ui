use std::marker::PhantomData;

pub trait Mapper {
    type In;
    type Out;
    fn map<'s, 'd>(&'s self, input: &'d Self::In) -> &'d Self::Out;
    fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> &'d mut Self::Out;
}

// impl<I, O, FR, FM> Mapper for (FR, FM, PhantomData<(I, O)>)
// where
// 	FR: Clone + Fn(&I) -> &O,
// 	FM: Clone + Fn(&mut I) -> &mut O,
// {
//     type In = I;

//     type Out = O;

//     fn map<'s, 'd>(&'s self, input: &'d Self::In) -> &'d Self::Out {
// 		(self.0)(input)
//     }

//     fn map_mut<'s, 'd>(&'s self, input: &'d mut Self::In) -> &'d mut Self::Out {
// 		(self.1)(input)
//     }
// }
