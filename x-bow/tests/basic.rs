use x_bow::{create_store, Track};

#[derive(Track)]
struct MyStruct {
    number: i32,
}
#[test]
fn structure() {
    let store = create_store(MyStruct { number: 5 });
    assert_eq!(*store.number.borrow(), 5);
    *store.number.borrow_mut() = 42;
    assert_eq!(*store.number.borrow(), 42);
    *store.borrow_mut() = MyStruct { number: 7 };
    assert_eq!(*store.number.borrow(), 7);
}

#[derive(Track)]
enum MyEnum {
    First { number: i32 },
    Second { value: bool },
}
#[test]
fn enumeration() {
    let store = create_store(MyEnum::First { number: 5 });
    assert_eq!(*store.First_number.borrow_opt().unwrap(), 5);
    assert!(store.Second_value.borrow_opt().is_none());
    *store.First_number.borrow_mut_opt().unwrap() = 42;
    assert_eq!(*store.First_number.borrow_opt().unwrap(), 42);
    *store.borrow_mut() = MyEnum::Second { value: true };
    assert!(store.First_number.borrow_opt().is_none());
    assert_eq!(*store.Second_value.borrow_opt().unwrap(), true);
}
