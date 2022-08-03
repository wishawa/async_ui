use x_bow::{Store, TrackedExt, TrackedExtGuaranteed};

pub mod play {

    #[derive(x_bow::Track)]
    pub struct MyStruct<C: FnOnce(i32)> {
        #[x_bow(no_track)]
        pub field1: i32,
        #[x_bow(no_track)]
        pub field2: usize,
        pub field3: InnerStruct<C>,
        pub field4: InnerTuple,
        pub field5: GenericStruct<Vec<bool>>,
        pub ef: MyEnum,
    }
    #[derive(x_bow::Track)]
    pub struct InnerStruct<C: FnOnce(i32)> {
        #[x_bow(no_track)]
        pub inner1: bool,
        pub inner2: RunOnDrop<i32, C>,
        pub inner3: GenericEnum<i32, Box<i32>>,
    }
    #[derive(x_bow::Track)]
    pub struct InnerTuple(#[x_bow(no_track)] pub bool);
    #[derive(x_bow::Track)]
    pub struct GenericStruct<T> {
        pub value: Wrapped<T>,
    }
    #[derive(x_bow::Track)]
    pub struct Wrapped<T> {
        #[x_bow(no_track)]
        pub wrapped: T,
    }
    #[derive(x_bow::Track)]
    pub struct RunOnDrop<V, T: FnOnce(V)> {
        #[x_bow(no_track)]
        pub closure: T,
        #[x_bow(no_track)]
        pub value: V,
    }
    #[derive(x_bow::Track)]
    pub enum MyEnum {
        A(#[x_bow(no_track)] bool),
        B {
            #[x_bow(no_track)]
            val: i64,
            another: InnerTuple,
        },
    }
    #[derive(x_bow::Track)]
    pub enum GenericEnum<U, T: std::ops::Deref<Target = U>> {
        Pointer(#[x_bow(no_track)] T),
        Value(#[x_bow(no_track)] U),
    }
}
fn main() {
    use play::*;
    let proj = Store::new(MyStruct {
        field1: 42,
        field2: 0,
        field3: InnerStruct {
            inner1: false,
            inner2: RunOnDrop {
                closure: |v| {
                    println!("wow {}", v);
                },
                value: 42,
            },
            inner3: GenericEnum::Pointer(Box::new(5)),
        },
        field4: InnerTuple(true),
        field5: GenericStruct {
            value: Wrapped {
                wrapped: Vec::new(),
            },
        },
        ef: MyEnum::A(false),
    })
    .project();
    let b = *proj.field1.borrow();
    let b = *proj.field3.inner1.borrow();
    let b = proj.field4.0;
    let b = &**proj.field5.value.wrapped.borrow();
    let b = proj.field3.inner2.closure.borrow();
    let b = proj.ef.A_0;

    let b = &*proj.ef.B_another.borrow_opt().unwrap();
    let b = proj.field3.inner3.Pointer_0.borrow_opt().unwrap();

    println!("Hello, world!");
}
