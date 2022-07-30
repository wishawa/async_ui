pub trait New {
    type Arg;
    fn new(arg: Self::Arg) -> Self;
}
