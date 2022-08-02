use x_bow::{ProjectionExtGuaranteed, Store};

#[derive(x_bow::XBowProject)]
struct MyStruct {
    #[x_bow(no_project)]
    field1: i32,
    #[x_bow(no_project)]
    field2: usize,
    field3: InnerStruct,
    field4: InnerTuple,
}
#[derive(x_bow::XBowProject)]
struct InnerStruct {
    #[x_bow(no_project)]
    inner1: bool,
}
#[derive(x_bow::XBowProject)]
struct InnerTuple(#[x_bow(no_project)] bool);
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
