use std::marker::PhantomData;

pub struct True;
pub struct False;
pub trait Boolean {
    const VALUE: bool;
}
impl Boolean for True {
    const VALUE: bool = true;
}
impl Boolean for False {
    const VALUE: bool = false;
}
// pub struct BooleanAnd<I1: Boolean, I2: Boolean>(PhantomData<(I1, I2)>);
// impl<I1: Boolean, I2: Boolean> Boolean for BooleanAnd<I1, I2> {
//     const VALUE: bool = I1::VALUE && I2::VALUE;
// }

// pub struct BooleanOr<I1: Boolean, I2: Boolean>(PhantomData<(I1, I2)>);
// impl<I1: Boolean, I2: Boolean> Boolean for BooleanOr<I1, I2> {
//     const VALUE: bool = I1::VALUE || I2::VALUE;
// }
