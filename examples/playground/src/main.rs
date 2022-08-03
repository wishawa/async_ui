use x_bow::{Store, TrackedExtGuaranteed};

#[derive(x_bow::Track)]
struct MyStruct {
    #[x_bow(no_track)]
    field1: i32,
    #[x_bow(no_track)]
    field2: usize,
    field3: InnerStruct,
    field4: InnerTuple,
}
#[derive(x_bow::Track)]
struct InnerStruct {
    #[x_bow(no_track)]
    inner1: bool,
}
#[derive(x_bow::Track)]
struct InnerTuple(#[x_bow(no_track)] bool);
fn main() {
    let store = Store::new(MyStruct {
        field1: 42,
        field2: 0,
        field3: InnerStruct { inner1: false },
        field4: InnerTuple(true),
    });
    let proj = store.project();
    let i = *proj.field1.borrow();
    let b = *proj.field3.inner1.borrow();
    let k = proj.field4.0;

    println!("Hello, world!");
}
